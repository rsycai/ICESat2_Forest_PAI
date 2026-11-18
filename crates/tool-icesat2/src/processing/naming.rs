use crate::processing::error::bad;
use std::path::{Path, PathBuf};
use xt_error::Result;

pub(super) fn filename_key(path: &Path, product: &str) -> Result<String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| bad("Input filename must be a standard UTF-8 ICESat-2 product name"))?;
    let fields = name
        .strip_suffix(".h5")
        .ok_or_else(|| bad("Input file must have a .h5 extension"))?
        .split('_')
        .collect::<Vec<_>>();
    if fields.len() != 5
        || fields[0] != product
        || [14, 8, 3, 2].iter().enumerate().any(|(i, &n)| {
            fields[i + 1].len() != n || !fields[i + 1].bytes().all(|c| c.is_ascii_digit())
        })
    {
        return Err(bad(format!(
            "{product} filename must follow PRODUCT_TIME_ORBITCYCLEREGION_RELEASE_REVISION.h5"
        )));
    }
    Ok(fields[1..4].join("_"))
}

pub fn paired_key(atl03: &Path, atl08: &Path) -> Result<String> {
    let a = filename_key(atl03, "ATL03")?;
    let b = filename_key(atl08, "ATL08")?;
    if a != b {
        return Err(bad(
            "ATL03/ATL08 observation mismatch: time, orbit, cycle, region, or release",
        ));
    }
    Ok(a)
}

pub fn paired_output_name(atl03: &Path, atl08: &Path) -> Result<String> {
    paired_key(atl03, atl08)?;
    let name = |p: &Path| {
        p.file_name()
            .and_then(|s| s.to_str())
            .map(str::to_owned)
            .ok_or_else(|| bad("Invalid original filename"))
    };
    Ok(format!(
        "{}.h5",
        xt_icesat2_meta::manifest::pair_stem(&name(atl03)?, &name(atl08)?)?
    ))
}

pub fn dated_output_path(root: &Path, atl03: &Path, atl08: &Path) -> Result<PathBuf> {
    let key = paired_key(atl03, atl08)?;
    let stamp = key.split('_').next().ok_or_else(|| bad("Missing acquisition time"))?;
    let date = chrono::NaiveDateTime::parse_from_str(stamp, "%Y%m%d%H%M%S")
        .map_err(|_| bad("Invalid acquisition date in original product"))?;
    Ok(root
        .join(date.format("%Y").to_string())
        .join(date.format("%m").to_string())
        .join(date.format("%d").to_string())
        .join(paired_output_name(atl03, atl08)?))
}
