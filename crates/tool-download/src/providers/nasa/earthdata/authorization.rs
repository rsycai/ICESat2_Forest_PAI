use reqwest::Url;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use xt_error::{Error, ErrorKind, Result};
use xt_transfer::RequestAuthorization;

#[derive(Clone, Default)]
pub struct EarthdataAuthorization {
    pub(super) token: Option<HeaderValue>,
}

impl EarthdataAuthorization {
    pub fn from_env_or_token(xml_token: &str) -> Result<Self> {
        Self::from_sources(std::env::var("EARTHDATA_TOKEN"), xml_token)
    }
    pub(super) fn from_sources(
        env_token: std::result::Result<String, std::env::VarError>,
        xml_token: &str,
    ) -> Result<Self> {
        let token = match env_token {
            Ok(token) if !token.trim().is_empty() => return Self::with_token(&token),
            Ok(_) | Err(std::env::VarError::NotPresent) => xml_token,
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::Configuration,
                    "EARTHDATA_TOKEN must be valid Unicode text",
                ));
            }
        };
        if token.trim().is_empty() {
            Ok(Self::default())
        } else {
            Self::with_token(token)
        }
    }

    pub fn from_env() -> Result<Self> {
        match std::env::var("EARTHDATA_TOKEN") {
            Ok(token) => Self::with_token(&token),
            Err(std::env::VarError::NotPresent) => Ok(Self::default()),
            Err(_) => Err(Error::new(
                ErrorKind::Configuration,
                "EARTHDATA_TOKEN must be valid Unicode text",
            )),
        }
    }
    pub fn with_token(token: &str) -> Result<Self> {
        if token.trim().is_empty() {
            return Err(Error::new(
                ErrorKind::Configuration,
                "Earthdata token must not be empty",
            ));
        }
        let mut value = HeaderValue::from_str(&format!("Bearer {}", token.trim()))
            .map_err(|_| Error::new(ErrorKind::Configuration, "Invalid Earthdata token format"))?;
        value.set_sensitive(true);
        Ok(Self { token: Some(value) })
    }
}

impl RequestAuthorization for EarthdataAuthorization {
    fn headers(&self, url: &Url) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let trusted = url.scheme() == "https"
            && url.port_or_known_default() == Some(443)
            && url.host_str().is_some_and(|host| {
                [
                    "earthdata.nasa.gov",
                    "earthdatacloud.nasa.gov",
                    "ecs.nsidc.org",
                ]
                .iter()
                .any(|domain| {
                    host == *domain
                        || host
                            .strip_suffix(domain)
                            .is_some_and(|prefix| prefix.ends_with('.'))
                })
            });
        if trusted && let Some(token) = &self.token {
            headers.insert(AUTHORIZATION, token.clone());
        }
        Ok(headers)
    }
}
