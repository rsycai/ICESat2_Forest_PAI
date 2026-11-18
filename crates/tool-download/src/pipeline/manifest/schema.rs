use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DownloadDetails {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub granule_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub local_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acquired_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_size_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub catalog_size_mb: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub size_status: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_error_code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogState {
    Preparing,
    Ready,
    Interrupted,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DayReference {
    #[serde(rename = "@date")]
    pub date: String,
    #[serde(rename = "@data_type")]
    pub data_type: String,
    pub path: String,
    pub entries: usize,
    pub files: usize,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "download_manifest")]
pub struct ManifestIndex {
    pub schema_version: u8,
    pub run_id: String,
    pub generated_at: String,
    pub status: CatalogState,
    pub files: usize,
    pub selected_files: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temporal: Option<String>,
    #[serde(rename = "day", default)]
    pub days: Vec<DayReference>,
}
