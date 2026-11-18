use http::header::HeaderMap;
use url::Url;
use xt_error::Result;

pub trait RequestAuthorization: Send + Sync {
    fn headers(&self, url: &Url) -> Result<HeaderMap>;
}

pub struct NoAuthorization;

impl RequestAuthorization for NoAuthorization {
    fn headers(&self, _: &Url) -> Result<HeaderMap> {
        Ok(HeaderMap::new())
    }
}
