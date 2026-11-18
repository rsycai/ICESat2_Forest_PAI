use crate::domain::{BoundingBox, ShapefileCrs};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use xt_error::{DataError, Error, ErrorKind, GeoError, Result, ResultExt};

pub(super) fn corrupt(message: &str) -> Error {
    Error::new(ErrorKind::Data(DataError::Corrupt), message)
}

pub fn read_shapefile_bounds(path: &Path, crs: ShapefileCrs) -> Result<BoundingBox> {
    if !path
        .extension()
        .is_some_and(|value| value.eq_ignore_ascii_case("shp"))
    {
        return Err(crate::domain::invalid("Spatial extent file must have a .shp extension"));
    }
    let mut file = File::open(path).with_context(|| {
        format!(
            "Open spatial extent file {}",
            crate::services::storage::paths::display(path)
        )
    })?;
    let size = file.metadata().context("Read SHP file size")?.len();
    let mut header = [0u8; 100];
    file.read_exact(&mut header)
        .map_err(|_| corrupt("Incomplete SHP header"))?;
    let be = |offset| i32::from_be_bytes(header[offset..offset + 4].try_into().expect("Fixed length"));
    let le = |offset| i32::from_le_bytes(header[offset..offset + 4].try_into().expect("Fixed length"));
    if be(0) != 9994 || le(28) != 1000 || be(24) < 50 || (be(24) as u64) * 2 != size {
        return Err(corrupt("Invalid SHP signature, version, or declared length"));
    }
    if size == 100 || le(32) == 0 {
        return Err(Error::new(
            ErrorKind::Data(DataError::Empty),
            "SHP contains no valid spatial extent",
        ));
    }
    if !matches!(
        le(32),
        1 | 3 | 5 | 8 | 11 | 13 | 15 | 18 | 21 | 23 | 25 | 28 | 31
    ) {
        return Err(corrupt("Unknown SHP geometry type"));
    }
    if matches!(crs, ShapefileCrs::CheckPrj) {
        check_prj(path)?;
    }
    let number =
        |offset| f64::from_le_bytes(header[offset..offset + 8].try_into().expect("Fixed length"));
    BoundingBox::new(number(36), number(44), number(52), number(60))
        .map_err(|error| error.context("SHP bounds must be a nonzero-area WGS84 extent"))
}

pub(super) fn check_prj(path: &Path) -> Result<()> {
    let lower = path.with_extension("prj");
    let upper = path.with_extension("PRJ");
    let prj = if lower.try_exists().context("Check PRJ file")? {
        lower
    } else {
        upper
    };
    let file = match File::open(prj) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(Error::new(
                ErrorKind::Geo(GeoError::MissingCrs),
                "Missing .prj; use --assume-wgs84 only after confirming WGS84 coordinates",
            ));
        }
        Err(error) => return Err(error.into()),
    };
    let mut text = String::new();
    file.take(65537)
        .read_to_string(&mut text)
        .context("Read PRJ coordinate system")?;
    let wkt: String = text
        .chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(char::to_uppercase)
        .collect();
    let known_geographic =
        wkt.starts_with("GEOGCS[\"GCS_WGS_1984\",") || wkt.starts_with("GEOGCS[\"WGS84\",");
    let degree_units = wkt.contains("UNIT[\"DEGREE\",0.0174532925199433")
        || wkt.contains("UNIT[\"DEGREE\",0.017453292519943295");
    if text.len() > 65536
        || !known_geographic
        || !degree_units
        || !(wkt.contains("DATUM[\"D_WGS_1984\",") || wkt.contains("DATUM[\"WGS_1984\","))
    {
        return Err(Error::new(
            ErrorKind::Geo(GeoError::InvalidCrs),
            "Unsupported WGS84 coordinate system; reproject to EPSG:4326 or explicitly use --assume-wgs84 for equivalent WKT",
        ));
    }
    Ok(())
}
