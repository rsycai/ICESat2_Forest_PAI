use std::any::Any;
use xt_error::Result;

#[doc(hidden)]
pub struct Registration {
    pub prefix: &'static str,
    pub section: &'static str,
    pub name: &'static str,
    pub load: fn() -> Result<Box<dyn Any + Send + Sync>>,
}

inventory::collect!(Registration);
