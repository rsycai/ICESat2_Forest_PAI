use crate::config::catalog::AuthenticationProfile;
use crate::providers::nasa::earthdata::authorization::EarthdataAuthorization;
use crate::services::credentials::FixedAuthentication;
use crate::{Authentication, AuthenticationProvider, Result};
use std::sync::Arc;

pub struct EarthdataAuthenticationProvider;
impl AuthenticationProvider for EarthdataAuthenticationProvider {
    const ID: &'static str = "nasa.earthdata";
    fn create(profile: &AuthenticationProfile) -> Result<Box<dyn Authentication>> {
        let name = profile.value("token_env").unwrap_or("EARTHDATA_TOKEN");
        let auth = EarthdataAuthorization::from_sources(
            std::env::var(name),
            profile.value("token").unwrap_or_default(),
        )?;
        Ok(Box::new(FixedAuthentication {
            authorization: Arc::new(auth),
        }))
    }
}
crate::register_authentication!(EarthdataAuthenticationProvider);
