use crate::gsl::consts::{GSL_EBADFUNC, GSL_EZERODIV, GSL_SUCCESS};
use crate::gsl::function::GslFunctionFdf;
use crate::gsl::roots::GslRootFdfsolverType;
use crate::gsl::roots::state::NewtonState;
use crate::gsl::util::gsl_finite;
use crate::{gsl_error, gsl_fn_fdf_eval_df, gsl_fn_fdf_eval_f, gsl_fn_fdf_eval_f_df};

pub(super) fn newton_init<T: Clone>(
    state: &mut NewtonState,
    fdf: &GslFunctionFdf<T>,
    root: &mut f64,
) -> i32 {
    let x = *root;

    state.f = gsl_fn_fdf_eval_f!(fdf.clone(), x);
    state.df = gsl_fn_fdf_eval_df!(fdf.clone(), x);

    if !state.f.is_finite() || !state.df.is_finite() {
        gsl_error!("Initial function value or derivative is not finite", GSL_EBADFUNC);
    }
    GSL_SUCCESS
}

pub(super) fn newton_iterate<T: Clone>(
    state: &mut NewtonState,
    fdf: &GslFunctionFdf<T>,
    root: &mut f64,
) -> i32 {
    if state.df == 0.0 {
        gsl_error!("derivative is zero", GSL_EZERODIV);
    }

    let root_new = *root - (state.f / state.df);
    *root = root_new;

    let (mut f_new, mut df_new): (f64, f64) = (0., 0.);

    gsl_fn_fdf_eval_f_df!(fdf.clone(), root_new, &mut f_new, &mut df_new);

    state.f = f_new;
    state.df = df_new;

    if !gsl_finite(f_new) {
        gsl_error!("function value is not finite", GSL_EBADFUNC);
    }

    if !gsl_finite(df_new) {
        gsl_error!("derivative value is not finite", GSL_EBADFUNC);
    }

    GSL_SUCCESS
}

pub fn new_newton<T: Clone>() -> GslRootFdfsolverType<T> {
    GslRootFdfsolverType {
        name: "newton",
        state: crate::gsl::roots::state::DerivativeState::Newton(NewtonState::default()),
        parameter: std::marker::PhantomData,
    }
}
