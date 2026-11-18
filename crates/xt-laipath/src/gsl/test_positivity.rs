use crate::gsl::consts::GSL_DBL_EPSILON;

#[inline]
pub fn test_positivity(result: f64, resabs: f64) -> i32 {
    (result.abs() >= (1.0 - 50.0 * GSL_DBL_EPSILON) * resabs) as i32
}
