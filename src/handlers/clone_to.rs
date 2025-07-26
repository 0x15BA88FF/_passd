use crate::Config;
use jsonrpsee::Extensions;
use jsonrpsee::types::{ErrorObject, Params};
use log::{error, info};
use passd::models::secret::Secret;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct CloneParams {
    from_path: String,
    to_path: String,
    public_key: String,
    private_key: Option<String>,
    password: String,
}

pub fn handler(
    params: Params,
    ctx: &Arc<Config>,
    _ext: &Extensions,
) -> Result<String, ErrorObject<'static>> {
    let clone_params: CloneParams = params.parse().map_err(|e| {
        error!("Failed to parse parameters: {}", e);

        ErrorObject::owned(
            jsonrpsee::types::error::INVALID_PARAMS_CODE,
            "Invalid parameters",
            Some(format!("Failed to parse parameters: {}", e)),
        )
    })?;

    match (Secret {
        relative_path: clone_params.from_path.clone().into(),
        config: Arc::clone(ctx),
    })
    .clone_to(
        PathBuf::from(&clone_params.to_path),
        &clone_params.public_key,
        clone_params.private_key.as_deref(),
        &clone_params.password,
    ) {
        Ok(_) => {
            info!(
                "Successfully cloned secret from {} to {}",
                clone_params.from_path, clone_params.to_path
            );

            Ok(format!(
                "Successfully cloned secret from {} to {}",
                clone_params.from_path, clone_params.to_path
            ))
        }
        Err(e) => {
            error!(
                "Failed to clone secret from {} to {}: {}",
                clone_params.from_path, clone_params.to_path, e
            );

            Err(ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                format!(
                    "Failed to clone secret from {} to {}",
                    clone_params.from_path, clone_params.to_path
                ),
                Some(e.to_string()),
            ))
        }
    }
}
