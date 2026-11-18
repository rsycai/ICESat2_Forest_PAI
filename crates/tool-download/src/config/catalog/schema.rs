use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceDefinition {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@adapter", default)]
    pub adapter: String,
    #[serde(default)]
    pub authentication: Option<String>,
    pub search_url: String,
    #[serde(default)]
    pub product_prefix: String,
    #[serde(default)]
    pub default_product: String,
    #[serde(default)]
    pub products: Products,
    #[serde(default)]
    pub parameters: Parameters,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Products {
    #[serde(rename = "product", default)]
    pub entries: Vec<ProductDefinition>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductDefinition {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@version")]
    pub version: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parameters {
    #[serde(rename = "parameter", default)]
    pub entries: Vec<FixedParameter>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FixedParameter {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DownloadCatalog {
    #[serde(default)]
    pub authentication: AuthenticationProfiles,
    #[serde(rename = "source", default)]
    pub sources: Vec<SourceDefinition>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticationProfiles {
    #[serde(rename = "profile", default)]
    pub entries: Vec<AuthenticationProfile>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticationProfile {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@adapter")]
    pub adapter: String,
    #[serde(default)]
    pub parameters: AuthenticationParameters,
}
impl AuthenticationProfile {
    pub fn value(&self, name: &str) -> Option<&str> {
        self.parameters
            .entries
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.value.as_str())
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticationParameters {
    #[serde(rename = "parameter", default)]
    pub entries: Vec<AuthenticationParameter>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthenticationParameter {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value", default)]
    pub value: SecretValue,
    #[serde(rename = "@public", default)]
    pub public: bool,
}

#[derive(Clone, Default, Deserialize)]
#[serde(transparent)]
pub struct SecretValue(pub(super) String);

impl SecretValue {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for SecretValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("[REDACTED]")
    }
}
