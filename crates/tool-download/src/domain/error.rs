use xt_error::{Error, ErrorKind};

pub(crate) fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidArgument, message)
}
