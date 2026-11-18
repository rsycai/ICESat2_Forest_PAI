use crate::processing::config::Icesat2Config;
use crate::model::ProcessRequest;

#[derive(Debug, Clone)]
pub struct ProcessParams {
    pub request: ProcessRequest,
    pub options: Icesat2Config,
}
