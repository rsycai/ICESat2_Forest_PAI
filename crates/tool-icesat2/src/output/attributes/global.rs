use crate::output::attributes::error::{bad, h5};
use crate::output::attributes::selection::numeric;
use hdf5_pure::{File, GroupBuilder};
use xt_error::Result;

pub(crate) fn global(product: &mut GroupBuilder, source: &File) -> Result<()> {
    let root = source.root();
    if root.groups().map_err(h5)?.iter().any(|g| g == "orbit_info") {
        let group = source.group("orbit_info").map_err(h5)?;
        let mut orbit = product.create_group("orbit_info");
        for name in group.datasets().map_err(h5)? {
            let d = group.dataset(&name).map_err(h5)?;
            let shape = d.shape().map_err(h5)?;
            if shape.is_empty() {
                return Err(bad("Orbit information must include data dimensions"));
            }
            numeric(
                &d,
                &mut orbit,
                &name,
                &(0..shape[0]).collect::<Vec<_>>(),
                shape[0],
                65536,
            )?;
        }
        product.add_group(orbit.finish());
    }
    if root
        .groups()
        .map_err(h5)?
        .iter()
        .any(|g| g == "ancillary_data")
    {
        let source = source.group("ancillary_data").map_err(h5)?;
        let names = source.datasets().map_err(h5)?;
        let mut ancillary = product.create_group("ancillary_data");
        for name in [
            "granule_start_utc",
            "granule_end_utc",
            "data_start_utc",
            "data_end_utc",
            "release",
            "version",
        ] {
            if names.iter().any(|n| n == name) {
                let d = source.dataset(name).map_err(h5)?;
                let strings = d.read_string().map_err(h5)?;
                let refs = strings.iter().map(String::as_str).collect::<Vec<_>>();
                ancillary
                    .create_dataset(name)
                    .with_strings(&refs)
                    .map_err(|e| bad(e.to_string()))?
                    .with_shape(&d.shape().map_err(h5)?);
            }
        }
        if names.iter().any(|n| n == "atlas_sdp_gps_epoch") {
            let d = source.dataset("atlas_sdp_gps_epoch").map_err(h5)?;
            let shape = d.shape().map_err(h5)?;
            numeric(
                &d,
                &mut ancillary,
                "atlas_sdp_gps_epoch",
                &(0..shape[0]).collect::<Vec<_>>(),
                shape[0],
                65536,
            )?;
        }
        product.add_group(ancillary.finish());
    }
    for name in root
        .datasets()
        .map_err(h5)?
        .into_iter()
        .filter(|n| n.starts_with("ds_"))
    {
        let d = source.dataset(&name).map_err(h5)?;
        let shape = d.shape().map_err(h5)?;
        if shape.len() != 1 || shape[0] > 1_048_576 {
            return Err(bad("Unexpected dimension scale size or shape"));
        }
        numeric(
            &d,
            product,
            &name,
            &(0..shape[0]).collect::<Vec<_>>(),
            shape[0],
            65536,
        )?;
    }
    Ok(())
}
