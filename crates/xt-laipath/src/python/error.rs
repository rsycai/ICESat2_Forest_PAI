use pyo3::create_exception;
use pyo3::exceptions::{PyArithmeticError, PyRuntimeError, PyValueError};
use pyo3::prelude::{PyErr, PyResult, Python};
use pyo3::types::PyAnyMethods;
use std::error::Error as _;

create_exception!(xt_laipath, NumericalError, PyArithmeticError);

pub(super) fn python_error(error: crate::Error) -> PyErr {
    if let Some(source) = error
        .source()
        .and_then(|s| s.downcast_ref::<crate::NumericalError>())
    {
        let exception = NumericalError::new_err(error.to_string());
        let attributes = Python::attach(|py| -> PyResult<()> {
            exception.value(py).setattr("code", source.code)?;
            exception.value(py).setattr("operation", source.operation)?;
            Ok(())
        });
        return attributes.err().unwrap_or(exception);
    }
    match error.kind() {
        crate::ErrorKind::InvalidArgument => PyValueError::new_err(error.to_string()),
        _ => PyRuntimeError::new_err(error.to_string()),
    }
}
