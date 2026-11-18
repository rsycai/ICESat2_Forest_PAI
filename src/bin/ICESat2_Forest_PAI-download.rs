use clap::Parser;

#[tokio::main]
async fn main() -> std::process::ExitCode {
    let args = tool_download::Cli::parse();
    if let Err(error) = args.load_config() {
        return xt_core::exit_result(Err::<(), _>(error));
    }
    match xt_core::logging::LogConfig::from_env("ICESat2_Forest_PAI-download") {
        Ok(config) => {
            xt_core::logging::run_with_logging_async(&config, || tool_download::cli::execute(args))
                .await
        }
        Err(error) => xt_core::exit_result(Err::<(), _>(error)),
    }
}
