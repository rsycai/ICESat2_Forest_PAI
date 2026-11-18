use crate::error::data::DataError;
use crate::error::download::DownloadError;
use crate::error::geo::GeoError;
use crate::error::retry::RetryAdvice;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    InvalidArgument,
    Configuration,
    Io(io::ErrorKind),
    Download(DownloadError),
    Data(DataError),
    Geo(GeoError),
    ResourceExhausted,
    Cancelled,
    Internal,
}

impl ErrorKind {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidArgument => "INVALID_ARGUMENT",
            Self::Configuration => "CONFIGURATION",
            Self::Io(_) => "IO",
            Self::Download(kind) => match kind {
                DownloadError::Timeout => "DOWNLOAD_TIMEOUT",
                DownloadError::Connection => "DOWNLOAD_CONNECTION",
                DownloadError::Dns => "DOWNLOAD_DNS",
                DownloadError::Tls => "DOWNLOAD_TLS",
                DownloadError::HttpStatus(_) => "DOWNLOAD_HTTP_STATUS",
                DownloadError::Interrupted => "DOWNLOAD_INTERRUPTED",
                DownloadError::Incomplete => "DOWNLOAD_INCOMPLETE",
                DownloadError::ChecksumMismatch => "DOWNLOAD_CHECKSUM_MISMATCH",
                DownloadError::ResumeUnsupported => "DOWNLOAD_RESUME_UNSUPPORTED",
            },
            Self::Data(kind) => match kind {
                DataError::UnsupportedFormat => "DATA_UNSUPPORTED_FORMAT",
                DataError::InvalidEncoding => "DATA_INVALID_ENCODING",
                DataError::Parse => "DATA_PARSE",
                DataError::Corrupt => "DATA_CORRUPT",
                DataError::Decompression => "DATA_DECOMPRESSION",
                DataError::MissingField => "DATA_MISSING_FIELD",
                DataError::InvalidValue => "DATA_INVALID_VALUE",
                DataError::Empty => "DATA_EMPTY",
                DataError::SchemaMismatch => "DATA_SCHEMA_MISMATCH",
            },
            Self::Geo(kind) => match kind {
                GeoError::MissingCrs => "GEO_MISSING_CRS",
                GeoError::InvalidCrs => "GEO_INVALID_CRS",
                GeoError::CrsMismatch => "GEO_CRS_MISMATCH",
                GeoError::CoordinateTransform => "GEO_COORDINATE_TRANSFORM",
                GeoError::InvalidExtent => "GEO_INVALID_EXTENT",
                GeoError::NoOverlap => "GEO_NO_OVERLAP",
                GeoError::InvalidResolution => "GEO_INVALID_RESOLUTION",
                GeoError::GridMismatch => "GEO_GRID_MISMATCH",
                GeoError::InvalidBand => "GEO_INVALID_BAND",
                GeoError::MissingNoData => "GEO_MISSING_NODATA",
                GeoError::NoValidPixels => "GEO_NO_VALID_PIXELS",
                GeoError::InvalidGeometry => "GEO_INVALID_GEOMETRY",
                GeoError::Topology => "GEO_TOPOLOGY",
                GeoError::MissingDriver => "GEO_MISSING_DRIVER",
            },
            Self::ResourceExhausted => "RESOURCE_EXHAUSTED",
            Self::Cancelled => "CANCELLED",
            Self::Internal => "INTERNAL",
        }
    }

    pub const fn retry_advice(self) -> RetryAdvice {
        match self {
            Self::Download(
                DownloadError::Timeout
                | DownloadError::Connection
                | DownloadError::Interrupted
                | DownloadError::Incomplete
                | DownloadError::HttpStatus(408 | 429 | 500 | 502 | 503 | 504),
            )
            | Self::Io(
                io::ErrorKind::TimedOut
                | io::ErrorKind::Interrupted
                | io::ErrorKind::WouldBlock
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::ConnectionAborted,
            ) => RetryAdvice::Backoff,
            _ => RetryAdvice::DoNotRetry,
        }
    }
}
