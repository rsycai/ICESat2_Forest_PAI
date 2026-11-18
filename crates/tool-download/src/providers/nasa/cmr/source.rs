use crate::domain::{Granule, PageToken, SearchPage, SearchQuery};
use crate::providers::nasa::cmr::headers::header;
use crate::providers::nasa::cmr::response::Document;
use crate::services::transport::{HttpRequest as CatalogRequest, HttpResponse as CatalogResponse};
use xt_error::{DataError, DownloadError, Error, ErrorKind, Result};
use xt_transfer::Asset;

pub struct CmrSource {
    pub mission: String,
    pub(super) product: String,
    pub(super) version: String,
    pub(super) endpoint: String,
    pub(super) parameters: Vec<(String, String)>,
}

impl CmrSource {
    pub fn new(mission: &str, product: &str, version: &str) -> Result<Self> {
        let catalog = crate::config::catalog::DownloadCatalog::load_default()?;
        Self::from_definition(catalog.source(mission)?, Some(product), Some(version))
    }

    pub fn from_definition(
        definition: &crate::config::catalog::SourceDefinition,
        product: Option<&str>,
        version: Option<&str>,
    ) -> Result<Self> {
        let prefix = definition.product_prefix.as_str();
        crate::providers::nasa::validate_definition(definition, prefix)?;
        let mission = definition.id.clone();
        let (product, version) = definition.selection(product, version)?;
        let prefix = &definition.product_prefix;
        if !product.starts_with(prefix)
            || product.len() == prefix.len()
            || !product
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(crate::domain::invalid("Product name does not belong to the mission or contains invalid characters"));
        }
        if version.is_empty() || !version.bytes().all(|b| b.is_ascii_digit()) {
            return Err(crate::domain::invalid(
                "Product release must be an explicit numeric string, such as 002 or 007",
            ));
        }
        Ok(Self {
            mission,
            product: product.into(),
            version: version.into(),
            endpoint: definition.search_url.clone(),
            parameters: definition
                .parameters
                .entries
                .iter()
                .map(|parameter| (parameter.name.clone(), parameter.value.clone()))
                .collect(),
        })
    }

    pub fn request(
        &self,
        query: &SearchQuery,
        after: Option<&PageToken>,
    ) -> Result<CatalogRequest> {
        if !(1..=2000).contains(&query.page_size) {
            return Err(crate::domain::invalid("page_size must be in 1..=2000"));
        }
        let mut request = CatalogRequest {
            method: Default::default(),
            body: None,
            retry_safe: false,
            endpoint: self.endpoint.clone(),
            query: vec![
                ("short_name".into(), self.product.clone()),
                ("version".into(), self.version.clone()),
                ("page_size".into(), query.page_size.to_string()),
            ],
            headers: Default::default(),
        };
        request.query.extend(self.parameters.iter().cloned());
        if let Some(time) = &query.time {
            request.query.push(("temporal".into(), time.parameter()));
        }
        if let Some(bbox) = query.bbox {
            request
                .query
                .push(("bounding_box".into(), bbox.parameter()));
        }
        if let Some(after) = after {
            if after.0.is_empty() || after.0.chars().any(char::is_control) {
                return Err(crate::domain::invalid("Pagination cursor is empty or contains control characters"));
            }
            request
                .headers
                .insert("CMR-Search-After".into(), after.0.clone());
        }
        Ok(request)
    }

    pub fn parse(&self, response: CatalogResponse) -> Result<SearchPage> {
        if response.status != 200 {
            let mut error = Error::new(
                ErrorKind::Download(DownloadError::HttpStatus(response.status)),
                format!("{} catalog search failed: HTTP {}", self.mission, response.status),
            );
            if let Some(seconds) =
                header(&response, "retry-after").and_then(|value| value.parse::<u64>().ok())
            {
                error = error.with_retry_after(std::time::Duration::from_secs(seconds));
            }
            return Err(error);
        }
        let document: Document = serde_json::from_slice(&response.body).map_err(|_| {
            Error::new(
                ErrorKind::Data(DataError::SchemaMismatch),
                "CMR response does not match the granules.json schema",
            )
        })?;
        let next = if document.feed.entry.is_empty() {
            None
        } else {
            header(&response, "cmr-search-after").map(|value| PageToken(value.into()))
        };
        let granules = document
            .feed
            .entry
            .into_iter()
            .map(|entry| {
                let mut urls = std::collections::BTreeSet::new();
                let assets = entry
                    .links
                    .into_iter()
                    .filter(|link| {
                        !link.inherited
                            && link.rel == "http://esipfed.org/ns/fedsearch/1.1/data#"
                            && link.href.starts_with("https://")
                            && !link.href.chars().any(char::is_control)
                            && !link.title.to_ascii_lowercase().contains("opendap")
                            && urls.insert(link.href.clone())
                    })
                    .map(|link| Asset {
                        url: link.href,
                        size_bytes: None,
                        checksum: None,
                    })
                    .collect();
                Granule {
                    product: self.product.clone(),
                    acquired_at: entry.time_start,
                    catalog_size_mb: entry.granule_size.and_then(|value| {
                        let text = match value {
                            serde_json::Value::String(s) => s,
                            serde_json::Value::Number(n) => n.to_string(),
                            _ => return None,
                        };
                        text.parse::<f64>()
                            .ok()
                            .filter(|n| n.is_finite() && *n >= 0.0)
                            .map(|_| text)
                    }),
                    id: entry.id,
                    title: entry.title,
                    assets,
                }
            })
            .collect();
        Ok(SearchPage { granules, next })
    }
}
