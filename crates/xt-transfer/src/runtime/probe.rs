use crate::domain::remote::{Probe, RemoteIdentity};
use crate::hya::{DownloadConnector, transfer_error};
use crate::runtime::HyaDownloader;
use crate::runtime::errors::cancelled;
use crate::storage::identity::asset_key;
use hya_net::http::{header_lookup, probe_resilient, probe_via_get};
use hya_net::polite::{MAX_REDIRECTS, RedirectChain};
use xt_error::{DownloadError, Error, ErrorKind, Result};

impl HyaDownloader {
    pub(super) async fn probe(&self, value: &str) -> Result<Probe> {
        let mut url = self.url(value)?;
        let mut chain = RedirectChain::new(url.as_str());
        let mut use_get = false;
        for _ in 0..=MAX_REDIRECTS {
            let (connector, mut target) =
                DownloadConnector::new(&url, &self.settings, &self.hya_client)
                    .map_err(transfer_error)?;
            let mut headers = self.auth.headers(&url)?;
            if !headers.contains_key(http::header::COOKIE)
                && let Some(cookie) = self.cookies.cookies(&url)
            {
                headers.insert(http::header::COOKIE, cookie);
            }
            for (name, value) in &headers {
                target.headers.push(format!(
                    "{name}: {}",
                    value.to_str().map_err(|_| Error::new(
                        ErrorKind::InvalidArgument,
                        "Authentication header is not valid text"
                    ))?
                ));
            }
            target.agent = Some(concat!("xtools/", env!("CARGO_PKG_VERSION")).into());
            let reply = tokio::select! {
                _ = self.cancel.cancelled() => return Err(cancelled()),
                reply = tokio::time::timeout(self.settings.timeout, async {
                    if use_get { probe_via_get(&connector, &target).await }
                    else { probe_resilient(&connector, &target).await }
                }) => {
                    reply.map_err(|_| Error::new(ErrorKind::Download(DownloadError::Timeout), "Hya file probe timed out"))?.map_err(|e| transfer_error(e).context("Probe remote file"))?
                }
            };
            self.cookies.store(&url, &reply.raw_head);
            if reply.is_redirect() && !use_get {
                use_get = true;
                continue;
            }
            if reply.is_redirect() {
                let location = reply
                    .location
                    .as_deref()
                    .ok_or_else(|| Error::new(ErrorKind::InvalidArgument, "Redirect is missing Location"))?;
                let next = url
                    .join(location)
                    .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid redirect URL"))?;
                url = self.url(next.as_str())?;
                if !chain.advance(url.as_str()) {
                    return Err(Error::new(ErrorKind::InvalidArgument, "Redirect loop"));
                }
                continue;
            }
            if !(200..300).contains(&reply.status) {
                return Err(Error::new(
                    ErrorKind::Download(DownloadError::HttpStatus(reply.status)),
                    format!("Hya probe returned HTTP {}", reply.status),
                ));
            }
            if reply
                .content_type
                .as_deref()
                .is_some_and(|s| s.to_ascii_lowercase().contains("text/html"))
            {
                return Err(Error::new(
                    ErrorKind::Download(DownloadError::HttpStatus(401)),
                    "Received HTML; check download permissions and credentials",
                ));
            }
            return Ok(Probe {
                url,
                ranges: reply.ranges,
                identity: RemoteIdentity {
                    key: asset_key(value)?,
                    size: if reply.ranges {
                        Some(reply.size)
                    } else {
                        reply.stated_length()
                    },
                    etag: header_lookup(&reply.raw_head, "etag")
                        .filter(|s| !s.to_ascii_lowercase().starts_with("w/")),
                    modified: reply.last_modified,
                },
            });
        }
        Err(Error::new(
            ErrorKind::InvalidArgument,
            "Hya redirect limit exceeded",
        ))
    }
}
