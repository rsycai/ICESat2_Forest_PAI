///
#[derive(crate::Config)]
#[config(crate_path = "crate")]
pub struct AppConfig {}

pub(super) static APP_CONFIG: AppConfig = AppConfig {};
