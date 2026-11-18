use crate::gsl::function::GslFunction;
use crate::gsl::roots::state::{BracketState, GslRootFsolver, GslRootFsolverType};
use crate::gsl::roots::{bisection, brent, falsepos};
use crate::gsl::{GSL_EINVAL, GSL_SUCCESS};
impl<T: Clone> GslRootFsolver<T> {
    pub fn new(s: GslRootFsolverType<T>, f: GslFunction<T>) -> Self {
        Self {
            r#type: s,
            function: f,
            root: 0.0,
            x_lower: 0.0,
            x_upper: 0.0,
            initialized: false,
        }
    }
    pub fn set(&mut self, x_lower: f64, x_upper: f64) -> i32 {
        self.initialized = false;
        if !(x_lower.is_finite() && x_upper.is_finite() && x_lower <= x_upper) {
            return GSL_EINVAL;
        }
        self.x_lower = x_lower;
        self.x_upper = x_upper;
        let status = match &mut self.r#type.state {
            BracketState::Bisection(state) => {
                bisection::bisection_init(state, &self.function, &mut self.root, x_lower, x_upper)
            }
            BracketState::Brent(state) => {
                brent::brent_init(state, &self.function, &mut self.root, x_lower, x_upper)
            }
            BracketState::Falsepos(state) => {
                falsepos::falsepos_init(state, &self.function, &mut self.root, x_lower, x_upper)
            }
        };
        self.initialized = status == GSL_SUCCESS;
        status
    }
    pub fn iterate(&mut self) -> i32 {
        if !self.initialized {
            return GSL_EINVAL;
        }
        let status = match &mut self.r#type.state {
            BracketState::Bisection(state) => bisection::bisection_iterate(
                state,
                &self.function,
                &mut self.root,
                &mut self.x_lower,
                &mut self.x_upper,
            ),
            BracketState::Brent(state) => brent::brent_iterate(
                state,
                &self.function,
                &mut self.root,
                &mut self.x_lower,
                &mut self.x_upper,
            ),
            BracketState::Falsepos(state) => falsepos::falsepos_iterate(
                state,
                &self.function,
                &mut self.root,
                &mut self.x_lower,
                &mut self.x_upper,
            ),
        };
        if status != GSL_SUCCESS {
            self.initialized = false;
        }
        status
    }
    pub fn name(&self) -> &str {
        self.r#type.name
    }
    pub fn root(&self) -> f64 {
        self.root
    }
    pub fn x_lower(&self) -> f64 {
        self.x_lower
    }
    pub fn x_upper(&self) -> f64 {
        self.x_upper
    }
}
