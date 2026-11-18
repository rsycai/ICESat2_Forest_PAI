use crate::model::photon::{MatchedIndex, Photon};
use crate::model::segment::RetainedSegment;
use serde::Serialize;

#[derive(Debug)]
pub struct MatchPlan {
    pub photons: Vec<MatchedIndex>,
    pub stats: BeamStats,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct BeamStats {
    pub atl03_photons: u64,
    pub atl08_photons: u64,
    pub unmatched_geo_photons: u64,
    pub unmatched_land_photons: u64,
    pub matched_photons: u64,
    pub input_segments: u64,
    pub segments_without_matched_photons: u64,
    pub kept_segments: u64,
    pub kept_photons: u64,
}

#[derive(Debug)]
pub struct BeamResult {
    pub photons: Vec<Photon>,
    pub segments: Vec<RetainedSegment>,
    pub stats: BeamStats,
}
