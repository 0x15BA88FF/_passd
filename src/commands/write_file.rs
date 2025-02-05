use crate::types::command_response;
use serde_json::Value;
use std::{fs, io, path::Path};
use warp;

pub fn write_file(path: &Path, content: &str) -> Result<(), io::Error> {
    if path.exists() && !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The specified path exists but is not a file",
        ));
    }

    fs::write(path, content)?;

    Ok(())
}

pub fn interface(parameters: &Option<Value>) -> Option<command_response::Response> {
    if let Some(params) = parameters {
        let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");

        if let Some(path_str) = params.get("path").and_then(Value::as_str) {
            let path = Path::new(path_str);

            match write_file(path, &content) {
                Ok(()) => {
                    return Some(command_response::Response {
                        data: None,
                        status: warp::http::StatusCode::OK.into(),
                        success: true,
                        message: "File was successfully written".to_string(),
                        error: None,
                    })
                }
                Err(error) => {
                    return Some(command_response::Response {
                        data: None,
                        status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                        success: false,
                        message: "Failed to write file".to_string(),
                        error: Some(command_response::Error {
                            r#type: Some(command_response::ErrorType::InvalidRequest),
                            message: format!("Error writing file: {}", error).to_string(),
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
