use crate::logging::{LogConfig, init_logging};
use std::{future::Future, process::ExitCode};
use xt_error::Result;

pub fn run_with_logging(config: &LogConfig, operation: impl FnOnce() -> Result<()>) -> ExitCode {
    let _guard = match init_logging(config) {
        Ok(guard) => guard,
        Err(error) => return crate::exit_result(Err::<(), _>(error)),
    };
    tracing::debug!("Logging initialized");
    complete(operation())
}

pub async fn run_with_logging_async<F>(
    config: &LogConfig,
    operation: impl FnOnce() -> F,
) -> ExitCode
where
    F: Future<Output = Result<()>>,
{
    let _guard = match init_logging(config) {
        Ok(guard) => guard,
        Err(error) => return crate::exit_result(Err::<(), _>(error)),
    };
    tracing::debug!("Logging initialized");
    complete(operation().await)
}

fn complete(result: Result<()>) -> ExitCode {
    match result {
        Ok(()) => {
            tracing::info!("Task completed");
            ExitCode::SUCCESS
        }
        Err(error) => {
            tracing::error!(code = error.code(), error = %error, "Task failed");
            ExitCode::FAILURE
        }
    }
}
