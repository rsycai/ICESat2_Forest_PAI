use crate::domain::{PageToken, SearchPage, SearchQuery};
use crate::services::transport::{
    HttpRequest as CatalogRequest, HttpResponse as CatalogResponse,
    HttpTransport as CatalogTransport,
};
use xt_error::Result;
use xt_transfer::DownloadFuture;

pub trait DataSource: Send + Sync {
    fn id(&self) -> &str;
    fn search_request(
        &self,
        query: &SearchQuery,
        after: Option<&PageToken>,
    ) -> Result<CatalogRequest>;
    fn parse_page(&self, response: CatalogResponse) -> Result<SearchPage>;

    fn search<'a>(
        &'a self,
        transport: &'a dyn CatalogTransport,
        query: &'a SearchQuery,
        after: Option<&'a PageToken>,
    ) -> DownloadFuture<'a, SearchPage> {
        Box::pin(async move {
            let request = self.search_request(query, after)?;
            let response = transport.send(request).await?;
            self.parse_page(response)
        })
    }
}

impl<T: DataSource + ?Sized> DataSource for &T {
    fn id(&self) -> &str {
        (**self).id()
    }
    fn search_request(
        &self,
        query: &SearchQuery,
        after: Option<&PageToken>,
    ) -> Result<CatalogRequest> {
        (**self).search_request(query, after)
    }
    fn parse_page(&self, response: CatalogResponse) -> Result<SearchPage> {
        (**self).parse_page(response)
    }
    fn search<'a>(
        &'a self,
        transport: &'a dyn CatalogTransport,
        query: &'a SearchQuery,
        after: Option<&'a PageToken>,
    ) -> DownloadFuture<'a, SearchPage> {
        (**self).search(transport, query, after)
    }
}
