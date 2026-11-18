use crate::domain::invalid;
use crate::services::storage::paths::{data_directory_name, safe_component};
use crate::{DataGroup, GroupIdentity, Result, StoragePolicy};
use std::path::PathBuf;

#[derive(Clone, Copy, Default)]
pub enum DirectoryLayout {
    DateOnly,
    #[default]
    DateAndId,
}

#[derive(Default)]
pub struct StandardStorage {
    pub layout: DirectoryLayout,
}

impl StoragePolicy for StandardStorage {
    fn identity(&self, group: &DataGroup) -> Result<GroupIdentity> {
        let first = group
            .granules
            .first()
            .ok_or_else(|| invalid("Data group must not be empty"))?;
        Ok(GroupIdentity {
            id: group.id.clone().unwrap_or_else(|| first.id.clone()),
            data_type: if first.product.is_empty() {
                "data".into()
            } else {
                safe_component(&first.product)
            },
            acquired_at: group
                .acquired_at
                .clone()
                .or_else(|| first.acquired_at.clone()),
            paired: false,
        })
    }
    fn directory(&self, identity: &GroupIdentity) -> Result<PathBuf> {
        let date = identity
            .acquired_at
            .as_ref()
            .and_then(|v| chrono::DateTime::parse_from_rfc3339(v).ok())
            .map(|v| v.with_timezone(&chrono::Utc).format("%Y/%m/%d").to_string())
            .unwrap_or_else(|| "undated".into());
        let mut path = PathBuf::from(date);
        if matches!(self.layout, DirectoryLayout::DateAndId) {
            path.push(data_directory_name(&identity.id)?);
        }
        Ok(path)
    }
}
