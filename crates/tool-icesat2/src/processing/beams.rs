use crate::model::BEAMS;
use crate::processing::error::{bad, h5};
use hdf5_pure::File;
use xt_error::Result;

pub(super) fn select_beams(a: &File, b: &File, requested: &[String]) -> Result<Vec<String>> {
    let groups_a = a.root().groups().map_err(h5)?;
    let groups_b = b.root().groups().map_err(h5)?;
    let candidates = if requested.is_empty() {
        BEAMS
            .iter()
            .filter(|beam| {
                groups_a.iter().any(|g| g == **beam) && groups_b.iter().any(|g| g == **beam)
            })
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        requested.to_vec()
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut beams = Vec::new();
    for beam in candidates {
        if !BEAMS.contains(&beam.as_str())
            || !seen.insert(beam.clone())
            || !groups_a.contains(&beam)
            || !groups_b.contains(&beam)
        {
            return Err(bad(format!("Invalid, duplicate, or missing beam: {beam}")));
        }
        let groups = b.group(&beam).map_err(h5)?.groups().map_err(h5)?;
        if !groups.iter().any(|name| name == "land_segments")
            && groups.iter().any(|name| name == "signal_photons")
        {
            if !requested.is_empty() {
                return Err(bad(format!(
                    "Selected beam {beam} lacks ATL08 land_segments; cannot match segments"
                )));
            }
            tracing::warn!(
                beam,
                "ATL08 beam has no land segments; skipping this beam"
            );
            continue;
        }
        beams.push(beam);
    }
    if beams.is_empty() {
        return Err(bad("No shared standard beams with matchable land segments"));
    }
    Ok(beams)
}
