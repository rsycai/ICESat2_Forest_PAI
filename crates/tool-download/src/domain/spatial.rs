use crate::domain::error::invalid;
use std::path::PathBuf;
use xt_error::Result;

#[derive(Debug, Clone, Copy, Default)]
pub enum ShapefileCrs {
    #[default]
    CheckPrj,
    AssumeWgs84,
}

#[derive(Debug, Clone, Default)]
pub enum DownloadRange {
    #[default]
    Global,
    Bounds(BoundingBox),
    Shapefile {
        path: PathBuf,
        crs: ShapefileCrs,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    pub(super) west: f64,
    pub(super) south: f64,
    pub(super) east: f64,
    pub(super) north: f64,
}

impl BoundingBox {
    pub fn coordinates(&self) -> [f64; 4] {
        [self.west, self.south, self.east, self.north]
    }

    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Result<Self> {
        if ![west, south, east, north].into_iter().all(f64::is_finite)
            || !(-180.0..=180.0).contains(&west)
            || !(-180.0..=180.0).contains(&east)
            || !(-90.0..=90.0).contains(&south)
            || !(-90.0..=90.0).contains(&north)
            || west >= east
            || south >= north
        {
            return Err(invalid("Invalid geographic bounds: require west < east and south < north"));
        }
        Ok(Self {
            west,
            south,
            east,
            north,
        })
    }
    pub fn parameter(self) -> String {
        format!("{},{},{},{}", self.west, self.south, self.east, self.north)
    }
}
