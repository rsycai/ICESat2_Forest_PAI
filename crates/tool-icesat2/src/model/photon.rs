#[derive(Debug, Clone)]
pub struct ClassifiedPhoton {
    pub segment_id: i32,
    pub relative_index: i32,
    pub classification: i8,
    pub height_above_ground: f32,
}

#[derive(Debug, Clone)]
pub struct MatchedIndex {
    pub atl03_index: u64,
    pub atl08_index: u64,
    pub geo_row: usize,
    pub land_row: usize,
    pub classification: i8,
    pub height_above_ground: f32,
}

#[derive(Debug, Clone)]
pub struct Photon {
    pub atl03_index: u64,
    pub atl08_index: u64,
    pub geolocation_segment_id: i32,
    pub land_row: usize,
    pub latitude: f64,
    pub longitude: f64,
    pub height: f32,
    pub height_above_ground: f32,
    pub along_track_m: f64,
    pub delta_time: f64,
    pub classification: i8,
    pub quality: i8,
    pub signal_confidence: i8,
}
