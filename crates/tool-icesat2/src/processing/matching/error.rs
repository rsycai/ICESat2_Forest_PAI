use xt_error::{DataError, Error, ErrorKind};

pub(super) fn invalid(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Data(DataError::Corrupt), message)
}
