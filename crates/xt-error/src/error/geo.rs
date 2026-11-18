#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GeoError {
    MissingCrs,
    InvalidCrs,
    CrsMismatch,
    CoordinateTransform,
    InvalidExtent,
    NoOverlap,
    InvalidResolution,
    GridMismatch,
    InvalidBand,
    MissingNoData,
    NoValidPixels,
    InvalidGeometry,
    Topology,
    MissingDriver,
}
