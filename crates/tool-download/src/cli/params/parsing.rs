use clap::Parser;
use std::ffi::OsString;
use xt_error::Result;

pub fn parse_range(values: &[OsString]) -> Result<crate::DownloadRange> {
    #[derive(Parser)]
    #[command(disable_help_flag = true)]
    struct RangeParser {
        #[command(flatten)]
        range: crate::cli::params::RangeArgs,
    }
    RangeParser::try_parse_from(
        std::iter::once(OsString::from("range")).chain(values.iter().cloned()),
    )
    .map(|args| args.range.into_range())
    .map_err(|error| crate::domain::invalid(&error.to_string()))
}
