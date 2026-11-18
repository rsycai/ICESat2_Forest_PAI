use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(about = "Load a configuration directory and export or preview environment mappings")]
pub(crate) struct Cli {
    #[arg(default_value = "examples/config")]
    pub path: std::path::PathBuf,
}
