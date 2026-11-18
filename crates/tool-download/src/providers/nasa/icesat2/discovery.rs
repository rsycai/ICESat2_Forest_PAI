use crate::domain::{DataGroup, DiscoveryResult, RunOptions, SearchQuery};
use crate::interfaces::Discovery;
use crate::providers::nasa::icesat2::pair::Icesat2PairSource;
use crate::services::transport::HttpTransport as CatalogTransport;
use xt_error::Result;
use xt_transfer::{CancellationToken, DownloadFuture};

impl Discovery for Icesat2PairSource {
    fn plan(&self, query: &SearchQuery) -> Result<serde_json::Value> {
        Ok(
            serde_json::json!({"mode":"matched-pairs","products":["ATL03","ATL08"],"match_fields":["acquisition","reference_ground_track","cycle","region","version"],"revision_policy":"highest-per-product","requests":self.search_requests(query)?}),
        )
    }
    fn discover<'a>(
        &'a self,
        transport: &'a dyn CatalogTransport,
        query: &'a SearchQuery,
        _options: &'a RunOptions,
        cancel: &'a CancellationToken,
    ) -> DownloadFuture<'a, DiscoveryResult> {
        Box::pin(async move {
            let selection = crate::providers::nasa::icesat2::pairing::select_pairs(
                self, transport, query, cancel,
            )
            .await?;
            Ok(selection.into_groups())
        })
    }
}

impl crate::providers::nasa::icesat2::pairing::PairSelection {
    pub fn into_groups(self) -> DiscoveryResult {
        tracing::info!(
            pairs = self.pairs.len(),
            unmatched = self.unmatched,
            unrecognized = self.unrecognized,
            "ATL03/ATL08 pair search complete"
        );
        let groups: Vec<DataGroup> = self
            .pairs
            .into_iter()
            .map(|pair| {
                let key = pair.key;
                let time = chrono::NaiveDateTime::parse_from_str(&key.acquisition, "%Y%m%d%H%M%S")
                    .ok()
                    .map(|v| {
                        v.and_utc()
                            .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    });
                DataGroup {
                    id: Some(format!(
                        "{}_{:04}{:02}{:02}_{:03}",
                        key.acquisition,
                        key.reference_ground_track,
                        key.cycle,
                        key.region,
                        key.version
                    )),
                    acquired_at: time,
                    granules: vec![pair.atl03, pair.atl08],
                }
            })
            .collect();
        DiscoveryResult {
            matched_groups: groups.len() as u64,
            groups,
        }
    }
}
