use crate::authentication::policy::Authentication;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use http::header::{AUTHORIZATION, COOKIE, HeaderMap, HeaderName, HeaderValue};
use xt_error::{Error, ErrorKind, Result};

pub(super) fn build_headers(auth: &Authentication) -> Result<HeaderMap> {
    let (name, value) = match auth {
        Authentication::Bearer { token } => {
            if token.is_empty() || token.bytes().any(|b| b.is_ascii_whitespace()) {
                return Err(invalid("Bearer token must be nonempty and contain no whitespace"));
            }
            (AUTHORIZATION, format!("Bearer {token}"))
        }
        Authentication::Basic { username, password } => {
            if username.contains(':')
                || username
                    .chars()
                    .chain(password.chars())
                    .any(char::is_control)
            {
                return Err(invalid("Basic username must not contain a colon; credentials must not contain control characters"));
            }
            (
                AUTHORIZATION,
                format!(
                    "Basic {}",
                    STANDARD.encode(format!("{username}:{password}"))
                ),
            )
        }
        Authentication::ApiKey { header, value } => {
            let name = HeaderName::from_bytes(header.as_bytes())
                .map_err(|_| invalid("Invalid API key header name"))?;
            if matches!(
                name.as_str(),
                "authorization"
                    | "proxy-authorization"
                    | "cookie"
                    | "host"
                    | "connection"
                    | "content-length"
                    | "transfer-encoding"
                    | "upgrade"
                    | "trailer"
                    | "te"
                    | "range"
                    | "if-range"
                    | "if-match"
                    | "if-unmodified-since"
                    | "accept-encoding"
            ) {
                return Err(invalid("API key must not override authentication, routing, or transport headers"));
            }
            if value.trim().is_empty() {
                return Err(invalid("API key must not be empty"));
            }
            (name, value.clone())
        }
        Authentication::Cookie { value } => {
            if value.trim().is_empty() || value.chars().any(char::is_control) {
                return Err(invalid("Cookie must be nonempty and contain no control characters"));
            }
            (COOKIE, value.clone())
        }
        Authentication::None | Authentication::Custom(_) | Authentication::Netrc { .. } => {
            return Ok(HeaderMap::new());
        }
    };
    let mut value = HeaderValue::from_str(&value).map_err(|_| invalid("Invalid authentication header value"))?;
    if value.to_str().is_err() {
        return Err(invalid("Authentication header must be valid ASCII text"));
    }
    value.set_sensitive(true);
    let mut headers = HeaderMap::new();
    headers.insert(name, value);
    Ok(headers)
}

fn invalid(message: &str) -> Error {
    Error::new(ErrorKind::InvalidArgument, message)
}
