use std::path::PathBuf;

pub struct DownloadTask {
    pub asset: Asset,
    pub destination: PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum ExistingFilePolicy {
    Fail,
    Skip,
    Overwrite,
}

#[derive(Debug, Clone, Copy)]
pub struct DownloadOptions {
    pub existing: ExistingFilePolicy,
    pub resume: bool,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            existing: ExistingFilePolicy::Fail,
            resume: false,
        }
    }
}

#[derive(Debug)]
pub enum DownloadOutcome {
    Completed {
        bytes: u64,
    },
    Skipped,
}

#[derive(Clone)]
pub struct Asset {
    pub url: String,
    pub size_bytes: Option<u64>,
    pub checksum: Option<Checksum>,
}

#[derive(Debug, Clone)]
pub struct Checksum {
    pub algorithm: String,
    pub value: String,
}
