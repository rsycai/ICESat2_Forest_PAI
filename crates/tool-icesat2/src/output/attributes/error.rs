use xt_error::{DataError, Error, ErrorKind};

pub(super) fn bad(s: impl Into<String>) -> Error {
    Error::new(ErrorKind::Data(DataError::SchemaMismatch), s)
}

pub(super) fn h5(e: hdf5_pure::Error) -> Error {
    bad("Failed to extract ICESat-2 extended attributes").with_source(e)
}
