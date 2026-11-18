mod bisection;
mod brent;
mod convergence;
mod falsepos;
mod fdfsolver;
mod fsolver;
mod newton;
mod secant;
mod steffenson;

pub use crate::gsl::roots::convergence::{
    gsl_root_test_delta, gsl_root_test_interval, gsl_root_test_residual,
};
pub use bisection::new_bisection;
pub use brent::new_brent;
pub use falsepos::new_falsepos;
pub use newton::new_newton;
pub use secant::new_secant;
pub use steffenson::new_steffenson;


mod state;

pub use state::{GslRootFdfsolver, GslRootFdfsolverType, GslRootFsolver, GslRootFsolverType};
