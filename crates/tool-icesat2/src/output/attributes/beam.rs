use crate::output::attributes::error::h5;
use crate::output::attributes::selection::{numeric, segment_group};
use crate::processing::config::Icesat2Config;
use crate::model::BeamResult;
use hdf5_pure::{File, GroupBuilder};
use xt_error::Result;

pub(crate) fn beam(
    product03: &mut GroupBuilder,
    product08: &mut GroupBuilder,
    a: &File,
    b: &File,
    name: &str,
    result: &BeamResult,
    options: &Icesat2Config,
) -> Result<Vec<u64>> {
    let window = options.read_window_rows as u64;
    let mut beam03 = product03.create_group(name);
    let mut heights = beam03.create_group("heights");
    let source = a.group(&format!("{name}/heights")).map_err(h5)?;
    let names = source.datasets().map_err(h5)?;
    let photons03 = result
        .photons
        .iter()
        .map(|p| p.atl03_index)
        .collect::<Vec<_>>();
    for column in [
        "lat_ph",
        "lon_ph",
        "h_ph",
        "delta_time",
        "dist_ph_along",
        "dist_ph_across",
        "quality_ph",
        "signal_conf_ph",
        "weight_ph",
        "signal_class_ph",
        "ph_id_channel",
        "ph_id_pulse",
        "ph_id_count",
        "pce_mframe_cnt",
    ] {
        if names.iter().any(|n| n == column) {
            numeric(
                &source.dataset(column).map_err(h5)?,
                &mut heights,
                column,
                &photons03,
                result.stats.atl03_photons,
                window,
            )?;
        }
    }
    beam03.add_group(heights.finish());
    let source = a.group(&format!("{name}/geolocation")).map_err(h5)?;
    let ids = source
        .dataset("segment_id")
        .map_err(h5)?
        .read_i32()
        .map_err(h5)?;
    let selected = result
        .photons
        .iter()
        .map(|p| p.geolocation_segment_id)
        .collect::<std::collections::BTreeSet<_>>();
    let geo_rows = ids
        .iter()
        .enumerate()
        .filter(|(_, id)| selected.contains(id))
        .map(|(i, _)| i as u64)
        .collect::<Vec<_>>();
    let names = source.datasets().map_err(h5)?;
    let mut geo = beam03.create_group("geolocation");
    for column in [
        "segment_id",
        "ph_index_beg",
        "segment_ph_cnt",
        "segment_dist_x",
        "delta_time",
    ] {
        if names.iter().any(|n| n == column) {
            numeric(
                &source.dataset(column).map_err(h5)?,
                &mut geo,
                column,
                &geo_rows,
                ids.len() as u64,
                window,
            )?;
        }
    }
    beam03.add_group(geo.finish());
    product03.add_group(beam03.finish());
    let mut beam08 = product08.create_group(name);
    let mut signals = beam08.create_group("signal_photons");
    let source = b.group(&format!("{name}/signal_photons")).map_err(h5)?;
    let photons08 = result
        .photons
        .iter()
        .map(|p| p.atl08_index)
        .collect::<Vec<_>>();
    for column in source.datasets().map_err(h5)? {
        numeric(
            &source.dataset(&column).map_err(h5)?,
            &mut signals,
            &column,
            &photons08,
            result.stats.atl08_photons,
            window,
        )?;
    }
    beam08.add_group(signals.finish());
    let mut segments = beam08.create_group("land_segments");
    let rows = result
        .segments
        .iter()
        .map(|s| s.source_row as u64)
        .collect::<Vec<_>>();
    segment_group(
        &b.group(&format!("{name}/land_segments")).map_err(h5)?,
        &mut segments,
        &rows,
        result.stats.input_segments,
        window,
        0,
    )?;
    beam08.add_group(segments.finish());
    product08.add_group(beam08.finish());
    Ok(geo_rows)
}
