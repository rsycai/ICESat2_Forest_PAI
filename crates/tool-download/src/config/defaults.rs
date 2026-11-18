use clap::ValueEnum;
use xt_transfer::ExistingFilePolicy;

#[derive(Debug, Clone, xt_config::EnvLoad, xt_config::SubConfig)]
#[env(prefix = "XT", section = "download", validate = "validate")]
pub struct DownloadConfig {
    #[env(default = 100)]
    pub page_size: u16,
    #[env(default = false)]
    pub progress: bool,
    #[env(default = 4)]
    pub concurrency: usize,
    #[env(default = 4)]
    pub workers: u64,
    #[env(default = 1_000_000_000)]
    pub multipart_threshold_bytes: u64,
    #[env(default = 3)]
    pub request_retries: u8,
    #[env(default = 60)]
    pub timeout_seconds: u64,
    #[env(default = 120)]
    pub request_max_retry_delay_seconds: u64,
    #[env(default = true)]
    pub resume: bool,
    #[env(default = Existing::Skip)]
    pub existing: Existing,
    #[env(default = "")]
    pub proxy: String,
    #[env(default = 0)]
    pub speed_limit: u64,
}

impl DownloadConfig {
    pub fn validate(&self) -> xt_error::Result<()> {
        if self.page_size == 0 {
            return Err(xt_error::Error::new(
                xt_error::ErrorKind::Configuration,
                "download.page_size must be greater than 0",
            ));
        }
        if self.multipart_threshold_bytes == 0 {
            return Err(xt_error::Error::new(
                xt_error::ErrorKind::Configuration,
                "download.multipart_threshold_bytes must be greater than 0",
            ));
        }
        if self.concurrency == 0
            || self.concurrency > 64
            || self.workers == 0
            || self.workers > 64
            || self.speed_limit > u32::MAX as u64
            || self.timeout_seconds == 0
            || self.timeout_seconds > 86400
            || self.request_retries > 20
            || self.request_max_retry_delay_seconds > 86400
        {
            return Err(xt_error::Error::new(
                xt_error::ErrorKind::Configuration,
                "Concurrency must be 1..=64, timeout 1..=86400 seconds, retries at most 20, and retry delay at most 86400 seconds",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum Existing {
    Fail,
    #[default]
    Skip,
    Overwrite,
}

impl From<Existing> for ExistingFilePolicy {
    fn from(value: Existing) -> Self {
        match value {
            Existing::Fail => Self::Fail,
            Existing::Skip => Self::Skip,
            Existing::Overwrite => Self::Overwrite,
        }
    }
}

impl std::str::FromStr for Existing {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        <Self as ValueEnum>::from_str(value, false)
    }
}
