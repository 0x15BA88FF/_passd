use crate::types::command_response;
use serde::{Serialize};
use serde_json::Value;
use std::{fs, io, path::{Path, PathBuf}};
use warp;

#[derive(Debug, Serialize)]
pub struct EntryInfo {
    pub name: String,
    pub path: PathBuf,
    pub r#type: EntryType,
    pub children: Option<Vec<EntryInfo>>,
}

#[derive(PartialEq, Debug, Serialize)]
pub enum EntryType {
    File,
    Directory,
}

pub fn list_items(path: &Path, recursive: Option<bool>) -> Result<Vec<EntryInfo>, io::Error> {
    let mut items = Vec::new();
    let recursive = recursive.unwrap_or(false);

    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provided path is not a directory",
        ));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let entry_type = if metadata.is_dir() {
            EntryType::Directory
        } else {
            EntryType::File
        };

        let children = if recursive && entry_type == EntryType::Directory {
            Some(list_items(&entry.path(), Some(true))?)
        } else {
            None
        };

        items.push(EntryInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path(),
            r#type: entry_type,
            children,
        });
    }

    Ok(items)
}

pub fn interface(parameters: &Option<Value>) -> Option<command_response::Response> {
    if let Some(params) = parameters {
        if let Some(path_str) = params.get("path").and_then(Value::as_str) {
            let path = Path::new(path_str);

            let recursive = params
                .get("recursive")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            match list_items(path, Some(recursive)) {
                Ok(items) => {
                    let content = serde_json::to_value(items).unwrap_or(Value::Null);

                    return Some(command_response::Response {
                        data: Some(content),
                        status: warp::http::StatusCode::OK.into(),
                        success: true,
                        message: "Items were successfully listed".to_string(),
                        error: None,
                    })
                }
                Err(error) => {
                    return Some(command_response::Response {
                        data: None,
                        status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                        success: false,
                        message: "Failed to list path items".to_string(),
                        error: Some(command_response::Error {
                            r#type: Some(command_response::ErrorType::InvalidRequest),
                            message: format!("Error listing: {}", error).to_string(),
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
