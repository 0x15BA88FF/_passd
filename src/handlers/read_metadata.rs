use crate::Config;
use jsonrpsee::Extensions;
use jsonrpsee::types::{ErrorObject, Params};
use log::{error, info};
use passd::models::{metadata::Metadata, secret::Secret};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize)]
pub struct ReadResponse {
    metadata: Metadata,
}

#[derive(Debug, Deserialize)]
struct ReadParams {
    path: String,
}

pub fn handler(
    params: Params,
    ctx: &Arc<Config>,
    _ext: &Extensions,
) -> Result<ReadResponse, ErrorObject<'static>> {
    let read_params: ReadParams = params.parse().map_err(|e| {
        error!("Failed to parse parameters: {}", e);

        ErrorObject::owned(
            jsonrpsee::types::error::INVALID_PARAMS_CODE,
            "Invalid parameters",
            Some(format!("Failed to parse parameters: {}", e)),
        )
    })?;

    let secret = Secret {
        relative_path: read_params.path.clone().into(),
        config: Arc::clone(ctx),
    };

    let metadata = match secret.metadata() {
        Ok(metadata) => {
            info!("Successfully read secret metadata {}", read_params.path);

            metadata
        }
        Err(e) => {
            error!(
                "Failed to read secret metadata {}: {}",
                read_params.path, e
            );

            return Err(ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                format!("Failed to read secret metadata {}", read_params.path),
                Some(e.to_string()),
            ));
        }
    };

    Ok(ReadResponse { metadata })
}
