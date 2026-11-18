use crate::model::entry::DataEntry;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(
    rename = "data_manifest",
    deny_unknown_fields,
    bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>")
)]
pub struct DataManifest<T = crate::XmlValue> {
    pub run_id: String,
    pub generated_at: String,
    pub data: DataEntry<T>,
}

impl<T> DataManifest<T> {
    pub fn latest_updated_at(&self) -> Option<String> {
        self.data
            .files
            .iter()
            .filter_map(|file| file.updated_at.as_deref())
            .chain(
                self.data
                    .status
                    .iter()
                    .filter_map(|status| status.updated_at.as_deref()),
            )
            .filter_map(|text| {
                chrono::DateTime::parse_from_rfc3339(text)
                    .ok()
                    .map(|time| (time, text))
            })
            .max_by_key(|(time, _)| *time)
            .map(|(_, text)| text.to_owned())
    }
}
