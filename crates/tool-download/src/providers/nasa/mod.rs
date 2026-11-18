pub mod cmr;
pub mod earthdata;
pub mod gedi;
pub mod icesat2;
mod validation;
pub use gedi::GediSource;
pub use icesat2::Icesat2PairSource;
pub use icesat2::Icesat2Source;
pub(crate) use validation::validate_definition;

pub(crate) mod metadata;

pub mod storage;
