#[derive(Clone, Debug)]
pub struct GslFunction<T>
where
    T: Clone,
{
    pub function: fn(f64, T) -> f64,
    pub param: T,
}

impl<T: Clone> GslFunction<T> {
    pub fn new(function: fn(f64, T) -> f64, param: T) -> GslFunction<T> {
        Self { function, param }
    }
}

#[derive(Clone, Debug)]
pub struct GslFunctionFdf<T: Clone> {
    pub f: fn(f64, T) -> f64,
    pub df: fn(f64, T) -> f64,
    pub fdf: fn(f64, T, &mut f64, &mut f64),
    pub params: T,
}
