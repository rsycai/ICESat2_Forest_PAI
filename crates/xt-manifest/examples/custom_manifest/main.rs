mod execution;
mod model;
use clap::Parser;

fn main() -> std::process::ExitCode {
    let args = model::Cli::parse();
    match xt_core::logging::LogConfig::from_env("xt-manifest-example") {
        Ok(config) => xt_core::logging::run_with_logging(&config, || execution::run(args)),
        Err(error) => xt_core::exit_result(Err::<(), _>(error)),
    }
}
