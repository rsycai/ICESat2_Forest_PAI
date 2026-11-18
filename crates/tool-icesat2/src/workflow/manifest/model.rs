use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchDetails {
    pub photons: u64,
    pub segments: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_sha256: Option<String>,
}
