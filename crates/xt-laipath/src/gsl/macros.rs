#[macro_export]
macro_rules! gsl_max {
    ($a:expr,$b:expr) => {{
        let (a, b) = ($a, $b);
        if a > b { a } else { b }
    }};
}
#[macro_export]
macro_rules! gsl_max_dbl {
    ($a:expr,$b:expr) => {
        $crate::gsl_max!($a, $b)
    };
}
#[macro_export]
macro_rules! gsl_min {
    ($a:expr,$b:expr) => {{
        let (a, b) = ($a, $b);
        if a < b { a } else { b }
    }};
}
#[macro_export]
macro_rules! gsl_fn_eval {
    ($f:expr,$x:expr) => {{
        let f = $f;
        (f.function)($x, f.param)
    }};
}
#[macro_export]
macro_rules! gsl_error {
    ($reason:expr,$code:expr) => {{
        let _ = $reason;
        return $code;
    }};
}
#[macro_export]
macro_rules! safe_func_call {
    ($f:expr,$x:expr,$output:expr) => {{
        let output = $output;
        *output = $crate::gsl_fn_eval!($f, $x);
        if !output.is_finite() {
            $crate::gsl_error!("Function value is not finite", $crate::gsl::GSL_EBADFUNC);
        }
    }};
}
#[macro_export]
macro_rules! gsl_fn_fdf_eval_f {
    ($f:expr,$x:expr) => {{
        let f = $f;
        (f.f)($x, f.params)
    }};
}
#[macro_export]
macro_rules! gsl_fn_fdf_eval_df {
    ($f:expr,$x:expr) => {{
        let f = $f;
        (f.df)($x, f.params)
    }};
}
#[macro_export]
macro_rules! gsl_fn_fdf_eval_f_df {
    ($f:expr,$x:expr,$y:expr,$dy:expr) => {{
        let f = $f;
        (f.fdf)($x, f.params, $y, $dy)
    }};
}
