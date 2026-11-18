use crate::error::model::NumericalError;
use crate::gsl::GSL_SUCCESS;
use xt_error::{DataError, Error, ErrorKind};

pub(crate) fn invalid(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::InvalidArgument, message)
}

pub(crate) fn numerical(code: i32, operation: &'static str) -> Error {
    let source = NumericalError { code, operation };
    Error::new(ErrorKind::Data(DataError::InvalidValue), source.to_string()).with_source(source)
}

pub(crate) fn check(code: i32, operation: &'static str) -> crate::Result<()> {
    if code == GSL_SUCCESS {
        Ok(())
    } else {
        Err(numerical(code, operation))
    }
}
