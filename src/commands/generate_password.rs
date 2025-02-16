use crate::types::command_response;
use rand::Rng;
use serde_json::Value;
use std::io;
use warp;

#[derive(PartialEq)]
pub enum Filter {
    Alphabets,
    Capital,
    Numbers,
    Symbols,
    Custom,
}

pub fn generate_password(
    length: Option<usize>,
    filter: Option<Vec<Filter>>,
    custom: Option<Vec<String>>,
    separators: Option<Vec<String>>,
) -> Result<String, io::Error> {
    let length = length.unwrap_or(25);
    let filters = filter.unwrap_or_else(|| {
        vec![
            Filter::Alphabets,
            Filter::Capital,
            Filter::Numbers,
            Filter::Symbols,
            Filter::Custom,
        ]
    });
    let custom = custom.unwrap_or_default();
    let separators = separators.unwrap_or_default();

    let numbers: Vec<String> = "0123456789".chars().map(|c| c.to_string()).collect();
    let lowercase: Vec<String> = "abcdefghijklmnopqrstuvwxyz"
        .chars()
        .map(|c| c.to_string())
        .collect();
    let uppercase: Vec<String> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        .chars()
        .map(|c| c.to_string())
        .collect();
    let symbols: Vec<String> = "!@#$%^&*()-_=+[]{}|;:',.<>?/`~"
        .chars()
        .map(|c| c.to_string())
        .collect();

    let mut pool: Vec<String> = Vec::new();

    if filters.contains(&Filter::Alphabets) {
        pool.extend(lowercase);
    }
    if filters.contains(&Filter::Capital) {
        pool.extend(uppercase);
    }
    if filters.contains(&Filter::Numbers) {
        pool.extend(numbers);
    }
    if filters.contains(&Filter::Symbols) {
        pool.extend(symbols);
    }
    if filters.contains(&Filter::Custom) {
        pool.extend(custom);
    }

    if pool.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "No characters available in the pool to generate a password.",
        ));
    }

    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| {
            let random_index = rng.gen_range(0..pool.len());
            pool[random_index].clone()
        })
        .collect();

    if !separators.is_empty() {
        let sep: Vec<String> = separators
            .iter()
            .flat_map(|s| s.chars().map(|c| c.to_string()))
            .collect();

        let mut password_with_separators = String::new();
        for (i, c) in password.chars().enumerate() {
            if i > 0 && i % 4 == 0 {
                password_with_separators.push_str(&sep[rng.gen_range(0..sep.len())]);
            }
            password_with_separators.push(c);
        }
        return Ok(password_with_separators);
    }

    Ok(password)
}

pub fn interface(parameters: &Option<Value>) -> Option<command_response::Response> {
    let length = parameters
        .as_ref()
        .and_then(|p| p.get("length"))
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(25);

    let filters = parameters
        .as_ref()
        .and_then(|p| p.get("filters"))
        .and_then(|f| f.as_array())
        .map(|arr| arr.iter().filter_map(|e| {
            match e.as_str() {
                Some("alphabets") => Some(Filter::Alphabets),
                Some("capital") => Some(Filter::Capital),
                Some("numbers") => Some(Filter::Numbers),
                Some("symbols") => Some(Filter::Symbols),
                Some("custom") => Some(Filter::Custom),
                _ => None,
            }
        }).collect())
        .unwrap_or_else(|| vec![
            Filter::Alphabets,
            Filter::Capital,
            Filter::Numbers,
            Filter::Symbols,
            Filter::Custom,
        ]);

    let custom = parameters
        .as_ref()
        .and_then(|p| p.get("custom"))
        .and_then(|c| c.as_array())
        .map(|arr| arr.iter().filter_map(|e| e.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default();

    let separators = parameters
        .as_ref()
        .and_then(|p| p.get("separators"))
        .and_then(|s| s.as_array())
        .map(|arr| arr.iter().filter_map(|e| e.as_str().map(|s| s.to_string())).collect::<Vec<String>>())
        .unwrap_or_default();

    match generate_password(Some(length), Some(filters), Some(custom), Some(separators)) {
        Ok(password) => {
            return Some(command_response::Response {
                data: Some(serde_json::Value::String(password)),
                status: warp::http::StatusCode::OK.into(),
                success: true,
                message: "Password was successfully generated".to_string(),
                error: None,
            })
        }
        Err(error) => {
            return Some(command_response::Response {
                data: None,
                status: warp::http::StatusCode::INTERNAL_SERVER_ERROR.into(),
                success: false,
                message: "Failed to generate password".to_string(),
                error: Some(command_response::Error {
                    r#type: Some(command_response::ErrorType::InvalidRequest),
                    message: format!("Error generating password: {}", error).to_string(),
                }),
            })
        }
    }
}
