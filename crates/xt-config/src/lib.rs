//!

mod app;
pub mod config_env;
#[doc(hidden)]
pub mod env_load;
pub mod paths;
pub mod registry;

#[doc(hidden)]
pub use app::take_missing_default;
pub use app::{AppConfig, DEFAULT_CONFIG_PATH, ensure_loaded};
pub use config_env::{EnvConfig, ExportReport, init_config};
pub use registry::{ConfigRegistry, get, init_registered};
pub use xt_error::{Error, ErrorKind, Result, ResultExt};
pub use xt_macros::{Config, EnvLoad, SubConfig};

#[doc(hidden)]
pub use inventory as __inventory;
