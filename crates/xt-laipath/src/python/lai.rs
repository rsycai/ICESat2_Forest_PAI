use crate::python::error::python_error;
use crate::python::histogram::PyHistogram;
use pyo3::prelude::{PyResult, Python, pyfunction};

#[pyfunction]
#[pyo3(signature = (gap_fraction, zenith=0.0, g=0.5))]
pub(super) fn calc_eff_lai(gap_fraction: f64, zenith: f64, g: f64) -> PyResult<f64> {
    crate::calc_eff_lai(gap_fraction, zenith, g).map_err(python_error)
}

#[pyfunction]
#[pyo3(signature = (histogram, gap_fraction, zenith=0.0, g=0.5))]
pub(super) fn calc_lai_path(
    py: Python<'_>,
    histogram: &PyHistogram,
    gap_fraction: f64,
    zenith: f64,
    g: f64,
) -> PyResult<f64> {
    let histogram = histogram.inner.clone();
    py.detach(move || crate::calc_lai_path(&histogram, gap_fraction, zenith, g))
        .map_err(python_error)
}

#[pyfunction]
#[pyo3(signature = (gap_fraction, zenith=0.0, g=0.5))]
pub(super) fn calc_lai_ellipse(
    py: Python<'_>,
    gap_fraction: f64,
    zenith: f64,
    g: f64,
) -> PyResult<f64> {
    py.detach(move || crate::calc_lai_ellipse(gap_fraction, zenith, g))
        .map_err(python_error)
}

#[pyfunction]
#[pyo3(signature = (eff_lai, zenith=0.0, g=0.5))]
pub(super) fn calc_laie2lai_ellipse(
    py: Python<'_>,
    eff_lai: f64,
    zenith: f64,
    g: f64,
) -> PyResult<f64> {
    py.detach(move || crate::calc_laie2lai_ellipse(eff_lai, zenith, g))
        .map_err(python_error)
}
