use crate::error::{invalid, numerical};
use crate::gsl::integration::gsl_integration_qags;
use crate::gsl::{GSL_SUCCESS, GslFunction};
use crate::lai_path::constants::EPSREL;
use crate::lai_path::model::EllipseParam;
use crate::lai_path::solver::solve;
use crate::lai_path::validation::{finish, gap_check, geometry};
use std::cell::Cell;
use xt_error::Result;

pub(super) fn ellipse_bias(lambda: f64, param: &EllipseParam) -> f64 {
    if lambda == 0.0 {
        return 1.0 - param.gap;
    }
    let f = GslFunction::new(
        |theta, lambda: f64| (-lambda * theta.sin()).exp() * theta.sin(),
        lambda,
    );
    let (mut value, mut error) = (0.0, 0.0);
    let status = gsl_integration_qags(
        &f,
        0.0,
        std::f64::consts::FRAC_PI_2,
        0.0,
        EPSREL,
        1000,
        &mut value,
        &mut error,
    );
    if status != GSL_SUCCESS {
        param.failure.set(Some(status));
        return f64::NAN;
    }
    value - param.gap
}

pub fn calc_lai_ellipse(gap_fraction: f64, zenith: f64, g: f64) -> Result<f64> {
    gap_check(gap_fraction)?;
    let scale = geometry(zenith, g)?;
    if gap_fraction == 1.0 {
        return Ok(0.0);
    }
    let param = EllipseParam {
        gap: gap_fraction,
        failure: Cell::new(None),
    };
    let root = solve(ellipse_bias, &param, -gap_fraction.ln());
    if let Some(code) = param.failure.get() {
        return Err(numerical(code, "Ellipsoidal gap fraction integration"));
    }
    finish(root? * std::f64::consts::FRAC_PI_4 * scale)
}

pub fn calc_laie2lai_ellipse(eff_lai: f64, zenith: f64, g: f64) -> Result<f64> {
    if !eff_lai.is_finite() || eff_lai < 0.0 {
        return Err(invalid("Effective LAI must be finite and nonnegative"));
    }
    let scale = geometry(zenith, g)?;
    calc_lai_ellipse((-eff_lai / scale).exp(), zenith, g)
}
