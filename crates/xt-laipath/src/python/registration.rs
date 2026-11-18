use crate::python::error::NumericalError;
use crate::python::histogram::PyHistogram;
use crate::python::lai::{calc_eff_lai, calc_lai_ellipse, calc_lai_path, calc_laie2lai_ellipse};
use crate::python::point::PyPoint;
use crate::python::statistics::histogram_stats;
use pyo3::prelude::{Bound, PyModule, PyResult, pymodule, wrap_pyfunction};
use pyo3::types::PyModuleMethods;

#[pymodule]
pub(super) fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyHistogram>()?;
    module.add_class::<PyPoint>()?;
    module.add("NumericalError", module.py().get_type::<NumericalError>())?;
    module.add("__version__", env!("CARGO_PKG_VERSION"))?;
    module.add("LAI_MAX", crate::LAI_MAX)?;
    module.add_function(wrap_pyfunction!(histogram_stats, module)?)?;
    module.add_function(wrap_pyfunction!(calc_eff_lai, module)?)?;
    module.add_function(wrap_pyfunction!(calc_lai_path, module)?)?;
    module.add_function(wrap_pyfunction!(calc_lai_ellipse, module)?)?;
    module.add_function(wrap_pyfunction!(calc_laie2lai_ellipse, module)?)?;
    Ok(())
}
