use crate::{DataGroup, Granule, GroupIdentity, Result};
use std::path::PathBuf;

pub trait StoragePolicy: Send + Sync {
    fn identity(&self, group: &DataGroup) -> Result<GroupIdentity>;
    fn directory(&self, identity: &GroupIdentity) -> Result<PathBuf>;
    fn acquired_at(&self, granule: &Granule) -> Option<String> {
        granule.acquired_at.clone()
    }
}
