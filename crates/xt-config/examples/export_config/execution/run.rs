use crate::model::Cli;
#[cfg(windows)]
use crate::model::ExampleLogConfig;
use clap::Parser;

#[cfg(windows)]
pub(crate) fn run() -> xt_config::Result<()> {
    let path = Cli::parse().path;
    let report = xt_config::envload!(path)?;
    println!(
        "Read {} files, wrote {} environment variables, retained {} existing variables",
        report.files, report.written, report.preserved
    );
    let logging = xt_config::get::<ExampleLogConfig>()?;
    println!("Log directory:{}", logging.directory.display());
    Ok(())
}

#[cfg(not(windows))]
pub(crate) fn run() -> xt_config::Result<()> {
    let path = Cli::parse().path;
    let prepared = xt_config::config_env::EnvConfig::from_path(path, "XT")?;
    println!(
        "Loaded {} files and generated {} configuration entries",
        prepared.files().len(),
        prepared.variables().len()
    );
    Ok(())
}
