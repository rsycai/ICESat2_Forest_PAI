use crate::config::Existing;
use crate::domain::{BoundingBox, DownloadRange, ShapefileCrs};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "ICESat2_Forest_PAI-download",
    version,
    about = "Search and download registered data products",
    arg_required_else_help = true,
    propagate_version = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Build a search plan without downloading.
    Plan(DownloadArgs),
    /// Search and write a download manifest.
    Search(DownloadArgs),
    /// Search, write a manifest, and download files.
    Fetch(DownloadArgs),
}

#[derive(Args, Debug, Clone)]
pub struct DownloadArgs {
    #[arg(value_name = "SOURCE")]
    /// Registered data source identifier.
    pub source: SourceId,
    #[arg(value_name = "PRODUCT")]
    /// Product name, or default to use the catalog value.
    pub product: String,
    #[arg(value_name = "VERSION")]
    /// Product release, or default to use the catalog value.
    pub release: String,
    #[arg(value_name = "START")]
    /// Start time in RFC3339 format, including timezone.
    pub start: chrono::DateTime<chrono::FixedOffset>,
    #[arg(value_name = "END")]
    /// End time in RFC3339 format, including timezone.
    pub end: chrono::DateTime<chrono::FixedOffset>,
    #[command(flatten)]
    pub range: RangeArgs,
    #[arg(long, value_name = "FILE")]
    /// Configuration file; defaults to config/config.yaml.
    pub config: Option<PathBuf>,
    #[arg(long, value_parser = clap::value_parser!(u16).range(1..))]
    /// Records per search page.
    pub page_size: Option<u16>,
    #[arg(long, value_name = "DIR")]
    /// Download root directory.
    pub output: Option<PathBuf>,
    #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true", require_equals = true)]
    /// Enable periodic progress messages.
    pub progress: Option<bool>,
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=64))]
    /// Concurrent downloads (1-64).
    pub concurrency: Option<u8>,
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=64))]
    /// Maximum parallel chunks per file (1-64).
    pub workers: Option<u64>,
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
    /// Minimum file size in bytes for multipart downloads.
    pub multipart_threshold: Option<u64>,
    #[arg(long, value_parser = clap::value_parser!(u8).range(0..=20))]
    /// Additional search and authentication retries (0-20).
    pub request_retries: Option<u8>,
    #[arg(long, value_parser = clap::value_parser!(u64).range(..=86400))]
    /// Maximum retry delay in seconds.
    pub request_max_retry_delay: Option<u64>,
    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=86400))]
    /// Connection and stall timeout in seconds.
    pub timeout: Option<u64>,
    #[arg(long, conflicts_with = "no_proxy")]
    /// Proxy URL; an empty value disables the proxy.
    pub proxy: Option<String>,
    #[arg(long)]
    /// Disable proxy use.
    pub no_proxy: bool,
    #[arg(long, value_parser = clap::value_parser!(u64).range(..=u32::MAX as u64))]
    /// Per-file rate limit in bytes per second; 0 means unlimited.
    pub speed_limit: Option<u64>,
    #[arg(long, value_enum)]
    /// Policy for existing files.
    pub existing: Option<Existing>,
    #[arg(long, action = clap::ArgAction::Set, num_args = 0..=1, default_missing_value = "true", require_equals = true, conflicts_with = "no_resume")]
    /// Resume eligible incomplete downloads.
    pub resume: Option<bool>,
    #[arg(long)]
    /// Disable resuming.
    pub no_resume: bool,
}

#[derive(Args, Debug, Clone, Default)]
pub struct RangeArgs {
    #[arg(long = "global", conflicts_with_all = ["bbox", "shp", "bounds"])]
    /// Search worldwide without spatial filtering.
    pub global: bool,
    #[arg(long, value_name = "W,S,E,N", allow_hyphen_values = true,
        conflicts_with_all = ["shp", "bounds"])]
    /// Bounding box in west,south,east,north order.
    pub bbox: Option<BoundingBox>,
    #[arg(long, value_name = "FILE", conflicts_with = "bounds")]
    /// Shapefile whose bounding box defines the search area.
    pub shp: Option<PathBuf>,
    #[arg(long, requires = "shp")]
    /// Treat shapefile coordinates as WGS84 longitude and latitude.
    pub assume_wgs84: bool,
    #[arg(value_name = "W,S,E,N", allow_hyphen_values = true)]
    /// Bounding box supplied as a positional argument.
    pub bounds: Option<BoundingBox>,
}

impl RangeArgs {
    pub fn into_range(self) -> DownloadRange {
        if let Some(bounds) = self.bbox.or(self.bounds) {
            DownloadRange::Bounds(bounds)
        } else if let Some(path) = self.shp {
            DownloadRange::Shapefile {
                path,
                crs: if self.assume_wgs84 {
                    ShapefileCrs::AssumeWgs84
                } else {
                    ShapefileCrs::CheckPrj
                },
            }
        } else {
            DownloadRange::Global
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceId(String);
impl SourceId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl std::str::FromStr for SourceId {
    type Err = crate::Error;
    fn from_str(value: &str) -> crate::Result<Self> {
        if value.is_empty()
            || !value.as_bytes()[0].is_ascii_alphanumeric()
            || !value
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c))
        {
            return Err(crate::domain::invalid("Invalid data source identifier"));
        }
        Ok(Self(value.into()))
    }
}
