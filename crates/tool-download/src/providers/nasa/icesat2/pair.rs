use crate::providers::nasa::icesat2::single::Icesat2Source;

pub struct Icesat2PairSource {
    pub(crate) atl03: Icesat2Source,
    pub(crate) atl08: Icesat2Source,
    pub(crate) version: u16,
}

impl Icesat2PairSource {
    pub fn new(version: &str) -> crate::Result<Self> {
        let catalog = crate::config::catalog::DownloadCatalog::load_default()?;
        Self::from_definition(catalog.source("icesat2")?, Some(version))
    }

    pub fn from_definition(
        definition: &crate::config::catalog::SourceDefinition,
        version: Option<&str>,
    ) -> crate::Result<Self> {
        let atl03 = Icesat2Source::from_definition(definition, Some("ATL03"), version)?;
        let atl08 = Icesat2Source::from_definition(definition, Some("ATL08"), version)?;
        let number = |product| -> crate::Result<u16> {
            definition
                .selection(Some(product), version)?
                .1
                .parse()
                .map_err(|_| crate::domain::invalid("Paired product release is outside the supported range"))
        };
        let version = number("ATL03")?;
        if version != number("ATL08")? {
            return Err(crate::domain::invalid(
                "ATL03/ATL08 pairing requires the same release; specify a common release",
            ));
        }
        Ok(Self {
            atl03,
            atl08,
            version,
        })
    }

    pub fn search_requests(
        &self,
        query: &crate::SearchQuery,
    ) -> crate::Result<[crate::CatalogRequest; 2]> {
        use crate::DataSource;
        Ok([
            self.atl03.search_request(query, None)?,
            self.atl08.search_request(query, None)?,
        ])
    }
}
