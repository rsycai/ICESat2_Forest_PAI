use crate::gsl::consts::{
    GSL_DBL_EPSILON, GSL_DBL_MIN, GSL_EDIVERGE, GSL_EFAILED, GSL_EMAXITER, GSL_EROUND, GSL_ESING,
    GSL_SUCCESS,
};
use crate::gsl_error;

#[inline]
pub fn rescale_error(err: f64, result_abs: f64, result_asc: f64) -> f64 {
    let mut err = err.abs();

    if result_asc != 0.0 && err != 0.0 {
        let scale = (200.0 * err / result_asc).powf(1.5);

        if scale < 1.0 {
            err = result_asc * scale;
        } else {
            err = result_asc;
        }
    }

    if result_abs > GSL_DBL_MIN / (50.0 * GSL_DBL_EPSILON) {
        let min_err = 50.0 * GSL_DBL_EPSILON * result_abs;

        if min_err > err {
            err = min_err;
        }
    }

    err
}

#[inline]
pub fn fmt_err(mut error_type: i32) -> i32 {
    if error_type > 2 {
        error_type -= 1;
    }
    match error_type {
        0 => GSL_SUCCESS,
        1 => gsl_error!("number of iterations was insufficient", GSL_EMAXITER),
        2 => gsl_error!(
            "cannot reach tolerance because of roundoff error",
            GSL_EROUND
        ),
        3 => gsl_error!(
            "bad integrand behavior found in the integration interval",
            GSL_ESING
        ),
        4 => gsl_error!(
            "roundoff error detected in the extrapolation table",
            GSL_EROUND
        ),
        5 => gsl_error!("integral is divergent, or slowly convergent", GSL_EDIVERGE),
        _ => gsl_error!("could not integrate function", GSL_EFAILED),
    }
}
