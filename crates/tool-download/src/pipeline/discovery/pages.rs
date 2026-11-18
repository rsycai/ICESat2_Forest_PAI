use crate::domain::{DataGroup, DiscoveryResult, Granule, RunOptions, SearchQuery};
use crate::interfaces::DataSource;
use crate::services::transport::HttpTransport as CatalogTransport;
use std::collections::{HashMap, HashSet};
use xt_error::{Error, ErrorKind, Result};
use xt_transfer::CancellationToken;

pub(super) async fn discover_pages(
    source: &dyn DataSource,
    transport: &dyn CatalogTransport,
    query: &SearchQuery,
    options: &RunOptions,
    cancel: &CancellationToken,
) -> Result<DiscoveryResult> {
    options.validate()?;
    let mut cursor = None;
    let mut cursors = HashSet::new();
    let mut urls = HashSet::new();
    let mut indices = HashMap::<String, usize>::new();
    let mut granules = Vec::<Granule>::new();
    loop {
        let page = tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(Error::new(ErrorKind::Cancelled, "Data discovery cancelled")),
            page = source.search(transport, query, cursor.as_ref()) => page?,
        };
        for mut granule in page.granules {
            let mut assets = Vec::new();
            for asset in granule.assets {
                if urls.insert(xt_transfer::asset_key(&asset.url)?) {
                    assets.push(asset);
                }
            }
            if assets.is_empty() {
                continue;
            }
            if let Some(index) = indices.get(&granule.id) {
                granules[*index].assets.extend(assets);
            } else {
                granule.assets = assets;
                indices.insert(granule.id.clone(), granules.len());
                granules.push(granule);
            }
        }
        match page.next {
            None => break,
            Some(next) => {
                if !cursors.insert(next.0.clone()) {
                    return Err(Error::new(
                        ErrorKind::Data(xt_error::DataError::Corrupt),
                        "Catalog returned a repeated pagination cursor; stopping search",
                    ));
                }
                cursor = Some(next);
            }
        }
    }
    let groups = granules
        .into_iter()
        .map(|g| DataGroup {
            id: None,
            acquired_at: None,
            granules: vec![g],
        })
        .collect();
    Ok(DiscoveryResult {
        groups,
        matched_groups: 0,
    })
}
