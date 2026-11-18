use crate::config::catalog::loading::invalid;
use crate::config::catalog::schema::SourceDefinition;
use std::collections::BTreeSet;
use xt_error::Result;

impl SourceDefinition {
    pub fn validate(&self) -> Result<()> {
        let valid_id = |value: &str| {
            !value.is_empty()
                && value
                    .bytes()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c))
                && value.as_bytes()[0].is_ascii_alphanumeric()
        };
        if !valid_id(&self.id) || !valid_id(self.adapter_id()) {
            return Err(invalid("Invalid data source or adapter identifier"));
        }
        let url = reqwest::Url::parse(&self.search_url).map_err(|_| invalid("Invalid search URL"))?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
        {
            return Err(invalid("Search URL must be HTTPS without credentials or fragments"));
        }
        let mut names = BTreeSet::new();
        for product in &self.products.entries {
            if product.name.is_empty()
                || product.version.is_empty()
                || product.name.chars().any(char::is_control)
                || product.version.chars().any(char::is_control)
                || !names.insert(&product.name)
            {
                return Err(invalid("Invalid or duplicate product name or release"));
            }
        }
        if !self.default_product.is_empty() && !names.contains(&self.default_product) {
            return Err(invalid("Default product is not listed in products"));
        }
        let mut keys = BTreeSet::new();
        for parameter in &self.parameters.entries {
            if parameter.name.is_empty()
                || parameter.name.chars().any(char::is_control)
                || parameter.value.chars().any(char::is_control)
                || !keys.insert(&parameter.name)
            {
                return Err(invalid("Invalid or duplicate fixed parameters"));
            }
        }
        Ok(())
    }
    pub fn adapter_id(&self) -> &str {
        if self.adapter.is_empty() {
            &self.id
        } else {
            &self.adapter
        }
    }
    pub fn selection<'a>(
        &'a self,
        product: Option<&'a str>,
        version: Option<&'a str>,
    ) -> Result<(&'a str, &'a str)> {
        let product = product.unwrap_or(&self.default_product);
        let version = match version {
            Some(version) => version,
            None => {
                &self
                    .products
                    .entries
                    .iter()
                    .find(|entry| entry.name == product)
                    .ok_or_else(|| invalid("Selected product has no default release; specify one explicitly"))?
                    .version
            }
        };
        Ok((product, version))
    }
}
