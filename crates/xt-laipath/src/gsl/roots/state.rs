use crate::gsl::function::{GslFunction, GslFunctionFdf};
use std::marker::PhantomData;

#[derive(Default)]
pub(super) struct BisectionState {
    pub(super) f_lower: f64,
    pub(super) f_upper: f64,
}

#[derive(Default)]
pub(super) struct BrentState {
    pub(super) a: f64,
    pub(super) b: f64,
    pub(super) c: f64,
    pub(super) d: f64,
    pub(super) e: f64,
    pub(super) fa: f64,
    pub(super) fb: f64,
    pub(super) fc: f64,
}

#[derive(Default)]
pub(super) struct FalseposState {
    pub(super) f_lower: f64,
    pub(super) f_upper: f64,
}

#[derive(Default)]
pub(super) struct NewtonState {
    pub(super) f: f64,
    pub(super) df: f64,
}

#[derive(Default)]
pub(super) struct SecantState {
    pub(super) f: f64,
    pub(super) df: f64,
}

#[derive(Default)]
pub(super) struct SteffensonState {
    pub(super) f: f64,
    pub(super) df: f64,
    pub(super) x: f64,
    pub(super) x_1: f64,
    pub(super) x_2: f64,
    pub(super) count: i32,
}

pub(super) enum BracketState {
    Bisection(BisectionState),
    Brent(BrentState),
    Falsepos(FalseposState),
}

pub(super) enum DerivativeState {
    Newton(NewtonState),
    Secant(SecantState),
    Steffenson(SteffensonState),
}

pub struct GslRootFsolverType<T: Clone> {
    pub(super) name: &'static str,
    pub(super) state: BracketState,
    pub(super) parameter: PhantomData<fn(T)>,
}

pub struct GslRootFsolver<T: Clone> {
    pub(super) r#type: GslRootFsolverType<T>,
    pub(super) function: GslFunction<T>,
    pub(super) root: f64,
    pub(super) x_lower: f64,
    pub(super) x_upper: f64,
    pub(super) initialized: bool,
}

pub struct GslRootFdfsolverType<T: Clone> {
    pub(super) name: &'static str,
    pub(super) state: DerivativeState,
    pub(super) parameter: PhantomData<fn(T)>,
}

pub struct GslRootFdfsolver<T: Clone> {
    pub(super) r#type: GslRootFdfsolverType<T>,
    pub(super) fdf: GslFunctionFdf<T>,
    pub(super) root: f64,
    pub(super) initialized: bool,
}
