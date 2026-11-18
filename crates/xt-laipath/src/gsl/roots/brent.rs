use crate::gsl::GslFunction;
use crate::gsl::consts::{GSL_DBL_EPSILON, GSL_EINVAL, GSL_SUCCESS};
use crate::gsl::roots::GslRootFsolverType;
use crate::gsl::roots::state::BrentState;
use crate::{gsl_error, gsl_min, safe_func_call};

pub(super) fn brent_init<T: Clone>(
    state: &mut BrentState,
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

    state.a = x_lower;
    state.fa = f_lower;

    state.b = x_upper;
    state.fb = f_upper;

    state.c = x_upper;
    state.fc = f_upper;

    state.d = x_upper - x_lower;
    state.e = x_upper - x_lower;

    if (f_lower < 0.0 && f_upper < 0.0) || (f_lower > 0.0 && f_upper > 0.0) {
        gsl_error!("endpoints do not straddle y=0", GSL_EINVAL);
    }

    GSL_SUCCESS
}

pub(super) fn brent_iterate<T: Clone>(
    state: &mut BrentState,
    f: &GslFunction<T>,
    root: &mut f64,
    x_lower: &mut f64,
    x_upper: &mut f64,
) -> i32 {
    let mut ac_equal = 0;

    let (mut a, mut b, mut c) = (state.a, state.b, state.c);
    let (mut fa, mut fb, mut fc) = (state.fa, state.fb, state.fc);
    let (mut d, mut e) = (state.d, state.e);

    if (fb < 0.0 && fc < 0.0) || (fb > 0.0 && fc > 0.0) {
        ac_equal = 1;
        c = a;
        fc = fa;
        d = b - a;
        e = b - a;
    }

    if fc.abs() < fb.abs() {
        ac_equal = 1;
        a = b;
        b = c;
        c = a;
        fa = fb;
        fb = fc;
        fc = fa;
    }

    let tol: f64 = 0.5 * GSL_DBL_EPSILON * b.abs();
    let m: f64 = 0.5 * (c - b);

    if fb == 0.0 {
        *root = b;
        *x_lower = b;
        *x_upper = b;

        return GSL_SUCCESS;
    }

    if m.abs() <= tol {
        *root = b;

        if b < c {
            *x_lower = b;
            *x_upper = c;
        } else {
            *x_lower = c;
            *x_upper = b;
        }

        return GSL_SUCCESS;
    }

    if e.abs() < tol || fa.abs() <= fb.abs() {
        d = m; // Fall back to bisection.
        e = m;
    } else {
        let s = fb / fa; // Try inverse quadratic interpolation.
        let (mut p, mut q, r);

        if ac_equal == 1 {
            p = 2.0 * m * s;
            q = 1.0 - s;
        } else {
            q = fa / fc;
            r = fb / fc;
            p = s * (2.0 * m * q * (q - r) - (b - a) * (r - 1.0));
            q = (q - 1.0) * (r - 1.0) * (s - 1.0);
        }

        if p > 0.0 {
            q = -q;
        } else {
            p = -p;
        }

        if 2.0 * p < gsl_min!(3.0 * m * q - tol * q.abs(), (e * q).abs()) {
            e = d;
            d = p / q;
        } else {
            d = m;
            e = m;
        }
    }

    a = b;
    fa = fb;

    if d.abs() > tol {
        b += d;
    } else {
        b += if m > 0.0 { tol } else { -tol };
    }

    safe_func_call!(f.clone(), b, &mut fb);

    state.a = a;
    state.b = b;
    state.c = c;
    state.d = d;
    state.e = e;
    state.fa = fa;
    state.fb = fb;
    state.fc = fc;

    *root = b;

    if (fb < 0.0 && fc < 0.0) || (fb > 0.0 && fc > 0.0) {
        c = a;
    }

    if b < c {
        *x_lower = b;
        *x_upper = c;
    } else {
        *x_lower = c;
        *x_upper = b;
    }

    GSL_SUCCESS
}

pub fn new_brent<T: Clone>() -> GslRootFsolverType<T> {
    GslRootFsolverType {
        name: "brent",
        state: crate::gsl::roots::state::BracketState::Brent(BrentState::default()),
        parameter: std::marker::PhantomData,
    }
}
