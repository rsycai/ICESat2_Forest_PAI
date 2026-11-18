use crate::Granule;

pub struct DataGroup {
    pub id: Option<String>,
    pub acquired_at: Option<String>,
    pub granules: Vec<Granule>,
}

pub struct DiscoveryResult {
    pub groups: Vec<DataGroup>,
    pub matched_groups: u64,
}

pub struct GroupIdentity {
    pub id: String,
    pub data_type: String,
    pub acquired_at: Option<String>,
    pub paired: bool,
}
