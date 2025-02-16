use crate::types::command_response;
use serde_json::Value;
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use warp;

pub fn copy_item(
    source: &Path,
    destination: &Path,
    recursive: Option<bool>,
    force: Option<bool>,
) -> Result<(), io::Error> {
    if !source.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The source directory / file does not exist.",
        ));
    }

    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The destination directory does not exist.",
            ));
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The destination directory does not exist.",
        ));
    }

    if source.is_file() {
        let destination = if destination.is_dir() {
            destination.join(source.file_name().unwrap())
        } else {
            destination.to_path_buf()
        };

        if destination.exists() && !force.unwrap_or(false) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Existing file cannot be overwritten without force.",
            ));
        }

        fs::copy(source, destination)?;
    } else {
        if let Some(true) = recursive {
            if !destination.exists() {
                fs::create_dir_all(destination)?;
            }

            for entry in fs::read_dir(source)? {
                let entry = entry?;
                let entry_path = entry.path();
                let mut dest_path = PathBuf::from(destination);
                dest_path.push(entry.file_name());

                copy_item(&entry_path, &dest_path, recursive, force)?;
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot copy directory without resursive parameter.",
            ));
        }
    }

    Ok(())
}

pub fn interface(parameters: &Option<Value>) -> Option<command_response::Response> {
    if let Some(params) = parameters {
        let force = params
            .get("force")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let recursive = params
            .get("recursive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if let Some(source_str) = params.get("source").and_then(Value::as_str) {
            if let Some(destination_str) = params.get("destination").and_then(Value::as_str) {
                let source = Path::new(source_str);
                let destination = Path::new(destination_str);

                match copy_item(source, destination, Some(recursive), Some(force)) {
                    Ok(()) => {
                        return Some(command_response::Response {
                            data: None,
                            status: warp::http::StatusCode::OK.into(),
                            success: true,
                            message: format!(
                                "{} was successfully copied to {}",
                                source_str, destination_str
                            )
                            .to_string(),
                            error: None,
                        })
                    }
                    Err(error) => {
                        return Some(command_response::Response {
                            data: None,
                            status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                            success: false,
                            message: format!(
                                "Failed to copy {} to {}",
                                source_str, destination_str
                            )
                            .to_string(),
                            error: Some(command_response::Error {
                                r#type: Some(command_response::ErrorType::InvalidRequest),
                                message: format!("Error copying file: {}", error).to_string(),
                            }),
                        })
                    }
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
