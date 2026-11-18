use url::Url;
use xt_error::{Error, ErrorKind, Result};

pub fn validate_url(value: &str, allow_local: bool) -> Result<Url> {
    let url =
        Url::parse(value).map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid download URL"))?;
    let local = allow_local
        && url.scheme() == "http"
        && url.host_str().is_some_and(|h| {
            h == "localhost"
                || h.parse::<std::net::IpAddr>()
                    .is_ok_and(|ip| ip.is_loopback())
        });
    if (url.scheme() != "https" && !local)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(Error::new(
            ErrorKind::InvalidArgument,
            "Only HTTPS URLs without user information or fragments are allowed",
        ));
    }
    Ok(url)
}
