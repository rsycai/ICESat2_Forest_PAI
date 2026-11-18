use std::path::PathBuf;
use clap::Parser;
#[derive(Parser)]
#[command(about="ICESat2_Forest_PAI: ATL03 + ATL08 + landcover -> forest PAI; G(0)=0.5")]
struct Args {
    #[arg(long)] config: Option<PathBuf>,
    #[arg(long, default_value = "tests/data/ATL03_20200101235632_00960601_007_01.h5")] atl03: PathBuf,
    #[arg(long, default_value = "tests/data/ATL08_20200101235632_00960601_007_01.h5")] atl08: PathBuf,
    #[arg(long, default_value = "tests/data/landcover.tif")] landcover: PathBuf,
    #[arg(short,long, default_value = "tests/output/PAI.h5")] output: PathBuf,
}
fn main() -> anyhow::Result<()> {
    let a = Args::parse();
    if let Some(path) = &a.config { xt_config::AppConfig::from_file(path)?; }
    for p in [&a.atl03, &a.atl08, &a.landcover] { anyhow::ensure!(p.is_file(), "Input does not exist: {}", p.display()); }
    let temp = tempfile::tempdir()?;
    let stem = a.atl03.file_stem().and_then(|s|s.to_str()).ok_or_else(|| anyhow::anyhow!("Invalid ATL03 filename"))?;
    anyhow::ensure!(stem.starts_with("ATL03_"), "Use original ATL03 granule filename");
    let matched = temp.path().join(format!("ATL03_08_{}.h5", &stem[6..]));
    let manifest = temp.path().join("manifest.xml");
    tool_icesat2::process_pair(&tool_icesat2::ProcessRequest {
        atl03:a.atl03, atl08:a.atl08, output:matched.clone(), manifest:Some(manifest.clone()), beams:vec![], overwrite:false,
    }, &tool_icesat2::Icesat2Config::new()?)?;
    let config = tool_pai::ProductConfig { landcover:a.landcover.to_string_lossy().into_owned(), ..tool_pai::ProductConfig::new()?};
    let product=tool_pai::run_product(&matched,&temp.path().join("pai.h5"),&config,&manifest)?;
    anyhow::ensure!(!a.output.exists(), "Output already exists");
    if let Some(parent) = a.output.parent().filter(|p| !p.as_os_str().is_empty()) { std::fs::create_dir_all(parent)?; }
    std::fs::copy(&product, &a.output)?;
    println!("{}", a.output.display());
    Ok(())
}



