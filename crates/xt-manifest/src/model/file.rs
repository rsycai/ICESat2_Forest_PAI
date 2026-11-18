use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileRecord<T = crate::XmlValue> {
    #[serde(rename = "@id")]
    pub product: String,
    pub tool: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    pub name: String,
    pub local_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    pub details: T,
}

impl<T> FileRecord<T> {
    pub fn touch(&mut self) {
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }
}

impl<T> Deref for FileRecord<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.details
    }
}

impl<T> DerefMut for FileRecord<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.details
    }
}
