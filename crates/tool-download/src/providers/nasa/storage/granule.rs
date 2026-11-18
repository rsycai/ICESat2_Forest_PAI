use crate::domain::{DataGroup, Granule, GroupIdentity};
use crate::interfaces::StoragePolicy;
use crate::services::storage::{DirectoryLayout, StandardStorage};
use std::path::PathBuf;
use xt_error::Result;

#[derive(Default)]
pub struct NasaGranuleStorage {
    pub layout: DirectoryLayout,
}
impl StoragePolicy for NasaGranuleStorage {
    fn identity(&self, group: &DataGroup) -> Result<GroupIdentity> {
        let mut identity = StandardStorage {
            layout: self.layout,
        }
        .identity(group)?;
        identity.acquired_at = group.acquired_at.clone().or_else(|| {
            group
                .granules
                .first()
                .and_then(crate::providers::nasa::metadata::acquisition)
        });
        Ok(identity)
    }
    fn directory(&self, identity: &GroupIdentity) -> Result<PathBuf> {
        StandardStorage {
            layout: self.layout,
        }
        .directory(identity)
    }
    fn acquired_at(&self, granule: &Granule) -> Option<String> {
        crate::providers::nasa::metadata::acquisition(granule)
    }
}
