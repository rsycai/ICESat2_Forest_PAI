use crate::config::catalog::AuthenticationProfile;
use crate::domain::invalid;
use crate::services::transport::HttpTransport;
use crate::{Authentication, AuthenticationProvider, DownloadFuture, Result};
use reqwest::Url;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use std::collections::BTreeSet;
use std::sync::Arc;
use xt_transfer::RequestAuthorization;

pub struct ScopedBearer {
    token: HeaderValue,
    hosts: BTreeSet<String>,
}
impl ScopedBearer {
    pub fn new(token: &str, hosts: &str) -> Result<Self> {
        if token.trim().is_empty() || token.trim().chars().any(char::is_whitespace) {
            return Err(invalid("Invalid Bearer token format"));
        }
        let mut token = HeaderValue::from_str(&format!("Bearer {}", token.trim()))
            .map_err(|_| invalid("Invalid Bearer credentials"))?;
        token.set_sensitive(true);
        let mut allowed = BTreeSet::new();
        for host in hosts.split(',').map(str::trim) {
            let url =
                Url::parse(&format!("https://{host}")).map_err(|_| invalid("Invalid authentication host"))?;
            if host.contains(['/', ':', '@', '?', '#'])
                || host.is_empty()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.port().is_some()
                || url.path() != "/"
                || url.query().is_some()
                || url.fragment().is_some()
            {
                return Err(invalid("Authentication profile requires an exact hostname"));
            }
            allowed.insert(
                url.host_str()
                    .ok_or_else(|| invalid("Authentication host is missing"))?
                    .to_owned(),
            );
        }
        if token.as_bytes() == b"Bearer " || allowed.is_empty() {
            return Err(invalid("Bearer token or host is not configured"));
        }
        Ok(Self {
            token,
            hosts: allowed,
        })
    }
}
impl RequestAuthorization for ScopedBearer {
    fn headers(&self, url: &Url) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        if url.scheme() == "https"
            && url.port_or_known_default() == Some(443)
            && url.host_str().is_some_and(|host| self.hosts.contains(host))
        {
            headers.insert(AUTHORIZATION, self.token.clone());
        }
        Ok(headers)
    }
}

pub struct BearerAuthenticationProvider;
impl AuthenticationProvider for BearerAuthenticationProvider {
    const ID: &'static str = "bearer";
    fn create(profile: &AuthenticationProfile) -> Result<Box<dyn Authentication>> {
        let from_env = match profile.value("token_env") {
            Some(name) if !name.is_empty() => match std::env::var(name) {
                Ok(value) if !value.trim().is_empty() => Some(value),
                Ok(_) | Err(std::env::VarError::NotPresent) => None,
                Err(_) => return Err(invalid("Token environment variable is not valid text")),
            },
            _ => None,
        };
        let token = from_env
            .as_deref()
            .or_else(|| profile.value("token"))
            .unwrap_or_default();
        let auth = ScopedBearer::new(token, profile.value("hosts").unwrap_or_default())?;
        Ok(Box::new(FixedAuthentication {
            authorization: Arc::new(auth),
        }))
    }
}
crate::register_authentication!(BearerAuthenticationProvider);

pub struct FixedAuthentication {
    pub authorization: Arc<dyn RequestAuthorization>,
}

impl Authentication for FixedAuthentication {
    fn authorize<'a>(
        &'a self,
        _: &'a dyn HttpTransport,
    ) -> DownloadFuture<'a, Arc<dyn RequestAuthorization>> {
        Box::pin(async move { Ok(self.authorization.clone()) })
    }
}
