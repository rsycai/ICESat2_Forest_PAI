use crate::gsl::consts::{GSL_EBADFUNC, GSL_EZERODIV, GSL_SUCCESS};
use crate::gsl::function::GslFunctionFdf;
use crate::gsl::roots::GslRootFdfsolverType;
use crate::gsl::roots::state::SteffensonState;
use crate::{gsl_error, gsl_fn_fdf_eval_df, gsl_fn_fdf_eval_f, gsl_fn_fdf_eval_f_df};

pub(super) fn steffenson_init<T: Clone>(
    state: &mut SteffensonState,
    fdf: &GslFunctionFdf<T>,
    root: &mut f64,
) -> i32 {
    let x = *root;

    state.f = gsl_fn_fdf_eval_f!(fdf.clone(), x);
    state.df = gsl_fn_fdf_eval_df!(fdf.clone(), x);

    state.x = x;
    state.x_1 = 0.0;
    state.x_2 = 0.0;
    state.count = 1;

    if !state.f.is_finite() || !state.df.is_finite() {
        gsl_error!("Initial function value or derivative is not finite", GSL_EBADFUNC);
    }
    GSL_SUCCESS
}

pub(super) fn steffenson_iterate<T: Clone>(
    state: &mut SteffensonState,
    fdf: &GslFunctionFdf<T>,
    root: &mut f64,
) -> i32 {
    let x_1 = state.x_1;
    let x = state.x;

    if state.df == 0.0 {
        gsl_error!("derivative is zero", GSL_EZERODIV);
    }

    let x_new = x - (state.f / state.df);
    let (mut f_new, mut df_new) = (0., 0.);
    gsl_fn_fdf_eval_f_df!(fdf.clone(), x_new, &mut f_new, &mut df_new);

    state.x_2 = x_1;
    state.x_1 = x;
    state.x = x_new;
    state.f = f_new;
    state.df = df_new;

    if !f_new.is_finite() {
        gsl_error!("function value is not finite", GSL_EBADFUNC);
    }

    if state.count < 3 {
        *root = x_new;
        state.count += 1;
    } else {
        let u = x - x_1;
        let v = x_new - 2.0 * x + x_1;

        *root = if v == 0.0 {
            x_new // Avoid division by zero.
        } else {
            x_1 - u * u / v // Use the accelerated estimate.
        };
    }

    if !df_new.is_finite() {
        gsl_error!("derivative value is not finite", GSL_EBADFUNC);
    }

    GSL_SUCCESS
}

pub fn new_steffenson<T: Clone>() -> GslRootFdfsolverType<T> {
    GslRootFdfsolverType {
        name: "steffenson",
        state: crate::gsl::roots::state::DerivativeState::Steffenson(SteffensonState::default()),
        parameter: std::marker::PhantomData,
    }
}
