#[derive(Default, Debug)]
pub struct DownloadSummary {
    pub matched_groups: u64,
    pub granules: u64,
    pub files: u64,
    pub completed: u64,
    pub skipped: u64,
    pub failed: u64,
    pub bytes: u64,
    pub manifests: Vec<std::path::PathBuf>,
}
