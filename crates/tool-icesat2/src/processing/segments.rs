use crate::model::{ClassifiedPhoton, GeoSegment, LandSegment};
use crate::processing::reading::{read1, same_length};
use hdf5_pure::File;
use xt_error::Result;

pub(super) fn load_segments(
    a: &File,
    b: &File,
    beam: &str,
) -> Result<(Vec<GeoSegment>, Vec<LandSegment>, Vec<ClassifiedPhoton>)> {
    let g = format!("{beam}/geolocation");
    let ids = read1::<i32>(a, &format!("{g}/segment_id"))?;
    let starts = read1::<i64>(a, &format!("{g}/ph_index_beg"))?;
    let counts = read1::<i32>(a, &format!("{g}/segment_ph_cnt"))?;
    let distances = read1::<f64>(a, &format!("{g}/segment_dist_x"))?;
    for (name, n) in [
        ("ph_index_beg", starts.len()),
        ("segment_ph_cnt", counts.len()),
        ("segment_dist_x", distances.len()),
    ] {
        same_length(ids.len(), n, name)?;
    }
    let geo = ids
        .into_iter()
        .zip(starts)
        .zip(counts)
        .zip(distances)
        .map(|(((id, first_photon), photon_count), start_m)| GeoSegment {
            id,
            first_photon,
            photon_count,
            start_m,
        })
        .collect();
    let beg = read1::<i32>(b, &format!("{beam}/land_segments/segment_id_beg"))?;
    let end = read1::<i32>(b, &format!("{beam}/land_segments/segment_id_end"))?;
    same_length(beg.len(), end.len(), "ATL08 segment bounds")?;
    let land = beg
        .into_iter()
        .zip(end)
        .map(|(begin_id, end_id)| LandSegment { begin_id, end_id })
        .collect();
    let g = format!("{beam}/signal_photons");
    let ids = read1::<i32>(b, &format!("{g}/ph_segment_id"))?;
    let indices = read1::<i32>(b, &format!("{g}/classed_pc_indx"))?;
    let classes = read1::<i8>(b, &format!("{g}/classed_pc_flag"))?;
    let heights = read1::<f32>(b, &format!("{g}/ph_h"))?;
    for (name, n) in [
        ("classed_pc_indx", indices.len()),
        ("classed_pc_flag", classes.len()),
        ("ph_h", heights.len()),
    ] {
        same_length(ids.len(), n, name)?;
    }
    let classified = ids
        .into_iter()
        .zip(indices)
        .zip(classes)
        .zip(heights)
        .map(
            |(((segment_id, relative_index), classification), height_above_ground)| {
                ClassifiedPhoton {
                    segment_id,
                    relative_index,
                    classification,
                    height_above_ground,
                }
            },
        )
        .collect();
    Ok((geo, land, classified))
}
