use crate::domain::invalid;
use crate::pipeline::manifest::schema::ManifestIndex;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use xt_error::Result;

pub(super) fn merge_catalog(path: &Path, mut incoming: ManifestIndex) -> Result<()> {
    let mut days = BTreeMap::new();
    match fs::read_to_string(path) {
        Ok(text) => {
            let previous: ManifestIndex =
                quick_xml::de::from_str(&text).map_err(|_| invalid("Invalid existing master manifest"))?;
            if previous.schema_version != incoming.schema_version {
                return Err(invalid("Existing master manifest version mismatch"));
            }
            for day in previous.days {
                days.insert((day.date.clone(), day.data_type.clone()), day);
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    for day in incoming.days {
        days.insert((day.date.clone(), day.data_type.clone()), day);
    }
    incoming.days = days.into_values().collect();
    incoming.files = incoming.days.iter().map(|day| day.files).sum();
    xt_manifest::write_xml(path, &incoming)
}
