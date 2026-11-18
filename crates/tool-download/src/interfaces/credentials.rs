use crate::config::catalog::AuthenticationProfile;
use crate::services::transport::HttpTransport;
use crate::{DownloadFuture, Result};
use std::sync::Arc;
use xt_transfer::RequestAuthorization;

pub trait Authentication: Send + Sync {
    fn authorize<'a>(
        &'a self,
        transport: &'a dyn HttpTransport,
    ) -> DownloadFuture<'a, Arc<dyn RequestAuthorization>>;
}

pub trait AuthenticationProvider: Send + Sync + 'static {
    const ID: &'static str;
    fn create(profile: &AuthenticationProfile) -> Result<Box<dyn Authentication>>;
}
