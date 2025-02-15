use crate::types::command_response;
use serde_json::Value;
use std::{fs, io, path::Path};
use warp;

pub fn remove_file(path: &Path) -> Result<(), io::Error> {
    if path.is_file() {
        fs::remove_file(path)?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The specified path is not a file or does not exist",
        ));
    };

    Ok(())
}

pub fn interface(parameters: &Option<Value>) -> Option<command_response::Response> {
    if let Some(params) = parameters {
        if let Some(path_str) = params.get("path").and_then(Value::as_str) {
            let path = Path::new(path_str);

            match remove_file(path) {
                Ok(()) => {
                    return Some(command_response::Response {
                        data: None,
                        status: warp::http::StatusCode::OK.into(),
                        success: true,
                        message: "File was successfully removed".to_string(),
                        error: None,
                    })
                }
                Err(error) => {
                    return Some(command_response::Response {
                        data: None,
                        status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                        success: false,
                        message: "Failed to remove file".to_string(),
                        error: Some(command_response::Error {
                            r#type: Some(command_response::ErrorType::InvalidRequest),
                            message: format!("Error removing file: {}", error).to_string(),
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
