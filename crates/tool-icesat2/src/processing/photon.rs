use crate::processing::config::Icesat2Config;
use crate::model::{GeoSegment, MatchPlan, Photon};
use crate::processing::error::{bad, h5};
use crate::processing::reading::{dataset, same_length};
use hdf5_pure::{Dataset, File};
use xt_error::Result;

pub(super) struct PhotonSource {
    pub(super) lat: Dataset,
    pub(super) lon: Dataset,
    pub(super) height: Dataset,
    pub(super) along: Dataset,
    pub(super) time: Dataset,
    pub(super) quality: Dataset,
    pub(super) confidence: Dataset,
    pub(super) count: u64,
}

impl PhotonSource {
    pub(super) fn new(file: &File, beam: &str) -> Result<Self> {
        let base = format!("{beam}/heights");
        let get = |name: &str| dataset(file, &format!("{base}/{name}"));
        let lat = get("lat_ph")?;
        let shape = lat.shape().map_err(h5)?;
        if shape.len() != 1 {
            return Err(bad("lat_ph must be a one-dimensional dataset"));
        }
        let count = shape[0];
        let source = Self {
            lat,
            lon: get("lon_ph")?,
            height: get("h_ph")?,
            along: get("dist_ph_along")?,
            time: get("delta_time")?,
            quality: get("quality_ph")?,
            confidence: get("signal_conf_ph")?,
            count,
        };
        for ds in [
            &source.lon,
            &source.height,
            &source.along,
            &source.time,
            &source.quality,
        ] {
            if ds.shape().map_err(h5)? != [count] {
                return Err(bad("Inconsistent ATL03 heights column lengths or dimensions"));
            }
        }
        if source.confidence.shape().map_err(h5)? != [count, 5] {
            return Err(bad("signal_conf_ph must have shape [photon_count,5]; land is column 0"));
        }
        Ok(source)
    }
    pub(super) fn read_matched(
        &self,
        plan: &MatchPlan,
        geo: &[GeoSegment],
        options: &Icesat2Config,
    ) -> Result<Vec<Photon>> {
        let mut photons = Vec::with_capacity(plan.photons.len());
        let mut cursor = 0;
        let size = options.read_window_rows as u64;
        while cursor < plan.photons.len() {
            let start = plan.photons[cursor].atl03_index / size * size;
            let end = (start + size).min(self.count);
            let count = end - start;
            let lat = self.lat.read_f64_rows(start, count).map_err(h5)?;
            let lon = self.lon.read_f64_rows(start, count).map_err(h5)?;
            let heights = self.height.read_f32_rows(start, count).map_err(h5)?;
            let along = self.along.read_f32_rows(start, count).map_err(h5)?;
            let times = self.time.read_f64_rows(start, count).map_err(h5)?;
            let quality = self.quality.read_i8_rows(start, count).map_err(h5)?;
            let confidence = self.confidence.read_i8_rows(start, count).map_err(h5)?;
            for (name, n) in [
                ("lat_ph", lat.len()),
                ("lon_ph", lon.len()),
                ("h_ph", heights.len()),
                ("dist_ph_along", along.len()),
                ("delta_time", times.len()),
                ("quality_ph", quality.len()),
            ] {
                same_length(count as usize, n, name)?;
            }
            same_length(count as usize * 5, confidence.len(), "signal_conf_ph window")?;
            while cursor < plan.photons.len() && plan.photons[cursor].atl03_index < end {
                let p = &plan.photons[cursor];
                let i = (p.atl03_index - start) as usize;
                let distance = if along[i].is_finite() && along[i] != f32::MAX {
                    geo[p.geo_row].start_m + along[i] as f64
                } else {
                    f64::NAN
                };
                photons.push(Photon {
                    atl03_index: p.atl03_index,
                    atl08_index: p.atl08_index,
                    geolocation_segment_id: geo[p.geo_row].id,
                    land_row: p.land_row,
                    latitude: lat[i],
                    longitude: lon[i],
                    height: heights[i],
                    height_above_ground: p.height_above_ground,
                    along_track_m: distance,
                    delta_time: times[i],
                    classification: p.classification,
                    quality: quality[i],
                    signal_confidence: confidence[i * 5],
                });
                cursor += 1;
            }
        }
        Ok(photons)
    }
}
