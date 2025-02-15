use crate::types::command_response;
use serde_json::Value;
use std::{fs, io, path::Path};
use warp;

pub fn read_file(path: &Path) -> Result<String, io::Error> {
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The specified path is not a file or does not exist",
        ));
    }

    let content = fs::read_to_string(path)?;

    Ok(content)
}

pub fn interface(parameters: &Option<Value>) -> Option<command_response::Response> {
    if let Some(params) = parameters {
        if let Some(path_str) = params.get("path").and_then(Value::as_str) {
            let path = Path::new(path_str);

            match read_file(path) {
                Ok(content) => {
                    return Some(command_response::Response {
                        data: Some(serde_json::Value::String(content)),
                        status: warp::http::StatusCode::OK.into(),
                        success: true,
                        message: "File was successfully read".to_string(),
                        error: None,
                    })
                }
                Err(error) => {
                    return Some(command_response::Response {
                        data: None,
                        status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                        success: false,
                        message: "Failed to read file".to_string(),
                        error: Some(command_response::Error {
                            r#type: Some(command_response::ErrorType::InvalidRequest),
                            message: format!("Error reading file: {}", error).to_string(),
                        }),
                    })
                }
            }
        }
    }

    Some(command_response::Response {
        data: None,
        status: warp::http::StatusCode::BAD_REQUEST.into(),
        success: false,
        message: "Missing parameters".to_string(),
        error: Some(command_response::Error {
            r#type: Some(command_response::ErrorType::InvalidRequest),
            message: "Required parameters are missing".to_string(),
        }),
    })
}
