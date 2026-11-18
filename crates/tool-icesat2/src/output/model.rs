use hdf5_pure::{FileBuilder, GroupBuilder};
use std::path::PathBuf;

pub(crate) struct OutputBuilder {
    pub(super) file: FileBuilder,
    pub(super) atl03: GroupBuilder,
    pub(super) atl08: GroupBuilder,
    pub(super) matched: GroupBuilder,
    pub(super) source03: PathBuf,
    pub(super) source08: PathBuf,
}
