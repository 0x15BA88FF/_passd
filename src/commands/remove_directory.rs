use crate::types::command_response;
use serde_json::Value;
use std::{fs, io, path::Path};
use warp;

pub fn remove_directory(path: &Path, force: Option<bool>) -> Result<(), io::Error> {
    if path.is_dir() {
        if path.read_dir()?.next().is_some() {
            if force.unwrap_or(false) {
                fs::remove_dir_all(path)?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::DirectoryNotEmpty,
                    "Non-empty directory cannot be removed without force",
                ));
            }
        } else {
            fs::remove_dir(path)?;
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The specified path is not a directory or does not exist",
        ));
    }

    Ok(())
}

pub fn interface(parameters: &Option<Value>) -> Option<command_response::Response> {
    if let Some(params) = parameters {
        let force = params
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if let Some(path_str) = params.get("path").and_then(Value::as_str) {
            let path = Path::new(path_str);

            match remove_directory(path, Some(force)) {
                Ok(()) => {
                    return Some(command_response::Response {
                        data: None,
                        status: warp::http::StatusCode::OK.into(),
                        success: true,
                        message: "Directory was successfully removed".to_string(),
                        error: None,
                    })
                }
                Err(error) => {
                    return Some(command_response::Response {
                        data: None,
                        status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                        success: false,
                        message: "Failed to remove directory".to_string(),
                        error: Some(command_response::Error {
                            r#type: Some(command_response::ErrorType::InvalidRequest),
                            message: format!("Error removing directory: {}", error).to_string(),
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
