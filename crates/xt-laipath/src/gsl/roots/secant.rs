use crate::gsl::consts::{GSL_EBADFUNC, GSL_EZERODIV, GSL_SUCCESS};
use crate::gsl::function::GslFunctionFdf;
use crate::gsl::roots::GslRootFdfsolverType;
use crate::gsl::roots::state::SecantState;
use crate::{gsl_error, gsl_fn_fdf_eval_f, gsl_fn_fdf_eval_f_df};

pub(super) fn secant_init<T: Clone>(
    state: &mut SecantState,
    fdf: &GslFunctionFdf<T>,
    root: &mut f64,
) -> i32 {
    let x = *root;

    gsl_fn_fdf_eval_f_df!(fdf.clone(), x, &mut state.f, &mut state.df);

    if !state.f.is_finite() || !state.df.is_finite() {
        gsl_error!("Initial function value or derivative is not finite", GSL_EBADFUNC);
    }
    GSL_SUCCESS
}

pub(super) fn secant_iterate<T: Clone>(
    state: &mut SecantState,
    fdf: &GslFunctionFdf<T>,
    root: &mut f64,
) -> i32 {
    let x = *root;
    let f = state.f;
    let df = state.df;

    if f == 0.0 {
        return GSL_SUCCESS; // An exact root has been found.
    }

    if df == 0.0 {
        gsl_error!("derivative is zero", GSL_EZERODIV);
    }

    let x_new = x - (f / df);

    let f_new = gsl_fn_fdf_eval_f!(fdf.clone(), x_new);
    let df_new = df * ((f - f_new) / f);

    *root = x_new;
    state.f = f_new;
    state.df = df_new;

    if !f_new.is_finite() {
        gsl_error!("function value is not finite", GSL_EBADFUNC);
    }

    if !df_new.is_finite() {
        gsl_error!("derivative value is not finite", GSL_EBADFUNC);
    }

    GSL_SUCCESS
}

pub fn new_secant<T: Clone>() -> GslRootFdfsolverType<T> {
    GslRootFdfsolverType {
        name: "secant",
        state: crate::gsl::roots::state::DerivativeState::Secant(SecantState::default()),
        parameter: std::marker::PhantomData,
    }
}
