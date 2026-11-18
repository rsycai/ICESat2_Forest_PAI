use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct DataGroup {
    pub id: String,
    pub kind: String,
    pub files: Vec<LocalFile>,
}

#[derive(Debug, Clone)]
pub struct LocalFile {
    pub id: String,
    pub role: String,
    pub path: PathBuf,
    pub size_bytes: u64,
}
