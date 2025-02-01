use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub enum ErrorType {
    NotFound,
    ValueError,
    Unauthorized,
    InternalError,
    InvalidRequest,
    ValidationError,
    Custom(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub status: u16,
    pub success: bool,
    pub message: String,
    pub data: Option<Value>,
    pub error: Option<Error>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Error {
    pub r#type: Option<ErrorType>,
    pub message: String,
}
