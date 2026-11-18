use hya_net::socks::Proxy;
use std::io;
use url::Url;

pub(super) fn parse_proxy(raw: &str) -> io::Result<Proxy> {
    let raw = if raw.contains("://") {
        raw.to_owned()
    } else {
        format!("http://{raw}")
    };
    let mut url = Url::parse(&raw)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid proxy URL"))?;
    if url.scheme() == "https" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Hya does not support TLS proxies; use HTTP CONNECT or SOCKS5 for HTTPS targets",
        ));
    }
    url.set_path("");
    url.set_query(None);
    url.set_fragment(None);
    let mut proxy = Proxy::parse(url.as_str())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "Invalid proxy URL"))?;
    proxy.port = url.port_or_known_default().unwrap_or(1080);
    proxy.username = proxy.username.map(|v| hya_net::url::percent_decode(&v));
    proxy.password = proxy.password.map(|v| hya_net::url::percent_decode(&v));
    Ok(proxy)
}
