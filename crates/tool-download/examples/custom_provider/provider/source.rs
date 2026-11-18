use crate::provider::response::Page;
use tool_download::Asset;
use tool_download::CatalogRequest;
use tool_download::CatalogResponse;
use tool_download::DataSource;
use tool_download::Error;
use tool_download::ErrorKind;
use tool_download::Granule;
use tool_download::PageToken;
use tool_download::Result;
use tool_download::SearchPage;
use tool_download::SearchQuery;

pub(super) struct JsonSource {
    pub id: String,
    pub endpoint: String,
    pub product: String,
    pub release: String,
}
impl DataSource for JsonSource {
    fn id(&self) -> &str {
        &self.id
    }
    fn search_request(
        &self,
        query: &SearchQuery,
        after: Option<&PageToken>,
    ) -> Result<CatalogRequest> {
        let body = serde_json::json!({"product":self.product,"version":self.release,
            "bbox":query.bbox.map(|b| b.coordinates()),
            "time":query.time.as_ref().map(|t| t.parameter()),"page_size":query.page_size,"cursor":after.map(|v| &v.0)});
        Ok(CatalogRequest {
            method: tool_download::services::transport::HttpMethod::Post,
            endpoint: self.endpoint.clone(),
            headers: [("content-type".into(), "application/json".into())].into(),
            body: Some(
                serde_json::to_vec(&body)
                    .map_err(|_| Error::new(ErrorKind::Internal, "Failed to encode query"))?,
            ),
            retry_safe: true,
            ..Default::default()
        })
    }
    fn parse_page(&self, response: CatalogResponse) -> Result<SearchPage> {
        let page: Page = serde_json::from_slice(&response.body).map_err(|_| {
            Error::new(
                ErrorKind::Data(xt_error::DataError::SchemaMismatch),
                "Invalid example JSON catalog response",
            )
        })?;
        Ok(SearchPage {
            next: page.next.map(PageToken),
            granules: page
                .items
                .into_iter()
                .map(|item| Granule {
                    product: self.product.clone(),
                    title: item.id.clone(),
                    id: item.id,
                    acquired_at: item.acquired_at,
                    catalog_size_mb: None,
                    assets: vec![Asset {
                        url: item.url,
                        size_bytes: item.size_bytes,
                        checksum: None,
                    }],
                })
                .collect(),
        })
    }
}
