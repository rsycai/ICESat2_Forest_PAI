use crate::processing::matching::error::invalid;
use crate::model::{BeamStats, ClassifiedPhoton, GeoSegment, LandSegment, MatchPlan, MatchedIndex};
use xt_error::Result;

pub fn build_match_plan(
    geo: &[GeoSegment],
    land: &[LandSegment],
    classified: &[ClassifiedPhoton],
    photon_count: u64,
) -> Result<MatchPlan> {
    for (i, g) in geo.iter().enumerate() {
        if g.id <= 0 || (i > 0 && g.id <= geo[i - 1].id) || g.photon_count < 0 || g.first_photon < 0
        {
            return Err(invalid(
                "ATL03 geolocation segment IDs must increase strictly; photon indices and counts must be nonnegative",
            ));
        }
        if g.photon_count > 0 {
            if g.first_photon == 0 || !g.start_m.is_finite() || g.start_m == f64::MAX {
                return Err(invalid("Nonempty ATL03 segment lacks a valid starting photon index or along-track origin"));
            }
            let end = (g.first_photon as u64 - 1)
                .checked_add(g.photon_count as u64)
                .ok_or_else(|| invalid("ATL03 segment index overflow"))?;
            if end > photon_count {
                return Err(invalid("ATL03 segment photon range exceeds the heights dataset"));
            }
        }
    }
    let mut previous_end = 0;
    for g in geo.iter().filter(|g| g.photon_count > 0) {
        let start = g.first_photon as u64 - 1;
        if start < previous_end {
            return Err(invalid("Overlapping ATL03 geolocation segment photon ranges"));
        }
        previous_end = start + g.photon_count as u64;
    }
    for (i, s) in land.iter().enumerate() {
        if s.begin_id <= 0 || s.end_id < s.begin_id || (i > 0 && s.begin_id <= land[i - 1].end_id) {
            return Err(invalid("ATL08 land segments must be ordered and nonoverlapping"));
        }
    }
    let mut stats = BeamStats {
        atl03_photons: photon_count,
        atl08_photons: classified.len() as u64,
        input_segments: land.len() as u64,
        ..Default::default()
    };
    let mut photons = Vec::new();
    for (atl08_index, p) in classified.iter().enumerate() {
        let Ok(geo_row) = geo.binary_search_by_key(&p.segment_id, |g| g.id) else {
            stats.unmatched_geo_photons += 1;
            continue;
        };
        let g = &geo[geo_row];
        if p.relative_index <= 0 || p.relative_index > g.photon_count || g.first_photon <= 0 {
            return Err(invalid(format!(
                "ATL08 row {atl08_index}: photon index exceeds ATL03 segment {}",
                g.id
            )));
        }
        let candidate = land.partition_point(|s| s.begin_id <= p.segment_id);
        if candidate == 0 || p.segment_id > land[candidate - 1].end_id {
            stats.unmatched_land_photons += 1;
            continue;
        }
        photons.push(MatchedIndex {
            atl03_index: (g.first_photon as u64 - 1) + (p.relative_index as u64 - 1),
            atl08_index: atl08_index as u64,
            geo_row,
            land_row: candidate - 1,
            classification: p.classification,
            height_above_ground: p.height_above_ground,
        });
    }
    photons.sort_unstable_by_key(|p| p.atl03_index);
    if photons
        .windows(2)
        .any(|w| w[0].atl03_index == w[1].atl03_index)
    {
        return Err(invalid("Duplicate ATL03 photon mappings in ATL08; refusing double counting"));
    }
    stats.matched_photons = photons.len() as u64;
    Ok(MatchPlan { photons, stats })
}
