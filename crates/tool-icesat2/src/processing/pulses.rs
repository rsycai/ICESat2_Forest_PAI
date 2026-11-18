use crate::model::{BeamResult, GeoSegment};
use crate::processing::error::{bad, h5};
use crate::processing::reading::read1;
use hdf5_pure::File;
use std::collections::HashSet;
use xt_error::Result;

pub(super) fn counts(
    file: &File,
    beam: &str,
    geo: &[GeoSegment],
    result: &BeamResult,
) -> Result<Option<Vec<(i32, i32)>>> {
    let path = format!("{beam}/heights");
    let names = file.group(&path).map_err(h5)?.datasets().map_err(h5)?;
    let has = |key| names.iter().any(|name| name == key);
    if !has("ph_id_pulse") && !has("pce_mframe_cnt") {
        return Ok(None);
    }
    let frames = read1::<u32>(file, &format!("{path}/pce_mframe_cnt"))?;
    let pulses = read1::<u8>(file, &format!("{path}/ph_id_pulse"))?;
    if frames.len() != pulses.len() || frames.len() as u64 != result.stats.atl03_photons {
        return Err(bad("ATL03 pulse IDs and photon columns differ in length"));
    }
    let key = |i: usize| {
        let frame = frames[i];
        let pulse = pulses[i];
        (1..=201).contains(&pulse).then_some((frame, pulse))
    };
    let mut observed = HashSet::new();
    let mut ground = HashSet::new();
    let mut out = Vec::with_capacity(result.segments.len());
    for s in &result.segments {
        observed.clear();
        ground.clear();
        let start = geo.partition_point(|g| g.id < s.begin_id);
        for g in geo[start..].iter().take_while(|g| g.id <= s.end_id) {
            if g.photon_count > 0 {
                let first = (g.first_photon - 1) as usize;
                observed.extend((first..first + g.photon_count as usize).filter_map(key));
            }
        }
        for p in &result.photons[s.photon_start..s.photon_start + s.photon_count] {
            if p.classification == 1
                && let Some(id) = key(p.atl03_index as usize)
            {
                ground.insert(id);
            }
        }
        out.push((observed.len() as i32, ground.len() as i32));
    }
    Ok(Some(out))
}
