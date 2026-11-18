use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, ClientBuilder};
use xt_error::{Error, ErrorKind, Result};
use xt_transfer::TransferSettings;

pub(crate) fn client_builder(settings: &TransferSettings) -> Result<ClientBuilder> {
    let mut builder = Client::builder()
        .user_agent("xt-download/0.1")
        .connect_timeout(settings.timeout)
        .read_timeout(settings.timeout)
        .default_headers(HeaderMap::from_iter([(
            reqwest::header::ACCEPT_ENCODING,
            HeaderValue::from_static("identity"),
        )]));
    if let Some(proxy) = &settings.proxy {
        builder = builder.no_proxy();
        if !proxy.is_empty() {
            builder = builder.proxy(
                reqwest::Proxy::all(proxy)
                    .map_err(|_| Error::new(ErrorKind::Configuration, "Invalid proxy URL"))?,
            );
        }
    }
    Ok(builder)
}
