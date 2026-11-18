use crate::cli::params::arguments::{Cli, Command};
use crate::cli::params::runtime::{DownloadMode, DownloadParams};
use crate::config::DownloadConfig;
use crate::domain::{RunOptions, SearchQuery, TimeRange};
use std::time::Duration;
use xt_error::Result;
use xt_transfer::{DownloadOptions, TransferSettings};

impl Cli {
    pub fn resolve(self) -> Result<DownloadParams> {
        self.resolve_with_paths(
            DownloadConfig::new()?,
            xt_config::paths::PathsConfig::new()?,
        )
    }

    pub fn resolve_with(self, defaults: DownloadConfig) -> Result<DownloadParams> {
        self.resolve_with_paths(defaults, xt_config::paths::PathsConfig::default())
    }

    pub fn resolve_with_paths(
        self,
        defaults: DownloadConfig,
        mut paths: xt_config::paths::PathsConfig,
    ) -> Result<DownloadParams> {
        let (mode, args) = match self.command {
            Command::Plan(args) => (DownloadMode::Plan, args),
            Command::Search(args) => (DownloadMode::Search, args),
            Command::Fetch(args) => (DownloadMode::Fetch, args),
        };
        if let Some(root) = args.output {
            paths.root = root;
        }
        paths.validate()?;
        let speed_limit = args.speed_limit.unwrap_or(defaults.speed_limit);
        let proxy = match (args.no_proxy, args.proxy) {
            (true, _) => Some(String::new()),
            (false, Some(proxy)) => Some(proxy),
            (false, None) => (!defaults.proxy.is_empty()).then_some(defaults.proxy),
        };
        let params = DownloadParams {
            mode,
            source: args.source,
            product: (args.product != "default").then_some(args.product),
            release: (args.release != "default").then_some(args.release),
            query: SearchQuery {
                bbox: args.range.into_range().resolve()?,
                time: Some(TimeRange::new(
                    &args.start.to_rfc3339(),
                    &args.end.to_rfc3339(),
                )?),
                page_size: args.page_size.unwrap_or(defaults.page_size),
            },
            transfer: TransferSettings {
                workers: args.workers.unwrap_or(defaults.workers),
                multipart_threshold_bytes: args
                    .multipart_threshold
                    .unwrap_or(defaults.multipart_threshold_bytes),
                timeout: Duration::from_secs(args.timeout.unwrap_or(defaults.timeout_seconds)),
                proxy,
                speed_limit: (speed_limit > 0).then_some(speed_limit),
                lock_directory: None,
                allow_http_loopback: false,
            },
            requests: crate::services::transport::RequestSettings {
                retries: args.request_retries.unwrap_or(defaults.request_retries),
                max_retry_delay: Duration::from_secs(
                    args.request_max_retry_delay
                        .unwrap_or(defaults.request_max_retry_delay_seconds),
                ),
            },
            options: RunOptions {
                progress: args.progress.unwrap_or(defaults.progress),
                paths,
                concurrency: args
                    .concurrency
                    .map(usize::from)
                    .unwrap_or(defaults.concurrency),
                download: DownloadOptions {
                    existing: args.existing.unwrap_or(defaults.existing).into(),
                    resume: if args.no_resume {
                        false
                    } else {
                        args.resume.unwrap_or(defaults.resume)
                    },
                },
                search_only: mode == DownloadMode::Search,
            },
        };
        params.validate()?;
        Ok(params)
    }

    pub fn load_config(&self) -> crate::Result<()> {
        let args = match &self.command {
            Command::Plan(args) | Command::Search(args) | Command::Fetch(args) => args,
        };
        if let Some(path) = &args.config {
            xt_config::AppConfig::from_file(path)?;
            Ok(())
        } else {
            xt_config::ensure_loaded()
        }
    }
}
