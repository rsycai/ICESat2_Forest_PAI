mod file;
pub(crate) mod remote;
pub use file::{
    Asset, Checksum, DownloadOptions, DownloadOutcome, DownloadTask, ExistingFilePolicy,
};
