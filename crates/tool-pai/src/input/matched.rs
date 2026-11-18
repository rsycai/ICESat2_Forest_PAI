use crate::{
    calculation::{projection, segment},
    model::{BEAMS, Location, Row},
};
use anyhow::{Context, Result, ensure};
use hdf5_pure::{File, H5Element};
use std::{collections::BTreeMap, path::Path};

fn column<T: H5Element>(f: &File, path: &str) -> Result<Vec<T>> {
    let d = f.dataset(path).with_context(|| path.to_owned())?;
    ensure!(d.shape()?.len() == 1, "{path} must be a one-dimensional column");
    Ok(d.read()?)
}

pub(crate) fn read(
    path: &Path,
    raster: &(),
) -> Result<BTreeMap<String, Vec<Row>>> {
    let f = File::open_streaming(path)?;
    let names = f.group("ATL03_08")?.groups()?;
    let mut out = BTreeMap::new();
    for beam in BEAMS.into_iter().filter(|b| names.iter().any(|s| s == b)) {
        let s = format!("ATL08/{beam}/land_segments");
        let p = format!("ATL08/{beam}/signal_photons");
        let m = format!("ATL03_08/{beam}");
        let begin = column::<i32>(&f, &format!("{s}/segment_id_beg"))?;
        let end = column::<i32>(&f, &format!("{s}/segment_id_end"))?;
        let lon = column::<f32>(&f, &format!("{s}/longitude"))?;
        let lat = column::<f32>(&f, &format!("{s}/latitude"))?;
        let time = column::<f64>(&f, &format!("{s}/delta_time"))?;
        let ids = column::<u64>(&f, &format!("{m}/segments/source_index"))?;
        ensure!(
            [end.len(), lon.len(), lat.len(), time.len(), ids.len()]
                .into_iter()
                .all(|n| n == begin.len()),
            "{beam}: segment columns have inconsistent lengths"
        );
        let x = column::<f64>(&f, &format!("{m}/photons/along_track_m"))?;
        let height_path = format!("{p}/ph_h");
        let h = column::<f32>(&f, &height_path)?;
        let attributes = f.dataset(&height_path)?.attrs()?;
        let fill = attributes.get("_FillValue").and_then(|value| match value {
            hdf5_pure::AttrValue::F32(v) => Some(*v),
            hdf5_pure::AttrValue::F64(v) => Some(*v as f32),
            _ => None,
        });
        let flags = column::<i8>(&f, &format!("{p}/classed_pc_flag"))?;
        let rows = column::<u64>(&f, &format!("{m}/photons/land_segment_index"))?;
        ensure!(
            [x.len(), h.len(), flags.len()]
                .into_iter()
                .all(|n| n == rows.len()),
            "{beam}: photon columns have inconsistent lengths"
        );
        let lookup: BTreeMap<_, _> = ids.iter().copied().zip(0..ids.len()).collect();
        ensure!(lookup.len() == ids.len(), "{beam}: duplicate segment indices");
        let mut groups = vec![Vec::new(); ids.len()];
        for (i, id) in rows.iter().enumerate() {
            let j = *lookup.get(id).context("Photon references a missing segment")?;
            if matches!(flags[i], 1..=3) && Some(h[i]) != fill {
                groups[j].push((x[i], h[i]));
            }
        }
        let g0 = projection::sample(raster, lon.iter().copied().zip(lat.iter().copied()))?;
        let results = groups
            .into_iter()
            .enumerate()
            .map(|(i, g)| {
                segment(
                    Location {
                        begin: begin[i],
                        end: end[i],
                        time: time[i],
                        lon: lon[i],
                        lat: lat[i],
                    },
                    g.into_iter(),
                    g0[i],
                )
            })
            .collect();
        out.insert(beam.to_owned(), results);
    }
    Ok(out)
}

