use crate::python::error::python_error;
use pyo3::prelude::{PyResult, pyclass, pymethods};

#[pyclass(name = "Histogram", module = "xt_laipath", frozen)]
pub(super) struct PyHistogram {
    pub(super) inner: crate::Histogram,
}

#[pymethods]
impl PyHistogram {
    #[new]
    pub(super) fn new(bin: Vec<f64>, range: Vec<f64>) -> PyResult<Self> {
        crate::Histogram::new(bin, range)
            .map(|inner| Self { inner })
            .map_err(python_error)
    }

    #[getter]
    pub(super) fn bin(&self) -> Vec<f64> {
        self.inner.bin().to_vec()
    }

    #[getter]
    pub(super) fn range(&self) -> Vec<f64> {
        self.inner.range().to_vec()
    }

    pub(super) fn probability(&self, value: f64) -> PyResult<f64> {
        self.inner.probability(value).map_err(python_error)
    }

    pub(super) fn __repr__(&self) -> String {
        format!(
            "Histogram(bin={:?}, range={:?})",
            self.inner.bin(),
            self.inner.range()
        )
    }
}
