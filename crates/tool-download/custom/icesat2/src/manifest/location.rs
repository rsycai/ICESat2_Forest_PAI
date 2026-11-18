use crate::error::invalid;
use crate::manifest::naming::pair_stem;
use std::path::Path;
use std::path::PathBuf;
use xt_error::Result;

pub fn record_path_in(root: &Path, atl03: &Path, atl08: &Path) -> Result<PathBuf> {
    let a = atl03
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| invalid("Invalid ATL03 filename"))?;
    let b = atl08
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| invalid("Invalid ATL08 filename"))?;
    let stem = pair_stem(a, b)?;
    let stamp = a.split('_').nth(1).ok_or_else(|| invalid("Missing observation time"))?;
    Ok(root
        .join(&stamp[..4])
        .join(&stamp[4..6])
        .join(&stamp[6..8])
        .join(format!("{stem}.xml")))
}

pub fn record_path(atl03: &Path, atl08: &Path) -> Result<PathBuf> {
    let root = [atl03, atl08]
        .into_iter()
        .find_map(|input| {
            input
                .ancestors()
                .skip(1)
                .find(|dir| dir.file_name().is_some_and(|name| name == "ATL03_08"))
        })
        .unwrap_or_else(|| atl03.parent().unwrap_or_else(|| Path::new(".")));
    record_path_in(&root.join("manifests"), atl03, atl08)
}
