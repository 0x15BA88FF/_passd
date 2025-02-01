use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub command: String,
    pub parameters: Option<Value>,
    pub metadata: Option<RequestMetadata>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestMetadata {
    pub request_id: String,
    pub timestamp: String,
}
