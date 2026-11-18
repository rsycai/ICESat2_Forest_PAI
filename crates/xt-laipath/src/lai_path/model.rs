use crate::histogram::Histogram;
use std::cell::Cell;

pub(super) struct PathParam<'a> {
    pub(super) histogram: &'a Histogram,
    pub(super) gap: f64,
    pub(super) failure: Cell<Option<i32>>,
}

pub(super) struct EllipseParam {
    pub(super) gap: f64,
    pub(super) failure: Cell<Option<i32>>,
}
