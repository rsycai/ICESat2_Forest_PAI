#[derive(Clone, Copy)]
pub(crate) struct Location {
    pub begin: i32,
    pub end: i32,
    pub time: f64,
    pub lon: f32,
    pub lat: f32,
}

pub(crate) struct Row {
    pub loc: Location,
    pub pai: f32,
    pub effective: f32,
    pub ci: f32,
    pub gap: f32,
    pub qa: i8,
    pub status: u8,
}

pub(crate) const FILL: f32 = -9999.0;
pub(crate) const BEAMS: [&str; 6] = ["gt1l", "gt1r", "gt2l", "gt2r", "gt3l", "gt3r"];
