use crate::pipeline::manifest::schema::DownloadDetails;
use std::path::PathBuf;
use xt_transfer::DownloadTask;

pub type ManifestFile = xt_manifest::FileRecord<DownloadDetails>;

pub type DataEntry = xt_manifest::DataEntry<DownloadDetails>;

pub type DataManifest = xt_manifest::DataManifest<DownloadDetails>;

pub(crate) struct PlannedFile {
    pub task: DownloadTask,
    pub group: usize,
    pub file: usize,
    pub title: String,
}

pub(crate) struct Journal {
    pub skipped_groups: std::collections::HashSet<usize>,
    pub(super) paths: xt_config::paths::PathsConfig,
    pub index_paths: std::collections::BTreeMap<String, PathBuf>,
    pub(super) run_id: String,
    pub(super) generated_at: String,
    pub(super) bbox: Option<String>,
    pub(super) temporal: Option<String>,
    pub entries: Vec<DataEntry>,
    pub(super) file_states: Vec<Vec<xt_manifest::TaskState>>,
    pub(super) record_paths: Vec<PathBuf>,
}
