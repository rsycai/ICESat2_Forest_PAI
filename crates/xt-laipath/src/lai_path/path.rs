use crate::error::numerical;
use crate::gsl::integration::gsl_integration_qagp;
use crate::gsl::{GSL_SUCCESS, GslFunction};
use crate::histogram::Histogram;
use crate::lai_path::constants::EPSREL;
use crate::lai_path::model::PathParam;
use crate::lai_path::solver::solve;
use crate::lai_path::validation::{finish, gap_check, geometry};
use std::cell::Cell;
use xt_error::Result;

pub(super) fn path_bias(lambda: f64, param: &PathParam<'_>) -> f64 {
    if lambda == 0.0 {
        return 1.0 - param.gap;
    }
    let args = (lambda, param.histogram);
    let f = GslFunction::new(
        |x, p: (f64, &Histogram)| (-p.0 * x).exp() * p.1.density(x),
        args,
    );
    let (mut value, mut error) = (0.0, 0.0);
    let limit = param.histogram.range().len().saturating_mul(2).max(1000);
    let status = gsl_integration_qagp(
        &f,
        param.histogram.range(),
        0.0,
        EPSREL,
        limit,
        &mut value,
        &mut error,
    );
    if status != GSL_SUCCESS {
        param.failure.set(Some(status));
        return f64::NAN;
    }
    value - param.gap
}

pub fn calc_lai_path(histogram: &Histogram, gap_fraction: f64, zenith: f64, g: f64) -> Result<f64> {
    gap_check(gap_fraction)?;
    let scale = geometry(zenith, g)?;
    if gap_fraction == 1.0 {
        return Ok(0.0);
    }
    let param = PathParam {
        histogram,
        gap: gap_fraction,
        failure: Cell::new(None),
    };
    let root = solve(path_bias, &param, -gap_fraction.ln());
    if let Some(code) = param.failure.get() {
        return Err(numerical(code, "Path gap fraction integration"));
    }
    let root = root?;
    let mean: f64 = histogram
        .bin()
        .iter()
        .zip(histogram.range().windows(2))
        .map(|(density, r)| density * (r[1] - r[0]) * (r[0] * 0.5 + r[1] * 0.5))
        .sum();
    finish(root * mean * scale)
}
