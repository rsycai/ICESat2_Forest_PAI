use crate::config::catalog::DownloadCatalog;
use crate::services::registry;
use crate::services::transport::DownloadClient;
use crate::{Result, SourceAdapter};
use std::sync::Arc;
use xt_transfer::{CancellationToken, NoAuthorization, TransferSettings};

pub async fn authenticated_downloader(
    catalog: &DownloadCatalog,
    source: &SourceAdapter,
    settings: TransferSettings,
    requests: crate::services::transport::RequestSettings,
    cancel: CancellationToken,
) -> Result<DownloadClient> {
    authenticated_profile(
        catalog,
        source.authentication.as_deref(),
        settings,
        requests,
        cancel,
    )
    .await
}

pub async fn authenticated_profile(
    catalog: &DownloadCatalog,
    profile: Option<&str>,
    settings: TransferSettings,
    requests: crate::services::transport::RequestSettings,
    cancel: CancellationToken,
) -> Result<DownloadClient> {
    let transport = DownloadClient::new(settings, Arc::new(NoAuthorization), cancel.clone())?
        .with_requests(requests)?;
    let Some(id) = profile else {
        return Ok(transport);
    };
    let flow = registry::authentication(catalog.authentication(id)?)?;
    let authorization = tokio::select! {
        _ = cancel.cancelled() => return Err(crate::Error::new(crate::ErrorKind::Cancelled, "Authentication cancelled")),
        result = flow.authorize(&transport) => result?,
    };
    Ok(transport.with_authorization(authorization))
}
