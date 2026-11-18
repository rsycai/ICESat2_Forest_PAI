use crate::authentication::headers::build_headers;
use crate::authentication::policy::Authentication;
use crate::interfaces::RequestAuthorization;
use http::HeaderMap;
use std::collections::HashMap;
use std::path::Path;
use url::{Host, Origin, Url};
use xt_error::{Error, ErrorKind, Result};

pub(super) struct NetrcAuthorization {
    machines: HashMap<String, HeaderMap>,
    fallback: Option<HeaderMap>,
    initial_origin: Origin,
}

impl NetrcAuthorization {
    pub(super) fn load(path: Option<&Path>, url: &Url) -> Result<Self> {
        let path = path
            .map(Path::to_path_buf)
            .or_else(|| std::env::var_os("NETRC").map(Into::into))
            .or_else(::netrc::Netrc::get_file)
            .ok_or_else(|| {
                Error::new(
                    ErrorKind::InvalidArgument,
                    "No .netrc file found; specify the credentials file path",
                )
            })?;
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content, url)
    }

    pub(super) fn parse(content: &str, url: &Url) -> Result<Self> {
        let parsed: ::netrc::Netrc = content
            .parse()
            .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid .netrc format"))?;
        let mut result = Self {
            machines: HashMap::new(),
            fallback: None,
            initial_origin: url.origin(),
        };
        for (host, auth) in parsed.hosts {
            if auth.login.is_empty() || auth.password.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidArgument,
                    ".netrc entries require nonempty login and password",
                ));
            }
            let headers = build_headers(&Authentication::Basic {
                username: auth.login,
                password: auth.password,
            })?;
            if host == "default" {
                result.fallback = Some(headers);
            } else {
                let host = Host::parse(&host)
                    .map_err(|_| {
                        Error::new(
                            ErrorKind::InvalidArgument,
                            ".netrc machine must be a hostname without a scheme or path",
                        )
                    })?
                    .to_string();
                if result.machines.insert(host, headers).is_some() {
                    return Err(Error::new(
                        ErrorKind::InvalidArgument,
                        "Duplicate host entries in .netrc",
                    ));
                }
            }
        }
        if result.machines.is_empty() && result.fallback.is_none() {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "No usable account entries in .netrc",
            ));
        }
        Ok(result)
    }
}

impl RequestAuthorization for NetrcAuthorization {
    fn headers(&self, url: &Url) -> Result<HeaderMap> {
        if url.scheme() != "https" && url.origin() != self.initial_origin {
            return Ok(HeaderMap::new());
        }
        let specific = url.host_str().and_then(|host| self.machines.get(host));
        Ok(specific
            .or_else(|| {
                (url.origin() == self.initial_origin)
                    .then_some(self.fallback.as_ref())
                    .flatten()
            })
            .cloned()
            .unwrap_or_default())
    }
}
