use serde::Deserialize;

#[derive(Deserialize)]
pub(super) struct Document {
    pub(super) feed: Feed,
}

#[derive(Deserialize)]
pub(super) struct Feed {
    pub(super) entry: Vec<Entry>,
}

#[derive(Deserialize)]
pub(super) struct Entry {
    #[serde(default)]
    pub(super) time_start: Option<String>,
    #[serde(default)]
    pub(super) granule_size: Option<serde_json::Value>,
    pub(super) id: String,
    pub(super) title: String,
    #[serde(default)]
    pub(super) links: Vec<Link>,
}

#[derive(Deserialize)]
pub(super) struct Link {
    pub(super) href: String,
    #[serde(default)]
    pub(super) rel: String,
    #[serde(default)]
    pub(super) inherited: bool,
    #[serde(default)]
    pub(super) title: String,
}
