use crate::config::TransferSettings;
use crate::hya::proxy::parse_proxy;
use hya_net::socks::{Proxy, ProxyKind};
use hya_net::tls::{MaybeTls, TlsCapableConnector};
use hya_net::{Connector, Target};
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use url::Url;

pub(crate) struct DownloadConnector {
    inner: TlsCapableConnector,
    origin: (String, u16),
    tls: bool,
    timeout: Duration,
    pub(super) rejected: Arc<AtomicBool>,
}

impl DownloadConnector {
    pub(crate) fn new(
        url: &Url,
        settings: &TransferSettings,
        base: &TlsCapableConnector,
    ) -> io::Result<(Self, Target)> {
        let host = url
            .host_str()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing host"))?;
        let port = url
            .port_or_known_default()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Missing port"))?;
        let mut path = url.path().to_owned();
        if let Some(query) = url.query() {
            path.push('?');
            path.push_str(query);
        }
        let mut target = Target::direct(host, port, &path);
        target.tls = url.scheme() == "https";
        let proxy = match &settings.proxy {
            Some(raw) if raw.is_empty() => Proxy::none(),
            Some(raw) => parse_proxy(raw)?,
            None => {
                let names = if target.tls {
                    ["https_proxy", "HTTPS_PROXY", "all_proxy", "ALL_PROXY"]
                } else {
                    ["http_proxy", "HTTP_PROXY", "all_proxy", "ALL_PROXY"]
                };
                let raw = names
                    .iter()
                    .find_map(|key| std::env::var(key).ok().filter(|v| !v.trim().is_empty()));
                let proxy = raw
                    .as_deref()
                    .map(parse_proxy)
                    .transpose()?
                    .unwrap_or_else(Proxy::none);
                if proxy.bypasses(host) {
                    Proxy::none()
                } else {
                    proxy
                }
            }
        };
        let mut inner = base.clone();
        if proxy.kind.is_socks() {
            inner = inner.with_socks(proxy);
        } else if proxy.kind == ProxyKind::Http {
            target.origin = Some(format!("{host}:{port}"));
            target.host = proxy.host;
            target.port = proxy.port;
            if let Some(user) = proxy.username {
                target.headers.push(format!(
                    "Proxy-Authorization: Basic {}",
                    hya_net::basic_auth(&user, proxy.password.as_deref().unwrap_or(""))
                ));
            }
        }
        Ok((
            Self {
                inner,
                origin: (host.to_owned(), port),
                tls: target.tls,
                timeout: settings.timeout,
                rejected: Arc::new(AtomicBool::new(false)),
            },
            target,
        ))
    }
}

impl Connector for DownloadConnector {
    type Stream = MaybeTls;

    fn connect<'a>(
        &'a self,
        target: &'a Target,
    ) -> Pin<Box<dyn Future<Output = io::Result<Self::Stream>> + Send + 'a>> {
        Box::pin(async move {
            if target.origin_endpoint() != self.origin || target.tls != self.tls {
                self.rejected.store(true, Ordering::Relaxed);
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Download source changed; probe again",
                ));
            }
            tokio::time::timeout(self.timeout, self.inner.connect(target))
                .await
                .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "Connection timed out"))?
        })
    }

    fn pool(&self) -> Option<hya_net::pool::SharedPool<Self::Stream>> {
        self.inner.pool()
    }
}
