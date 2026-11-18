use crate::gsl::function::GslFunctionFdf;
use crate::gsl::roots::state::{DerivativeState, GslRootFdfsolver, GslRootFdfsolverType};
use crate::gsl::roots::{newton, secant, steffenson};
use crate::gsl::{GSL_EINVAL, GSL_SUCCESS};
impl<T: Clone> GslRootFdfsolver<T> {
    pub fn new(s: GslRootFdfsolverType<T>, f: GslFunctionFdf<T>) -> Self {
        Self {
            r#type: s,
            fdf: f,
            root: 0.0,
            initialized: false,
        }
    }
    pub fn set(&mut self, root: f64) -> i32 {
        self.initialized = false;
        if !(root.is_finite()) {
            return GSL_EINVAL;
        }
        self.root = root;
        let status = match &mut self.r#type.state {
            DerivativeState::Newton(state) => newton::newton_init(state, &self.fdf, &mut self.root),
            DerivativeState::Secant(state) => secant::secant_init(state, &self.fdf, &mut self.root),
            DerivativeState::Steffenson(state) => {
                steffenson::steffenson_init(state, &self.fdf, &mut self.root)
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
            DerivativeState::Newton(state) => {
                newton::newton_iterate(state, &self.fdf, &mut self.root)
            }
            DerivativeState::Secant(state) => {
                secant::secant_iterate(state, &self.fdf, &mut self.root)
            }
            DerivativeState::Steffenson(state) => {
                steffenson::steffenson_iterate(state, &self.fdf, &mut self.root)
            }
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
}
