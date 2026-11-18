use http::HeaderValue;
use hya_net::cookies::{CookieJar, now_secs};
use std::sync::Mutex;
use url::Url;

#[derive(Default)]
pub struct Session {
    jar: Mutex<CookieJar>,
}
impl Session {
    pub fn store(&self, url: &Url, headers: &str) {
        if let Some(host) = url.host_str() {
            self.jar
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .store_response(headers, host, url.path(), now_secs());
        }
    }
    pub fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        let value = self
            .jar
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .header_value(
                url.host_str()?,
                url.path(),
                url.scheme() == "https",
                now_secs(),
            )?;
        HeaderValue::from_str(&value).ok()
    }
}
