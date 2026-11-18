use xt_error::{Error, ErrorKind};

pub(super) fn configuration_error(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Configuration, message)
}
