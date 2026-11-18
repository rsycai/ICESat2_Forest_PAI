use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(about = "Create or update an XML manifest using a custom model")]
pub(crate) struct Cli {
    #[arg(value_name = "MANIFEST.xml")]
    pub xml_path: PathBuf,
    #[arg(value_name = "OUTPUT_FILE")]
    pub output: PathBuf,
}
