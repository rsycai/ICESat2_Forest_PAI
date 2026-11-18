use crate::error::{check, numerical};
use crate::gsl::roots::{GslRootFsolver, gsl_root_test_interval, new_brent};
use crate::gsl::{GSL_CONTINUE, GSL_EBADFUNC, GSL_EMAXITER, GSL_SUCCESS, GslFunction};
use crate::lai_path::constants::ITERATIONS;
use xt_error::Result;

pub(super) fn solve<P>(callback: fn(f64, &P) -> f64, param: &P, initial: f64) -> Result<f64> {
    let mut upper = initial.max(1.0);
    let mut bracketed = false;
    for _ in 0..ITERATIONS {
        let value = callback(upper, param);
        if !value.is_finite() {
            return Err(numerical(GSL_EBADFUNC, "Construct LAI root bracket"));
        }
        if value <= 0.0 {
            bracketed = true;
            break;
        }
        upper *= 2.0;
        if !upper.is_finite() {
            break;
        }
    }
    if !bracketed {
        return Err(numerical(GSL_EMAXITER, "Construct LAI root bracket"));
    }
    let function = GslFunction::new(callback, param);
    let mut solver = GslRootFsolver::new(new_brent(), function);
    check(solver.set(0.0, upper), "Set LAI root bracket")?;
    for _ in 0..ITERATIONS {
        check(solver.iterate(), "LAI root-finding iteration")?;
        let status = gsl_root_test_interval(solver.x_lower(), solver.x_upper(), 1e-12, 1e-9);
        if status == GSL_SUCCESS {
            return Ok(solver.root());
        }
        if status != GSL_CONTINUE {
            return Err(numerical(status, "LAI convergence check"));
        }
    }
    Err(numerical(GSL_EMAXITER, "LAI root-finding iteration"))
}
