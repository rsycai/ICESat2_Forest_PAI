use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Page {
    pub items: Vec<Item>,
    pub next: Option<String>,
}

#[derive(Deserialize)]
pub(super) struct Item {
    pub id: String,
    pub url: String,
    pub acquired_at: Option<String>,
    pub size_bytes: Option<u64>,
}
