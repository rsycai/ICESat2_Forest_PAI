use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DataPaths {
    pub root: PathBuf,
    pub downloads: PathBuf,
    pub processed: PathBuf,
    pub products: PathBuf,
    pub manifests: PathBuf,
}
