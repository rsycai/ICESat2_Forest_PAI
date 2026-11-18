use proj4rs::{Proj, transform::transform};
use xt_error::{Error, ErrorKind, GeoError, Result};

pub(super) struct Projection(Option<(Proj, Proj)>);

impl Projection {
    pub fn new(epsg: Option<u32>) -> Result<Self> {
        let code = epsg.ok_or_else(|| {
            Error::new(
                ErrorKind::Geo(GeoError::MissingCrs),
                "GeoTIFF lacks a recognized EPSG coordinate system",
            )
        })?;
        if code == 4326 {
            return Ok(Self(None));
        }
        let load = |code: u32| {
            u16::try_from(code)
                .ok()
                .and_then(|id| Proj::from_epsg_code(id).ok())
                .ok_or_else(|| {
                    Error::new(
                        ErrorKind::Geo(GeoError::InvalidCrs),
                        format!("Unsupported EPSG:{code}"),
                    )
                })
        };
        Ok(Self(Some((load(4326)?, load(code)?))))
    }

    pub fn coordinates(&self, lon: f64, lat: f64) -> Result<(f64, f64)> {
        if !(-180.0..=180.0).contains(&lon) || !(-90.0..=90.0).contains(&lat) {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!("Longitude or latitude out of range ({lon}, {lat})"),
            ));
        }
        let Some((from, to)) = &self.0 else {
            return Ok((lon, lat));
        };
        let mut point = (lon.to_radians(), lat.to_radians(), 0.0);
        transform(from, to, &mut point).map_err(|e| {
            Error::new(ErrorKind::Geo(GeoError::CoordinateTransform), e.to_string())
        })?;
        Ok(if to.is_latlong() {
            (point.0.to_degrees(), point.1.to_degrees())
        } else {
            (point.0, point.1)
        })
    }
}
