use crate::python::error::python_error;
use crate::python::histogram::PyHistogram;
use pyo3::prelude::{PyResult, Python, pyfunction};

#[pyfunction]
#[pyo3(signature = (data, num_bins, a=0.0, b=1.0))]
pub(super) fn histogram_stats(
    py: Python<'_>,
    data: Vec<f64>,
    num_bins: usize,
    a: f64,
    b: f64,
) -> PyResult<PyHistogram> {
    py.detach(move || crate::histogram_stats(data, num_bins, a, b))
        .map(|inner| PyHistogram { inner })
        .map_err(python_error)
}
