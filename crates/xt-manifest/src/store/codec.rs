use crate::error::invalid;
use crate::store::paths::parent_dir;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fs;
use std::io::Write;
use std::path::Path;
use xt_error::{Error, Result};

pub fn read_xml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    quick_xml::de::from_str(&fs::read_to_string(path)?)
        .map_err(|e| invalid(format!("Invalid XML manifest: {e}")))
}

pub fn write_xml(path: &Path, value: &impl Serialize) -> Result<()> {
    let parent = parent_dir(path);
    fs::create_dir_all(parent)?;
    let mut body = String::new();
    let mut serializer = quick_xml::se::Serializer::new(&mut body);
    serializer.indent(' ', 2);
    value
        .serialize(serializer)
        .map_err(|e| invalid(format!("Failed to generate XML manifest: {e}")))?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    writeln!(temp, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{body}")?;
    temp.as_file().sync_all()?;
    temp.persist(path).map_err(|e| Error::from(e.error))?;
    Ok(())
}

pub(super) fn xml_model<T: Serialize, U: DeserializeOwned>(value: &T) -> Result<U> {
    let xml =
        quick_xml::se::to_string(value).map_err(|e| invalid(format!("Failed to generate manifest model: {e}")))?;
    quick_xml::de::from_str(&xml).map_err(|e| invalid(format!("Failed to read manifest model: {e}")))
}
