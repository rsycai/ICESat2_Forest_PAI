use crate::gsl::GslFunction;
use crate::gsl::consts::{GSL_EINVAL, GSL_SUCCESS};
use crate::gsl::roots::GslRootFsolverType;
use crate::gsl::roots::state::FalseposState;
use crate::{gsl_error, safe_func_call};

pub(super) fn falsepos_init<T: Clone>(
    state: &mut FalseposState,
    f: &GslFunction<T>,
    root: &mut f64,
    x_lower: f64,
    x_upper: f64,
) -> i32 {
    let mut f_lower: f64 = 0.0;
    let mut f_upper: f64 = 0.0;

    *root = 0.5 * (x_lower + x_upper);

    safe_func_call!(f.clone(), x_lower, &mut f_lower);
    safe_func_call!(f.clone(), x_upper, &mut f_upper);

    state.f_lower = f_lower;
    state.f_upper = f_upper;

    if (f_lower < 0.0 && f_upper < 0.0) || (f_lower > 0.0 && f_upper > 0.0) {
        gsl_error!("endpoints do not straddle y=0", GSL_EINVAL);
    }

    GSL_SUCCESS
}

pub(super) fn falsepos_iterate<T: Clone>(
    state: &mut FalseposState,
    f: &GslFunction<T>,
    root: &mut f64,
    x_lower: &mut f64,
    x_upper: &mut f64,
) -> i32 {
    let x_left = *x_lower;
    let x_right = *x_upper;

    let f_lower = state.f_lower;
    let f_upper = state.f_upper;

    let mut f_linear: f64 = 0.0;

    if f_lower == 0.0 {
        *root = x_left;
        *x_upper = x_left;
        return GSL_SUCCESS;
    }

    if f_upper == 0.0 {
        *root = x_right;
        *x_lower = x_right;
        return GSL_SUCCESS;
    }

    let x_linear: f64 = x_right - (f_upper * (x_left - x_right) / (f_lower - f_upper));

    safe_func_call!(f.clone(), x_linear, &mut f_linear);

    if f_linear == 0.0 {
        *root = x_linear;
        *x_lower = x_linear;
        *x_upper = x_linear;
        return GSL_SUCCESS;
    }

    let w: f64;

    if (f_lower > 0.0 && f_linear < 0.0) || (f_lower < 0.0 && f_linear > 0.0) {
        *root = x_linear;
        *x_upper = x_linear;
        state.f_upper = f_linear;
        w = x_linear - x_left;
    } else {
        *root = x_linear;
        *x_lower = x_linear;
        state.f_lower = f_linear;
        w = x_right - x_linear;
    }

    if w < 0.5 * (x_right - x_left) {
        return GSL_SUCCESS;
    }

    let x_bisect = 0.5 * (x_left + x_right);
    let mut f_bisect: f64 = 0.0;

    safe_func_call!(f.clone(), x_bisect, &mut f_bisect);

    if (f_lower > 0.0 && f_bisect < 0.0) || (f_lower < 0.0 && f_bisect > 0.0) {
        *x_upper = x_bisect;
        state.f_upper = f_bisect;
        if *root > x_bisect {
            *root = 0.5 * (x_left + x_bisect);
        }
    } else {
        *x_lower = x_bisect;
        state.f_lower = f_bisect;
        if *root < x_bisect {
            *root = 0.5 * (x_bisect + x_right);
        }
    }

    GSL_SUCCESS
}

pub fn new_falsepos<T: Clone>() -> GslRootFsolverType<T> {
    GslRootFsolverType {
        name: "falsepos",
        state: crate::gsl::roots::state::BracketState::Falsepos(FalseposState::default()),
        parameter: std::marker::PhantomData,
    }
}
