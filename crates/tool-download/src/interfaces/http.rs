use crate::services::transport::{HttpRequest, HttpResponse};
use xt_transfer::DownloadFuture;

pub trait HttpTransport: Send + Sync {
    fn send(&self, request: HttpRequest) -> DownloadFuture<'_, HttpResponse>;
}
