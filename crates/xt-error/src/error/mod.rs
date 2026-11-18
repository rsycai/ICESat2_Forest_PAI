//!

mod context;
mod data;
mod download;
mod extension;
mod geo;
mod kind;
mod result;
mod retry;
pub use context::Error;
pub use data::DataError;
pub use download::DownloadError;
pub use extension::ResultExt;
pub use geo::GeoError;
pub use kind::ErrorKind;
pub use result::Result;
pub use retry::RetryAdvice;

