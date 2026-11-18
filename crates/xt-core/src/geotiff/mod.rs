//!
//! ```no_run
//! use xt_core::geotiff::GeoTiffCache;
//! # fn main() -> xt_error::Result<()> {
//! let cache = GeoTiffCache::default();
//! let raster = cache.open("landcover.tif")?;
//! let class = raster.sample::<u8>(116.4074, 39.9042)?;
//! let classes = raster.sample_many::<u8>(&[(116.4074, 39.9042), (0.0, 0.0)])?;
//! # Ok(()) }
//! ```
mod cache;
mod projection;
mod raw;
mod sampler;

pub use cache::GeoTiffCache;
pub use sampler::GeoTiffSampler;
