use crate::{types::command_response, utils};
use serde_json::Value;
use std::{fs, io, path::Path};
use warp;

pub fn initialize(store_path: &Path, pgp_keys: Vec<&str>) -> Result<(), io::Error> {
    let gpg_id_path = store_path.join(".gpg-id");

    fs::create_dir_all(store_path)?;
    fs::write(&gpg_id_path, pgp_keys.join("\n"))?;

    Ok(())
}

pub fn interface(parameters: &Option<Value>) -> Option<command_response::Response> {
    if let Some(params) = parameters {
        let store_path_str = params
            .get("directory")
            .and_then(|v| v.as_str())
            .unwrap_or("~/.password-store/");

        let fingerprints = match params.get("fingerprints").and_then(Value::as_array) {
            Some(ids) if !ids.is_empty() => ids,
            Some(_) => {
                return Some(command_response::Response {
                    data: None,
                    status: warp::http::StatusCode::BAD_REQUEST.into(),
                    success: false,
                    message: "Fingerprints array must contain at least one string".to_string(),
                    error: Some(command_response::Error {
                        r#type: Some(command_response::ErrorType::InvalidRequest),
                        message: "Empty fingerprints array".to_string(),
                    }),
                })
            }
            None => {
                return Some(command_response::Response {
                    data: None,
                    status: warp::http::StatusCode::BAD_REQUEST.into(),
                    success: false,
                    message: "Fingerprints parameter is required and must be an array".to_string(),
                    error: Some(command_response::Error {
                        r#type: Some(command_response::ErrorType::InvalidRequest),
                        message: "Missing or invalid fingerprints parameter".to_string(),
                    }),
                })
            }
        };

        let pgp_keys: Vec<&str> = fingerprints.iter().filter_map(|v| v.as_str()).collect();

        let store_path = match utils::expand_path_str(&store_path_str) {
            Ok(path) => path,
            Err(error) => {
                return Some(command_response::Response {
                    data: None,
                    status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                    success: false,
                    message: "Failed to resolve store path".to_string(),
                    error: Some(command_response::Error {
                        r#type: Some(command_response::ErrorType::InvalidRequest),
                        message: format!("Error resolving store path: {}", error).to_string(),
                    }),
                })
            }
        };

        match initialize(&store_path, pgp_keys) {
            Ok(()) => {
                return Some(command_response::Response {
                    data: None,
                    status: warp::http::StatusCode::OK.into(),
                    success: true,
                    message: "Store was successfully initialized".to_string(),
                    error: None,
                })
            }
            Err(error) => {
                return Some(command_response::Response {
                    data: None,
                    status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                    success: false,
                    message: "Failed to initialize store".to_string(),
                    error: Some(command_response::Error {
                        r#type: Some(command_response::ErrorType::InvalidRequest),
                        message: format!("Error initializing password store: {}", error)
                            .to_string(),
                    }),
                })
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
