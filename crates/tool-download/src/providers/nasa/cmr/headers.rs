use crate::services::transport::HttpResponse as CatalogResponse;

pub(super) fn header<'a>(response: &'a CatalogResponse, name: &str) -> Option<&'a str> {
    response
        .headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case(name))
        .map(|(_, value)| value.as_str())
}
