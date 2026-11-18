use crate::gsl::{
    GSL_EBADFUNC, GSL_EBADTOL, GSL_EDIVERGE, GSL_EINVAL, GSL_EMAXITER, GSL_EROUND, GSL_ESING,
    GSL_EZERODIV,
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericalError {
    pub code: i32,
    pub operation: &'static str,
}

impl fmt::Display for NumericalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self.code {
            GSL_EBADFUNC => "Function value or derivative is not finite",
            GSL_EBADTOL => "Tolerance is invalid or unattainable",
            GSL_EMAXITER => "Iteration limit reached without convergence",
            GSL_EROUND => "Floating-point roundoff prevents further accuracy improvement",
            GSL_ESING => "Unhandled singularity in integration interval",
            GSL_EDIVERGE => "Integral diverges or converges too slowly",
            GSL_EZERODIV => "Zero derivative during root finding",
            GSL_EINVAL => "Invalid numerical parameters or root bracket",
            _ => "Numerical computation failed",
        };
        write!(
            f,
            "{}: {} (GSL status {})",
            self.operation, reason, self.code
        )
    }
}

impl std::error::Error for NumericalError {}
