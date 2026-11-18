use crate::processing::config::Icesat2Config;
use crate::model::BeamResult;
use crate::output::column;
use crate::output::metadata::json;
use crate::output::model::OutputBuilder;
use hdf5_pure::AttrValue;
use xt_error::Result;

pub(crate) fn add_beam(
    b: &mut OutputBuilder,
    name: &str,
    result: &BeamResult,
    counts: Option<&[(i32, i32)]>,
    a: &hdf5_pure::File,
    source08: &hdf5_pure::File,
    options: &Icesat2Config,
) -> Result<()> {
    let mut beam = b.matched.create_group(name);
    beam.set_attr("statistics", AttrValue::String(json(&result.stats)?));
    let mut p = beam.create_group("photons");
    let photons = &result.photons;
    column(
        &mut p,
        "atl03_index",
        photons.iter().map(|p| p.atl03_index).collect(),
        None,
    );
    column(
        &mut p,
        "atl08_index",
        photons.iter().map(|p| p.atl08_index).collect(),
        None,
    );
    column(
        &mut p,
        "land_segment_index",
        photons.iter().map(|p| p.land_row as u64).collect(),
        None,
    );
    column(
        &mut p,
        "along_track_m",
        photons.iter().map(|p| p.along_track_m).collect(),
        Some("m"),
    );
    beam.add_group(p.finish());
    let mut s = beam.create_group("segments");
    let segments = &result.segments;
    if let Some(counts) = counts {
        column(
            &mut s,
            "n_pulses",
            counts.iter().map(|c| c.0).collect(),
            Some("1"),
        );
        column(
            &mut s,
            "n_ground_pulses",
            counts.iter().map(|c| c.1).collect(),
            Some("1"),
        );
    }
    column(
        &mut s,
        "source_index",
        segments.iter().map(|s| s.source_row as u64).collect(),
        None,
    );
    column(
        &mut s,
        "photon_start_index",
        segments.iter().map(|s| s.photon_start as u64).collect(),
        None,
    );
    column(
        &mut s,
        "photon_count",
        segments.iter().map(|s| s.photon_count as u64).collect(),
        None,
    );
    column(
        &mut s,
        "along_track_start_m",
        segments.iter().map(|s| s.start_m).collect(),
        Some("m"),
    );
    column(
        &mut s,
        "along_track_end_m",
        segments.iter().map(|s| s.end_m).collect(),
        Some("m"),
    );
    column(
        &mut s,
        "length_m",
        segments.iter().map(|s| s.length_m).collect(),
        Some("m"),
    );
    beam.add_group(s.finish());
    let geo_rows = crate::output::attributes::beam(
        &mut b.atl03,
        &mut b.atl08,
        a,
        source08,
        name,
        result,
        options,
    )?;
    let mut geo = beam.create_group("geolocation");
    column(&mut geo, "source_index", geo_rows, None);
    beam.add_group(geo.finish());
    b.matched.add_group(beam.finish());
    Ok(())
}
