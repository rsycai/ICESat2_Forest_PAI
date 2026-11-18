use crate::cli::params::Cli;
use xt_error::Result;

pub async fn run(cli: Cli) -> Result<()> {
    cli.load_config()?;
    execute(cli).await
}

pub async fn execute(cli: Cli) -> Result<()> {
    let params = cli.resolve()?;
    crate::pipeline::command::run(params).await
}
