#[cfg(windows)]
#[derive(xt_config::EnvLoad, xt_config::SubConfig)]
#[env(prefix = "XT", section = "logging")]
pub(crate) struct ExampleLogConfig {
    #[env(default = "logs", os)]
    pub directory: std::path::PathBuf,
}
