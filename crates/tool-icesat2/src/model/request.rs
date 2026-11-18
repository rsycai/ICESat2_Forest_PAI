use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ProcessRequest {
    pub atl03: PathBuf,
    pub atl08: PathBuf,
    pub output: PathBuf,
    pub manifest: Option<PathBuf>,
    pub beams: Vec<String>,
    pub overwrite: bool,
}
