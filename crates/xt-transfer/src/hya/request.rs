use crate::config::TransferSettings;
use http::HeaderMap;
use std::path::PathBuf;
use url::Url;

pub(crate) struct Transfer {
    pub settings: TransferSettings,
    pub client: hya_net::tls::TlsCapableConnector,
    pub url: Url,
    pub headers: HeaderMap,
    pub path: PathBuf,
    pub size: Option<u64>,
    pub ranges: bool,
    pub resume: bool,
    pub asset_key: String,
    pub validator: Option<String>,
}
