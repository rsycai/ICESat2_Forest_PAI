#[derive(Debug, Clone, serde::Serialize, xt_config::EnvLoad, xt_config::SubConfig)]
#[env(prefix = "XT", section = "icesat2", validate = "validate")]
pub struct Icesat2Config {
    #[env(default = false)]
    #[serde(skip)]
    pub overwrite: bool,
    #[env(default = 65536)]
    pub read_window_rows: usize,
}

impl Icesat2Config {
    pub fn validate(&self) -> xt_error::Result<()> {
        if !(1..=1_048_576).contains(&self.read_window_rows) {
            return Err(xt_error::Error::new(
                xt_error::ErrorKind::Configuration,
                "icesat2: read window must be in 1..=1048576",
            ));
        }
        Ok(())
    }
}
