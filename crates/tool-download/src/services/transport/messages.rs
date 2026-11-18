use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, serde::Deserialize)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub endpoint: String,
    pub query: Vec<(String, String)>,
    pub headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Vec<u8>>,
    #[serde(skip)]
    pub retry_safe: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    #[default]
    Get,
    Head,
    Post,
    Put,
    Patch,
    Delete,
    Options,
}
impl HttpMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Options => "OPTIONS",
        }
    }
    pub fn retry_by_default(self) -> bool {
        matches!(self, Self::Get | Self::Head | Self::Options)
    }
}
impl std::fmt::Debug for HttpRequest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("HttpRequest")
            .field("method", &self.method)
            .field("query_fields", &self.query.len())
            .field("header_fields", &self.headers.len())
            .field("body_bytes", &self.body.as_ref().map(Vec::len))
            .finish()
    }
}

pub struct HttpResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}
