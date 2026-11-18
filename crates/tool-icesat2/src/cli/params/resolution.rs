use crate::processing::config::Icesat2Config;
use crate::cli::params::arguments::{Cli, Command};
use crate::cli::params::runtime::ProcessParams;
use crate::model::ProcessRequest;
use crate::processing;
use xt_error::Result;

impl Cli {
    pub fn load_config(&self) -> Result<()> {
        let Command::Match(args) = &self.command;
        if let Some(path) = &args.config {
            xt_config::AppConfig::from_file(path)?;
            Ok(())
        } else {
            xt_config::ensure_loaded()
        }
    }
}

impl Cli {
    pub fn resolve(self) -> Result<ProcessParams> {
        self.resolve_with_paths(Icesat2Config::new()?, xt_config::paths::PathsConfig::new()?)
    }

    pub fn resolve_with(self, defaults: Icesat2Config) -> Result<ProcessParams> {
        self.resolve_with_paths(defaults, xt_config::paths::PathsConfig::default())
    }

    pub fn resolve_with_paths(
        self,
        defaults: Icesat2Config,
        paths: xt_config::paths::PathsConfig,
    ) -> Result<ProcessParams> {
        let Command::Match(args) = self.command;
        processing::paired_key(&args.atl03, &args.atl08)?;
        let options = Icesat2Config {
            overwrite: args.overwrite.unwrap_or(defaults.overwrite),
            read_window_rows: args
                .read_window
                .map_or(defaults.read_window_rows, |value| value.get()),
        };
        options.validate()?;
        let directories = paths.for_data("ATL03_08")?;
        let manifest = xt_icesat2_meta::manifest::record_path_in(
            &directories.manifests,
            &args.atl03,
            &args.atl08,
        )?;
        let output = match args.output {
            Some(path) => path,
            None => {
                processing::dated_output_path(&directories.processed, &args.atl03, &args.atl08)?
            }
        };
        Ok(ProcessParams {
            request: ProcessRequest {
                atl03: args.atl03,
                atl08: args.atl08,
                output,
                manifest: Some(manifest),
                beams: args.beams,
                overwrite: options.overwrite,
            },
            options,
        })
    }
}
