
pub use xt_config::AppConfig;

mod config;
mod diagnostics;
mod execution;
mod formatter;
mod guard;
mod initialization;
mod retention;
mod rotation;
mod timer;
mod writer;
pub use config::LogConfig;
pub(crate) use diagnostics::report_error;
pub use execution::{run_with_logging, run_with_logging_async};
pub use guard::LogGuard;
pub use initialization::init_logging;

