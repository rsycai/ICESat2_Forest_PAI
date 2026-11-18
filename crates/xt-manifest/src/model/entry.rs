use crate::model::file::FileRecord;
use crate::model::status::OperationStatus;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(
    rename = "data",
    deny_unknown_fields,
    bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>")
)]
pub struct DataEntry<T = crate::XmlValue> {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@paired")]
    pub paired: bool,
    pub data_type: String,
    pub date: String,
    #[serde(rename = "status", default)]
    pub status: Vec<OperationStatus>,
    #[serde(rename = "file", default)]
    pub files: Vec<FileRecord<T>>,
}

impl<T> DataEntry<T> {
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        self.id == other.id
            && self.paired == other.paired
            && self.data_type == other.data_type
            && self.date == other.date
    }
}
