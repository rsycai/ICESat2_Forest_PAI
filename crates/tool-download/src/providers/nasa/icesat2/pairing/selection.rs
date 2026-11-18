use crate::domain::SearchQuery;
use crate::interfaces::DataSource;
use crate::providers::nasa::icesat2::pairing::identity::candidate;
use crate::providers::nasa::icesat2::pairing::pairs::{
    Candidate, Icesat2MatchKey, MatchedPair, PairSelection,
};
use crate::services::transport::HttpTransport as CatalogTransport;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use xt_error::{Error, ErrorKind, Result};
use xt_transfer::CancellationToken;

pub(super) async fn index(
    source: &dyn DataSource,
    product: &str,
    version: u16,
    transport: &dyn CatalogTransport,
    query: &SearchQuery,
    cancel: &CancellationToken,
) -> Result<(BTreeMap<Icesat2MatchKey, Candidate>, usize)> {
    let mut index = BTreeMap::<Icesat2MatchKey, Candidate>::new();
    let mut after = None;
    let mut cursors = HashSet::new();
    let mut unrecognized = 0;
    loop {
        let page = tokio::select! {
            biased;
            _ = cancel.cancelled() => return Err(Error::new(ErrorKind::Cancelled, "Pair search cancelled")),
            result = source.search(transport, query, after.as_ref()) => result?,
        };
        for granule in page.granules {
            let Some((key, incoming)) = candidate(granule, product) else {
                unrecognized += 1;
                continue;
            };
            if key.version != version {
                unrecognized += 1;
                continue;
            }
            insert_candidate(&mut index, key, incoming);
        }
        match page.next {
            None => break,
            Some(next) => {
                if !cursors.insert(next.0.clone()) {
                    return Err(Error::new(
                        ErrorKind::Data(xt_error::DataError::Corrupt),
                        "Pair catalog returned a repeated pagination cursor; stopping search",
                    ));
                }
                after = Some(next);
            }
        }
    }
    Ok((index, unrecognized))
}

type Candidates = BTreeMap<Icesat2MatchKey, Candidate>;

fn insert_candidate(index: &mut Candidates, key: Icesat2MatchKey, incoming: Candidate) {
    if index.get(&key).is_none_or(|old| {
        incoming.revision > old.revision
            || (incoming.revision == old.revision && incoming.granule.id < old.granule.id)
    }) {
        index.insert(key, incoming);
    }
}

#[derive(Default)]
pub struct PairCursor {
    atl03: Candidates,
    atl08: Candidates,
    submitted: BTreeSet<Icesat2MatchKey>,
}

impl PairCursor {
    pub async fn select(
        &mut self,
        source: &crate::providers::nasa::Icesat2PairSource,
        transport: &dyn CatalogTransport,
        query: &SearchQuery,
        cancel: &CancellationToken,
    ) -> Result<PairSelection> {
        let (atl03, skipped03) = index(
            &source.atl03,
            "ATL03",
            source.version,
            transport,
            query,
            cancel,
        )
        .await?;
        let (atl08, skipped08) = index(
            &source.atl08,
            "ATL08",
            source.version,
            transport,
            query,
            cancel,
        )
        .await?;
        let mut pairs = Vec::new();
        let keys = atl03
            .keys()
            .chain(atl08.keys())
            .cloned()
            .collect::<BTreeSet<_>>();
        for (index, batch) in [(&mut self.atl03, atl03), (&mut self.atl08, atl08)] {
            for (key, item) in batch {
                if !self.submitted.contains(&key) {
                    insert_candidate(index, key, item);
                }
            }
        }
        let mut unmatched = 0;
        for key in &keys {
            if self.atl03.contains_key(key) && self.atl08.contains_key(key) {
                let first = self.atl03.remove(key).unwrap();
                let second = self.atl08.remove(key).unwrap();
                self.submitted.insert(key.clone());
                pairs.push(MatchedPair {
                    key: key.clone(),
                    atl03: first.granule,
                    atl08: second.granule,
                });
            } else if !self.submitted.contains(key) {
                unmatched += 1;
            }
        }
        self.atl03.retain(|key, _| keys.contains(key));
        self.atl08.retain(|key, _| keys.contains(key));
        self.submitted.retain(|key| keys.contains(key));
        Ok(PairSelection {
            unmatched,
            pairs,
            unrecognized: skipped03 + skipped08,
        })
    }
}

pub async fn select_pairs(
    source: &crate::providers::nasa::Icesat2PairSource,
    transport: &dyn CatalogTransport,
    query: &SearchQuery,
    cancel: &CancellationToken,
) -> Result<PairSelection> {
    PairCursor::default()
        .select(source, transport, query, cancel)
        .await
}
