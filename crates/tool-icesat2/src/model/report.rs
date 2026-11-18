use crate::model::matching::BeamStats;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
pub struct BeamReport {
    pub beam: String,
    pub stats: BeamStats,
}

#[derive(Debug, Serialize)]
pub struct ProcessReport {
    pub skipped: bool,
    pub pair_key: String,
    pub output: PathBuf,
    pub manifest: PathBuf,
    pub beams: Vec<BeamReport>,
}
