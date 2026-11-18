use crate::processing::config::Icesat2Config;
use crate::processing::matching::error::invalid;
use crate::model::{BeamResult, BeamStats, LandSegment, Photon, RetainedSegment};
use xt_error::Result;

pub fn group_segments(
    land: &[LandSegment],
    mut photons: Vec<Photon>,
    mut stats: BeamStats,
    options: &Icesat2Config,
) -> Result<BeamResult> {
    options.validate()?;
    for p in &photons {
        let s = land
            .get(p.land_row)
            .ok_or_else(|| invalid("Photon ATL08 segment row is out of bounds"))?;
        if p.geolocation_segment_id < s.begin_id || p.geolocation_segment_id > s.end_id {
            return Err(invalid("Photon segment_id does not match the ATL08 segment"));
        }
    }
    photons.sort_unstable_by_key(|p| p.atl03_index);
    if photons
        .windows(2)
        .any(|w| w[0].atl03_index == w[1].atl03_index)
    {
        return Err(invalid("Duplicate photons"));
    }
    let mut previous_row = None;
    for p in &photons {
        if previous_row.is_some_and(|row| row > p.land_row) {
            return Err(invalid("ATL03 photon order differs from ATL08 segment order"));
        }
        previous_row = Some(p.land_row);
    }
    let mut segments = Vec::new();
    let mut cursor = 0;
    while cursor < photons.len() {
        let row = photons[cursor].land_row;
        let start = cursor;
        while cursor < photons.len() && photons[cursor].land_row == row {
            cursor += 1;
        }
        let points = &photons[start..cursor];
        let valid = || {
            points
                .iter()
                .map(|p| p.along_track_m)
                .filter(|x| x.is_finite() && x.abs() < f64::MAX / 2.0)
        };
        let min = valid().fold(f64::INFINITY, f64::min);
        let max = valid().fold(f64::NEG_INFINITY, f64::max);
        let (min, max) = if min.is_finite() {
            (min, max)
        } else {
            (f64::NAN, f64::NAN)
        };
        let length = max - min;
        let s = &land[row];
        segments.push(RetainedSegment {
            source_row: row,
            begin_id: s.begin_id,
            end_id: s.end_id,
            photon_start: start,
            photon_count: points.len(),
            start_m: min,
            end_m: max,
            length_m: length,
        });
    }
    stats.kept_segments = segments.len() as u64;
    stats.segments_without_matched_photons = land.len() as u64 - stats.kept_segments;
    stats.kept_photons = photons.len() as u64;
    Ok(BeamResult {
        photons,
        segments,
        stats,
    })
}
