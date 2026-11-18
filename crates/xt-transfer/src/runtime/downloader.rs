use crate::authentication::Session;
use crate::config::TransferSettings;
use crate::domain::{Asset, DownloadOptions, DownloadOutcome, DownloadTask};
use crate::interfaces::RequestAuthorization;
use crate::interfaces::{DownloadFuture, DownloadObserver, FileDownloader};
use crate::runtime::url::validate_url;
use hya_net::tls::TlsCapableConnector;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use url::Url;
use xt_error::Result;

#[derive(Clone)]
pub struct HyaDownloader {
    pub(crate) settings: TransferSettings,
    pub(crate) auth: Arc<dyn RequestAuthorization>,
    pub(crate) cancel: CancellationToken,
    pub(crate) cookies: Arc<Session>,
    pub(crate) hya_client: TlsCapableConnector,
}
impl HyaDownloader {
    pub fn new(
        settings: TransferSettings,
        auth: Arc<dyn RequestAuthorization>,
        cancel: CancellationToken,
    ) -> Result<Self> {
        settings.validate()?;
        let _ = rustls::crypto::ring::default_provider().install_default();
        Ok(Self {
            settings,
            auth,
            cancel,
            cookies: Arc::new(Session::default()),
            hya_client: TlsCapableConnector::new()?,
        })
    }
    pub fn session(&self) -> Arc<Session> {
        self.cookies.clone()
    }
    pub(crate) fn url(&self, value: &str) -> Result<Url> {
        validate_url(value, self.settings.allow_http_loopback)
    }
    pub fn with_authorization(mut self, auth: Arc<dyn RequestAuthorization>) -> Self {
        self.auth = auth;
        self
    }
}

impl FileDownloader for HyaDownloader {
    fn remote_size<'a>(&'a self, asset: &'a Asset) -> DownloadFuture<'a, Option<u64>> {
        Box::pin(async move { Ok(self.probe(&asset.url).await?.identity.size) })
    }

    fn download<'a>(
        &'a self,
        task: &'a DownloadTask,
        options: DownloadOptions,
        observer: &'a dyn DownloadObserver,
    ) -> DownloadFuture<'a, DownloadOutcome> {
        Box::pin(self.file(task, options, observer))
    }
}
