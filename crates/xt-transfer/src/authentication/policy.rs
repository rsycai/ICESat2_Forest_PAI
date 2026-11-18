use crate::authentication::headers::build_headers;
use crate::authentication::netrc::NetrcAuthorization;
use crate::interfaces::{NoAuthorization, RequestAuthorization};
use http::HeaderMap;
use std::path::PathBuf;
use std::sync::Arc;
use url::{Origin, Url};
use xt_error::Result;

///
/// ```no_run
/// # async fn example() -> xt_transfer::Result<()> {
/// use xt_transfer::{Authentication, DownloadConfig, download_file_with};
/// download_file_with("https://example.com/data.h5", "data/data.h5", DownloadConfig {
///     authorization: Authentication::Bearer { token: "your-token".into() },
///     ..Default::default()
/// }).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Default)]
pub enum Authentication {
    #[default]
    None,
    Bearer { token: String },
    Basic { username: String, password: String },
    ApiKey { header: String, value: String },
    Cookie { value: String },
    ///
    /// ```no_run
    /// # async fn example() -> xt_transfer::Result<()> {
    /// use xt_transfer::{Authentication, DownloadConfig, download_file_with};
    /// download_file_with("https://example.com/data.h5", "data/data.h5", DownloadConfig {
    ///     authorization: Authentication::Netrc { path: Some("config/.netrc".into()) },
    ///     ..Default::default()
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    Netrc { path: Option<PathBuf> },
    Custom(Arc<dyn RequestAuthorization>),
}

impl Authentication {
    pub(crate) fn resolve(&self, url: &Url) -> Result<Arc<dyn RequestAuthorization>> {
        match self {
            Self::None => Ok(Arc::new(NoAuthorization)),
            Self::Custom(auth) => Ok(auth.clone()),
            Self::Netrc { path } => Ok(Arc::new(NetrcAuthorization::load(path.as_deref(), url)?)),
            _ => Ok(Arc::new(OriginAuthorization {
                origin: url.origin(),
                headers: build_headers(self)?,
            })),
        }
    }
}

pub(super) struct OriginAuthorization {
    pub(super) origin: Origin,
    pub(super) headers: HeaderMap,
}

impl RequestAuthorization for OriginAuthorization {
    fn headers(&self, url: &Url) -> Result<HeaderMap> {
        Ok(if url.origin() == self.origin {
            self.headers.clone()
        } else {
            HeaderMap::new()
        })
    }
}
