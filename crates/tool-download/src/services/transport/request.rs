use crate::services::transport::client::DownloadClient;
use crate::services::transport::{HttpRequest, HttpResponse, HttpTransport};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use xt_error::{DownloadError, Error, ErrorKind, Result, RetryAdvice};
use xt_transfer::DownloadFuture;

impl HttpTransport for DownloadClient {
    fn send(&self, request: HttpRequest) -> DownloadFuture<'_, HttpResponse> {
        Box::pin(async move {
            let mut url = self.url(&request.endpoint)?;
            if !request.query.is_empty() {
                url.query_pairs_mut().extend_pairs(request.query);
            }
            let mut headers = HeaderMap::new();
            for (name, value) in request.headers {
                let name = HeaderName::from_bytes(name.as_bytes())
                    .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid header name"))?;
                if ["authorization", "cookie", "host"].contains(&name.as_str()) {
                    return Err(Error::new(
                        ErrorKind::InvalidArgument,
                        "Authentication headers must be supplied by the authentication adapter",
                    ));
                }
                headers.insert(
                    name,
                    HeaderValue::from_str(&value)
                        .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Invalid header value"))?,
                );
            }
            for attempt in 0..=self.requests.retries {
                let result = async {
                    let mut response = self.request(request.method, url.clone(), headers.clone(), request.body.clone()).await?;
                    let status = response.status().as_u16();
                    if status >= 400 { return Err(status_error(&response)); }
                    let response_headers = response.headers().iter().filter_map(|(k,v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string()))).collect();
                    let mut body = Vec::new();
                    loop {
                        let chunk = tokio::select! { _ = self.cancel.cancelled() => return Err(cancelled()),
                            chunk = response.chunk() => chunk.map_err(network_error)? };
                        let Some(chunk) = chunk else { break; };
                        if body.len() + chunk.len() > 32 * 1024 * 1024 { return Err(Error::new(ErrorKind::ResourceExhausted, "Catalog response exceeds the 32 MiB limit")); }
                        body.extend_from_slice(&chunk);
                    }
                    Ok(HttpResponse { status, headers: response_headers, body })
                }.await;
                match result {
                    Ok(value) => return Ok(value),
                    Err(error) => {
                        if (!request.method.retry_by_default() && !request.retry_safe)
                            || !self.retry_request(&error, attempt).await?
                        {
                            return Err(error);
                        }
                    }
                }
            }
            unreachable!()
        })
    }
}

impl DownloadClient {
    pub(crate) async fn retry_request(&self, error: &Error, attempt: u8) -> Result<bool> {
        if attempt >= self.requests.retries {
            return Ok(false);
        }
        let delay = match error.retry_advice() {
            RetryAdvice::DoNotRetry => return Ok(false),
            RetryAdvice::After(delay) => delay,
            RetryAdvice::Backoff => hya_net::polite::backoff_with_jitter(
                u32::from(attempt),
                Duration::from_secs(1),
                self.requests.max_retry_delay,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            ),
        };
        if delay > self.requests.max_retry_delay {
            return Ok(false);
        }
        tokio::select! { _ = self.cancel.cancelled() => Err(cancelled()),
        _ = tokio::time::sleep(delay) => Ok(true) }
    }
}

pub(crate) fn network_error(error: reqwest::Error) -> Error {
    let kind = if error.is_timeout() {
        DownloadError::Timeout
    } else if let Some(status) = error.status() {
        DownloadError::HttpStatus(status.as_u16())
    } else if error.is_connect() {
        DownloadError::Connection
    } else {
        DownloadError::Interrupted
    };
    Error::new(ErrorKind::Download(kind), "HTTP request or transfer failed").with_source(error.without_url())
}

pub(crate) fn status_error(response: &reqwest::Response) -> Error {
    let mut error = Error::new(
        ErrorKind::Download(DownloadError::HttpStatus(response.status().as_u16())),
        format!("Server returned HTTP {}", response.status().as_u16()),
    );
    if let Some(value) = response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
    {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let duration = hya_net::polite::parse_retry_after(value, now);
        if let Some(delay) = duration {
            error = error.with_retry_after(delay);
        }
    }
    error
}

pub(super) fn cancelled() -> Error {
    Error::new(ErrorKind::Cancelled, "Request cancelled")
}
