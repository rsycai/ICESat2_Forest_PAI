use crate::pipeline::manifest::state::DataEntry;
use crate::services::storage::paths::safe_component;
use xt_error::Result;

pub fn record_file_name(entry: &DataEntry) -> Result<String> {
    let stem = safe_component(&entry.id);
    if stem == entry.id && !stem.eq_ignore_ascii_case("index") {
        return Ok(format!("{stem}.xml"));
    }
    let asset = entry
        .files
        .first()
        .ok_or_else(|| crate::domain::invalid("Manifest data has no files"))?;
    let key = xt_transfer::asset_key(&asset.url)?;
    Ok(format!("{stem}-{}.xml", &key[..12]))
}
