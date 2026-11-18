use crate::model::status::OperationStatus;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct DataReference {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@paired")]
    pub paired: bool,
    pub path: String,
    pub files: usize,
    #[serde(rename = "status", default)]
    pub status: Vec<OperationStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "daily_index", deny_unknown_fields)]
pub struct DailyIndex {
    pub run_id: String,
    pub date: String,
    pub data_type: String,
    #[serde(rename = "data", default)]
    pub entries: Vec<DataReference>,
}
