use crate::cli::params::Cli;
use crate::workflow::process_pair;
use xt_error::Result;

pub fn run(cli: Cli) -> Result<()> {
    cli.load_config()?;
    execute(cli)
}

pub fn execute(cli: Cli) -> Result<()> {
    let params = cli.resolve()?;
    let report = process_pair(&params.request, &params.options)?;
    if report.skipped {
        return Ok(());
    }
    tracing::info!(
        output = %xt_manifest::display_path(&report.output),
        manifest = %xt_manifest::display_path(&report.manifest),
        "ATL03/ATL08 match saved"
    );
    for beam in &report.beams {
        tracing::info!(
            beam = %beam.beam,
            photons = beam.stats.matched_photons,
            segments = beam.stats.kept_segments,
            outside_land_photons = beam.stats.unmatched_land_photons,
            "Beam matching summary"
        );
    }
    Ok(())
}
