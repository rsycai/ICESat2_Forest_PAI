use crate::gsl::consts::{GSL_DBL_EPSILON, GSL_DBL_MIN};

pub fn subinterval_too_small(a1: f64, a2: f64, b2: f64) -> i32 {
    let e = GSL_DBL_EPSILON;
    let u = GSL_DBL_MIN;

    let tmp = (1.0 + 100.0 * e) * (a2.abs() + 1000.0 * u);

    let status = a1.abs() <= tmp && b2.abs() <= tmp;

    status as i32
}

pub fn gsl_finite(d: f64) -> bool {
    d.is_finite()
}
