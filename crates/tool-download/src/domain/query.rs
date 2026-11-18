use crate::domain::spatial::BoundingBox;
use crate::domain::time_range::TimeRange;
use xt_transfer::Asset;

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub bbox: Option<BoundingBox>,
    pub time: Option<TimeRange>,
    pub page_size: u16,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            bbox: None,
            time: None,
            page_size: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PageToken(pub String);

pub struct Granule {
    pub product: String,
    pub acquired_at: Option<String>,
    pub catalog_size_mb: Option<String>,
    pub id: String,
    pub title: String,
    pub assets: Vec<Asset>,
}

pub struct SearchPage {
    pub granules: Vec<Granule>,
    pub next: Option<PageToken>,
}
