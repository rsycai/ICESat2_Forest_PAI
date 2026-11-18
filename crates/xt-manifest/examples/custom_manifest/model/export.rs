use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub(crate) struct ExportFields {
    pub format: String,
    pub rows: u64,
}
