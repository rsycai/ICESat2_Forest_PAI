use crate::domain::{PageToken, SearchPage, SearchQuery};
use crate::interfaces::DataSource;
use crate::providers::nasa::cmr::source::CmrSource;
use crate::services::transport::{HttpRequest as CatalogRequest, HttpResponse as CatalogResponse};
use xt_error::Result;

impl DataSource for CmrSource {
    fn id(&self) -> &str {
        &self.mission
    }
    fn search_request(
        &self,
        query: &SearchQuery,
        after: Option<&PageToken>,
    ) -> Result<CatalogRequest> {
        self.request(query, after)
    }
    fn parse_page(&self, response: CatalogResponse) -> Result<SearchPage> {
        self.parse(response)
    }
}
