use crate::gsl::GslFunction;
use crate::gsl::consts::{GSL_EINVAL, GSL_SUCCESS};
use crate::gsl::roots::GslRootFsolverType;
use crate::gsl::roots::state::BisectionState;
use crate::{gsl_error, safe_func_call};

pub(super) fn bisection_init<T: Clone>(
    state: &mut BisectionState,
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

pub(super) fn bisection_iterate<T: Clone>(
    state: &mut BisectionState,
    f: &GslFunction<T>,
    root: &mut f64,
    x_lower: &mut f64,
    x_upper: &mut f64,
) -> i32 {
    let x_left = *x_lower;
    let x_right = *x_upper;

    let f_lower = state.f_lower;
    let f_upper = state.f_upper;

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

    let x_bisect = (x_left + x_right) / 2.0;

    let mut f_bisect: f64 = 0.;
    safe_func_call!(f.clone(), x_bisect, &mut f_bisect);

    if f_bisect == 0.0 {
        *root = x_bisect;
        *x_lower = x_bisect;
        *x_upper = x_bisect;
        return GSL_SUCCESS;
    }

    // Discard the half of the interval which doesn't contain the root.
    if (f_lower > 0.0 && f_bisect < 0.0) || (f_lower < 0.0 && f_bisect > 0.0) {
        *root = 0.5 * (x_left + x_bisect);
        *x_upper = x_bisect;
        state.f_upper = f_bisect;
    } else {
        *root = 0.5 * (x_bisect + x_right);
        *x_lower = x_bisect;
        state.f_lower = f_bisect;
    }

    GSL_SUCCESS
}

pub fn new_bisection<T: Clone>() -> GslRootFsolverType<T> {
    GslRootFsolverType {
        name: "bisection",
        state: crate::gsl::roots::state::BracketState::Bisection(BisectionState::default()),
        parameter: std::marker::PhantomData,
    }
}
