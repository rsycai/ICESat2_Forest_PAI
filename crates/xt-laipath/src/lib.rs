//!
//!
//! ```
//! use xt_laipath::{histogram_stats, calc_eff_lai, calc_lai_path};
//! let histogram = histogram_stats(vec![1.0, 2.0, 3.0, 4.0], 4, 0.0, 1.0)?;
//! let effective = calc_eff_lai(0.5, 0.0, 0.5)?;
//! let lai = calc_lai_path(&histogram, 0.5, 0.0, 0.5)?;
//! assert!(lai >= effective);
//! # Ok::<(), xt_error::Error>(())
//! ```
#![forbid(unsafe_code)]

mod error;
pub mod gsl;
mod histogram;
mod lai_path;
mod point;
#[cfg(feature = "python")]
mod python;

pub use error::NumericalError;
pub use histogram::{Histogram, histogram_stats};
pub use lai_path::calc_eff_lai as calc_eff_pai;
pub use lai_path::calc_lai_path as calc_pai_path;
pub use lai_path::{calc_eff_lai, calc_lai_ellipse, calc_lai_path, calc_laie2lai_ellipse};
pub use point::Point;
pub use xt_error::{Error, ErrorKind, Result};

mod constants;
pub use constants::LAI_MAX;
