#[derive(Debug, Clone)]
pub struct GeoSegment {
    pub id: i32,
    pub first_photon: i64,
    pub photon_count: i32,
    pub start_m: f64,
}

#[derive(Debug, Clone)]
pub struct LandSegment {
    pub begin_id: i32,
    pub end_id: i32,
}

#[derive(Debug, Clone)]
pub struct RetainedSegment {
    pub source_row: usize,
    pub begin_id: i32,
    pub end_id: i32,
    pub photon_start: usize,
    pub photon_count: usize,
    pub start_m: f64,
    pub end_m: f64,
    pub length_m: f64,
}
