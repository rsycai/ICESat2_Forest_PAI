use crate::services::transport::builder::client_builder;
use crate::services::transport::cookies::CatalogCookies;
use crate::services::transport::request::{cancelled, network_error};
use crate::services::transport::settings::RequestSettings;
use reqwest::header::HeaderMap;
use reqwest::{Client, Url};
use std::sync::Arc;
use xt_error::{Error, ErrorKind, Result};
use xt_transfer::{CancellationToken, HyaDownloader, RequestAuthorization, TransferSettings};

#[derive(Clone)]
pub struct DownloadClient {
    pub(super) settings: TransferSettings,
    pub(super) requests: RequestSettings,
    pub(super) auth: Arc<dyn RequestAuthorization>,
    pub(super) cancel: CancellationToken,
    pub(super) client: Client,
    pub(super) download: HyaDownloader,
}

impl DownloadClient {
    pub fn new(
        settings: TransferSettings,
        auth: Arc<dyn RequestAuthorization>,
        cancel: CancellationToken,
    ) -> Result<Self> {
        let download = HyaDownloader::new(settings.clone(), auth.clone(), cancel.clone())?;
        let client = client_builder(&settings)?
            .cookie_provider(Arc::new(CatalogCookies(download.session())))
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(network_error)?;
        Ok(Self {
            settings,
            requests: RequestSettings::default(),
            auth,
            cancel,
            client,
            download,
        })
    }

    pub fn with_requests(mut self, settings: RequestSettings) -> Result<Self> {
        settings.validate()?;
        self.requests = settings;
        Ok(self)
    }

    pub fn with_authorization(mut self, auth: Arc<dyn RequestAuthorization>) -> Self {
        self.auth = auth.clone();
        self.download = self.download.with_authorization(auth);
        self
    }

    pub(super) fn url(&self, value: &str) -> Result<Url> {
        xt_transfer::validate_url(value, self.settings.allow_http_loopback)
    }
    pub(crate) async fn request(
        &self,
        mut method: crate::services::transport::HttpMethod,
        mut url: Url,
        headers: HeaderMap,
        mut body: Option<Vec<u8>>,
    ) -> Result<reqwest::Response> {
        for _ in 0..=10 {
            self.url(url.as_str())?;
            let mut request_headers = headers.clone();
            request_headers.extend(self.auth.headers(&url)?);
            let mut request = self
                .client
                .request(
                    reqwest::Method::from_bytes(method.as_str().as_bytes())
                        .expect("Standard HTTP method"),
                    url.clone(),
                )
                .headers(request_headers);
            if let Some(body) = &body {
                request = request.body(body.clone());
            }
            let response = tokio::select! {
                _ = self.cancel.cancelled() => return Err(cancelled()),
                response = request.send() => response.map_err(network_error)?,
            };
            if matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308) {
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|h| h.to_str().ok())
                    .ok_or_else(|| {
                        Error::new(ErrorKind::InvalidArgument, "Redirect is missing a valid Location")
                    })?;
                let target = url
                    .join(location)
                    .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid redirect URL"))?;
                if response.status().as_u16() == 303
                    || (matches!(response.status().as_u16(), 301 | 302)
                        && method == crate::services::transport::HttpMethod::Post)
                {
                    method = crate::services::transport::HttpMethod::Get;
                    body = None;
                }
                if body.is_some() && target.origin() != url.origin() {
                    return Err(Error::new(
                        ErrorKind::InvalidArgument,
                        "Cross-origin request-body redirects are forbidden",
                    ));
                }
                url = target;
                continue;
            }
            return Ok(response);
        }
        Err(Error::new(ErrorKind::InvalidArgument, "Redirect limit exceeded"))
    }
}
