use crate::model::state::TaskState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationStatus {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@state")]
    pub state: TaskState,
    #[serde(rename = "@target")]
    pub target: String,
    #[serde(
        rename = "@updated_at",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub updated_at: Option<String>,
    #[serde(
        rename = "@error_code",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub error_code: Option<String>,
}

impl OperationStatus {
    pub fn new(id: impl Into<String>, state: TaskState, target: impl AsRef<str>) -> Self {
        Self {
            id: id.into(),
            state,
            target: target
                .as_ref()
                .split_whitespace()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(" "),
            updated_at: None,
            error_code: None,
        }
    }

    pub fn targets(&self) -> BTreeSet<&str> {
        self.target.split_whitespace().collect()
    }

    pub fn touch(&mut self) {
        self.updated_at = Some(chrono::Utc::now().to_rfc3339());
    }

    pub fn same_key(&self, other: &Self) -> bool {
        self.id == other.id && self.targets() == other.targets()
    }
}
