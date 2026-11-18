use url::Url;

pub(crate) struct RemoteIdentity {
    pub(crate) key: String,
    pub(crate) size: Option<u64>,
    pub(crate) etag: Option<String>,
    pub(crate) modified: Option<String>,
}

pub(crate) struct Probe {
    pub(crate) url: Url,
    pub(crate) identity: RemoteIdentity,
    pub(crate) ranges: bool,
}
