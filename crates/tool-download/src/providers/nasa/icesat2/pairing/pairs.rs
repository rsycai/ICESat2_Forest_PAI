use crate::domain::Granule;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Icesat2MatchKey {
    pub acquisition: String,
    pub reference_ground_track: u16,
    pub cycle: u8,
    pub region: u8,
    pub version: u16,
}

pub struct MatchedPair {
    pub key: Icesat2MatchKey,
    pub atl03: Granule,
    pub atl08: Granule,
}

pub struct PairSelection {
    pub pairs: Vec<MatchedPair>,
    pub unmatched: usize,
    pub unrecognized: usize,
}

pub(super) struct Candidate {
    pub(super) revision: u8,
    pub(super) granule: Granule,
}
