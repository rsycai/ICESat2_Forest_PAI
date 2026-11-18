use crate::Result;
use crate::config::catalog::SourceDefinition;
use crate::domain::invalid;
use std::collections::BTreeSet;

pub(crate) fn validate_definition(
    definition: &SourceDefinition,
    expected_prefix: &str,
) -> Result<()> {
    definition.validate()?;
    if expected_prefix.is_empty()
        || !expected_prefix
            .bytes()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
    {
        return Err(invalid("Invalid CMR product prefix"));
    }
    if definition.product_prefix != expected_prefix {
        return Err(invalid("XML product prefix does not match the mission"));
    }
    let authority = definition
        .search_url
        .strip_prefix("https://")
        .and_then(|value| value.split('/').next())
        .unwrap_or_default();
    if authority.is_empty()
        || authority.contains('@')
        || definition.search_url.chars().any(char::is_whitespace)
        || definition.search_url.contains(['?', '#'])
    {
        return Err(invalid(
            "XML search_url must be HTTPS without credentials, query, or fragment",
        ));
    }
    let mut names = BTreeSet::new();
    for product in &definition.products.entries {
        if !names.insert(&product.name)
            || !product.name.starts_with(expected_prefix)
            || product.name.len() == expected_prefix.len()
            || !product
                .name
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == b'_')
            || product.version.is_empty()
            || !product.version.bytes().all(|c| c.is_ascii_digit())
        {
            return Err(invalid("Invalid XML product name or release, or duplicate products"));
        }
    }
    if !names.contains(&definition.default_product) {
        return Err(invalid("XML default_product must be listed in products"));
    }
    let mut parameters = BTreeSet::new();
    for parameter in &definition.parameters.entries {
        if parameter.name.is_empty()
            || !parameters.insert(&parameter.name)
            || parameter.name.chars().any(char::is_control)
            || parameter.value.chars().any(char::is_control)
            || [
                "short_name",
                "version",
                "temporal",
                "bounding_box",
                "page_size",
                "page_num",
                "polygon",
                "point",
                "line",
                "collection_concept_id",
                "concept_id",
            ]
            .contains(&parameter.name.as_str())
        {
            return Err(invalid("Invalid, duplicate, or conflicting fixed XML parameters"));
        }
    }
    Ok(())
}
