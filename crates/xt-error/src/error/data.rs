#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DataError {
    UnsupportedFormat,
    InvalidEncoding,
    Parse,
    Corrupt,
    Decompression,
    MissingField,
    InvalidValue,
    Empty,
    SchemaMismatch,
}
