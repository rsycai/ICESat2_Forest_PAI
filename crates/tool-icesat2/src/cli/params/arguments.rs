use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "xt-icesat2",
    version,
    about = "Match ATL03/ATL08 while retaining original fields and quality information",
    arg_required_else_help = true,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Match(MatchArgs),
}

#[derive(Args, Debug, Clone)]
pub struct MatchArgs {
    #[arg(long, value_name = "FILE")]
    pub atl03: PathBuf,
    #[arg(long, value_name = "FILE")]
    pub atl08: PathBuf,
    #[arg(long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    #[arg(long = "beam", value_parser = ["gt1l", "gt1r", "gt2l", "gt2r", "gt3l", "gt3r"])]
    pub beams: Vec<String>,
    #[arg(long = "read-window")]
    pub read_window: Option<std::num::NonZeroUsize>,
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,
    #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    pub overwrite: Option<bool>,
}
