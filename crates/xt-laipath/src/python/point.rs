use crate::python::error::python_error;
use pyo3::prelude::{PyResult, Python, pyclass, pymethods};

#[pyclass(name = "Point", module = "xt_laipath", frozen)]
pub(super) struct PyPoint {
    pub(super) inner: crate::Point,
}

#[pymethods]
impl PyPoint {
    #[new]
    pub(super) fn new(x: f64, y: f64) -> PyResult<Self> {
        crate::Point::new(x, y)
            .map(|inner| Self { inner })
            .map_err(python_error)
    }

    #[getter]
    pub(super) fn x(&self) -> f64 {
        self.inner.x
    }

    #[getter]
    pub(super) fn y(&self) -> f64 {
        self.inner.y
    }

    pub(super) fn euclidean(&self, other: &Self) -> PyResult<f64> {
        self.inner.euclidean(&other.inner).map_err(python_error)
    }

    pub(super) fn haversine(&self, other: &Self) -> PyResult<f64> {
        self.inner.haversine(&other.inner).map_err(python_error)
    }

    pub(super) fn vincenty(&self, py: Python<'_>, other: &Self) -> PyResult<f64> {
        let (left, right) = (self.inner, other.inner);
        py.detach(move || left.vincenty(&right))
            .map_err(python_error)
    }

    pub(super) fn __repr__(&self) -> String {
        format!("Point(x={:?}, y={:?})", self.inner.x, self.inner.y)
    }
}
