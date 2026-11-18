mod conversion;
mod model;
pub(crate) use conversion::{check, invalid, numerical};
pub use model::NumericalError;
