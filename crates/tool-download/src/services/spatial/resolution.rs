use crate::domain::{BoundingBox, DownloadRange};
use crate::services::spatial::shapefile::read_shapefile_bounds;
use xt_error::{Error, Result};

impl DownloadRange {
    pub fn resolve(&self) -> Result<Option<BoundingBox>> {
        match self {
            Self::Global => Ok(None),
            Self::Bounds(bounds) => Ok(Some(*bounds)),
            Self::Shapefile { path, crs } => read_shapefile_bounds(path, *crs).map(Some),
        }
    }
}

impl std::str::FromStr for BoundingBox {
    type Err = Error;
    fn from_str(value: &str) -> Result<Self> {
        let coordinates = value
            .split(',')
            .map(|part| {
                part.trim().parse::<f64>().map_err(|_| {
                    crate::domain::invalid("Bounds must be four numbers: west,south,east,north")
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let [west, south, east, north] = coordinates.as_slice() else {
            return Err(crate::domain::invalid(
                "Bounds must contain four numbers: west,south,east,north",
            ));
        };
        Self::new(*west, *south, *east, *north)
    }
}
