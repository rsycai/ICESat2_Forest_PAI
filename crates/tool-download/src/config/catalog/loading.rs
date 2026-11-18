use crate::config::catalog::schema::{AuthenticationProfile, DownloadCatalog, SourceDefinition};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use xt_error::{Error, ErrorKind, Result};

impl DownloadCatalog {
    pub fn load_default() -> Result<Self> {
        if let Some(path) = std::env::var_os("XT_DOWNLOAD_CATALOG") {
            return Self::from_path(Path::new(&path));
        }
        match File::open(DEFAULT_CATALOG_PATH) {
            Ok(file) => Self::from_reader(file),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Self::parse_xml(EMBEDDED_CATALOG)
            }
            Err(error) => Err(invalid("Cannot open download data source XML").with_source(error)),
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path.as_ref()).map_err(|error| {
            invalid(format!(
                "Cannot open download data source XML: {}",
                crate::services::storage::paths::display(path.as_ref())
            ))
            .with_source(error)
        })?;
        Self::from_reader(file)
    }

    pub(super) fn from_reader(file: File) -> Result<Self> {
        let mut xml = String::new();
        file.take(MAX_XML_BYTES + 1)
            .read_to_string(&mut xml)
            .map_err(|error| invalid("Failed to read data source XML; UTF-8 encoding required").with_source(error))?;
        Self::parse_xml(&xml)
    }

    pub fn parse_xml(xml: &str) -> Result<Self> {
        if xml.len() as u64 > MAX_XML_BYTES {
            return Err(invalid("Data source XML must not exceed 1 MiB"));
        }
        let mut reader = quick_xml::Reader::from_str(xml);
        loop {
            match reader
                .read_event()
                .map_err(|_| invalid("Malformed data source XML"))?
            {
                quick_xml::events::Event::Decl(_) | quick_xml::events::Event::Comment(_) => {}
                quick_xml::events::Event::Text(text)
                    if text.iter().all(u8::is_ascii_whitespace) => {}
                quick_xml::events::Event::Start(element)
                    if element.name().as_ref() == b"download" =>
                {
                    break;
                }
                _ => return Err(invalid("Data source XML root element must be download")),
            }
        }
        let catalog: Self = quick_xml::de::from_str(xml)
            .map_err(|_| invalid("Invalid data source XML structure: missing, duplicate, or unknown fields"))?;
        catalog.validate()?;
        Ok(catalog)
    }

    pub fn validate(&self) -> Result<()> {
        if self.sources.is_empty() {
            return Err(invalid("XML requires at least one source"));
        }
        let mut ids = BTreeSet::new();
        let mut profiles = BTreeSet::new();
        for profile in &self.authentication.entries {
            if profile.id.is_empty() || !profiles.insert(&profile.id) {
                return Err(invalid("Missing or duplicate authentication profile ID"));
            }
            let mut keys = BTreeSet::new();
            for parameter in &profile.parameters.entries {
                if parameter.name.is_empty() || !keys.insert(&parameter.name) {
                    return Err(invalid("Missing or duplicate authentication parameter name"));
                }
            }
            crate::services::registry::validate_authentication(profile)?;
        }
        for source in &self.sources {
            if !ids.insert(&source.id) {
                return Err(invalid("Duplicate XML data source ID"));
            }
            source.validate()?;
            crate::services::registry::validate_source(source)?;
            if let Some(profile) = &source.authentication {
                self.authentication(profile)?;
            }
        }
        Ok(())
    }

    pub fn authentication(&self, id: &str) -> Result<&AuthenticationProfile> {
        self.authentication
            .entries
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| invalid("Referenced authentication profile does not exist"))
    }

    pub fn source(&self, id: &str) -> Result<&SourceDefinition> {
        self.sources
            .iter()
            .find(|source| source.id == id)
            .ok_or_else(|| invalid(format!("Data source not configured in XML: {id}")))
    }
}

pub const DEFAULT_CATALOG_PATH: &str = "resource/download/sources.xml";

pub(super) const EMBEDDED_CATALOG: &str = include_str!(concat!(env!("OUT_DIR"), "/sources.xml"));

pub(super) const MAX_XML_BYTES: u64 = 1024 * 1024;

pub(super) fn invalid(message: impl Into<String>) -> Error {
    Error::new(ErrorKind::Configuration, message)
}
