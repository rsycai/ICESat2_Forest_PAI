macro_rules! impl_source {
    ($name:ident) => {
        impl crate::DataSource for $name {
            fn id(&self) -> &str {
                self.0.mission.as_str()
            }
            fn search_request(
                &self,
                query: &crate::SearchQuery,
                after: Option<&crate::PageToken>,
            ) -> crate::Result<crate::CatalogRequest> {
                self.0.request(query, after)
            }
            fn parse_page(
                &self,
                response: crate::CatalogResponse,
            ) -> crate::Result<crate::SearchPage> {
                self.0.parse(response)
            }
        }
    };
}

pub(in crate::providers::nasa) use impl_source;
