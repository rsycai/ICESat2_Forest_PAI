use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug)]
pub struct EnvConfig {
    pub(super) variables: BTreeMap<String, String>,
    pub(super) files: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportReport {
    pub files: usize,
    pub written: usize,
    pub preserved: usize,
}
