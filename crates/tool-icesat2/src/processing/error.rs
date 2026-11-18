use xt_error::{DataError, Error, ErrorKind};

pub(super) fn bad(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Data(DataError::SchemaMismatch), message)
}

pub(super) fn h5(error: hdf5_pure::Error) -> Error {
    bad("Invalid HDF5 structure or data").with_source(error)
}
