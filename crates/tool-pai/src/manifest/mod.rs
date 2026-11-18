mod journal;
mod signature;
mod source;

pub(crate) use journal::Journal;
pub(crate) use source::resolve;

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct Details {
    pub input_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_modified: Option<String>,
}
