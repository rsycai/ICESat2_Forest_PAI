use reqwest::Url;
use reqwest::cookie::CookieStore;
use reqwest::header::HeaderValue;
use std::sync::Arc;
use xt_transfer::Session;

pub(super) struct CatalogCookies(pub(super) Arc<Session>);
impl CookieStore for CatalogCookies {
    fn set_cookies(&self, cookies: &mut dyn Iterator<Item = &HeaderValue>, url: &Url) {
        for cookie in cookies {
            if let Ok(value) = cookie.to_str() {
                self.0.store(
                    url,
                    &format!("HTTP/1.1 200 OK\r\nSet-Cookie: {value}\r\n\r\n"),
                );
            }
        }
    }
    fn cookies(&self, url: &Url) -> Option<HeaderValue> {
        self.0.cookies(url)
    }
}
