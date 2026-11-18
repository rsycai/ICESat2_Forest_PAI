use crate::gsl::consts::{GSL_CONTINUE, GSL_EBADTOL, GSL_EINVAL, GSL_SUCCESS};
use crate::gsl_error;

pub fn gsl_root_test_interval(x_lower: f64, x_upper: f64, epsabs: f64, epsrel: f64) -> i32 {
    if !x_lower.is_finite() || !x_upper.is_finite() || !epsabs.is_finite() || !epsrel.is_finite() {
        return GSL_EINVAL;
    }
    let abs_lower = x_lower.abs();
    let abs_upper = x_upper.abs();

    if epsrel < 0.0 {
        gsl_error!("relative tolerance is negative", GSL_EBADTOL);
    }

    if epsabs < 0.0 {
        gsl_error!("absolute tolerance is negative", GSL_EBADTOL);
    }

    if x_lower > x_upper {
        gsl_error!("lower bound larger than upper bound", GSL_EINVAL);
    }

    let min_abs = if (x_lower > 0.0 && x_upper > 0.0) || (x_lower < 0.0 && x_upper < 0.0) {
        abs_lower.min(abs_upper)
    } else {
        0.0
    };

    let tolerance: f64 = epsabs + epsrel * min_abs;

    if (x_upper - x_lower).abs() < tolerance {
        return GSL_SUCCESS;
    }

    GSL_CONTINUE
}

pub fn gsl_root_test_delta(x1: f64, x0: f64, epsabs: f64, epsrel: f64) -> i32 {
    if !x1.is_finite() || !x0.is_finite() || !epsabs.is_finite() || !epsrel.is_finite() {
        return GSL_EINVAL;
    }
    let tolerance = epsabs + epsrel * x1.abs();

    if epsrel < 0.0 {
        gsl_error!("relative tolerance is negative", GSL_EBADTOL);
    }

    if epsabs < 0.0 {
        gsl_error!("absolute tolerance is negative", GSL_EBADTOL);
    }

    if (x1 - x0).abs() < tolerance || x1 == x0 {
        return GSL_SUCCESS;
    }

    GSL_CONTINUE
}

pub fn gsl_root_test_residual(f: f64, epsabs: f64) -> i32 {
    if !f.is_finite() || !epsabs.is_finite() {
        return GSL_EINVAL;
    }
    if epsabs < 0.0 {
        gsl_error!("absolute tolerance is negative", GSL_EBADTOL);
    }

    if f.abs() < epsabs {
        return GSL_SUCCESS;
    }

    GSL_CONTINUE
}
