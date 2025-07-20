use jsonrpsee::Extensions;
use jsonrpsee::types::{ErrorObject, Params};
use log::{error, info};
use passd::models::{metadata::BaseMetadata, secret::Secret};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UpdateParams {
    path: String,
    content: String,
    metadata: BaseMetadata,
    public_key: Option<String>,
}

pub fn handler(
    params: Params,
    _ctx: &(),
    _ext: &Extensions,
) -> Result<String, ErrorObject<'static>> {
    let update_params: UpdateParams = params.parse().map_err(|e| {
        error!("Failed to parse parameters: {}", e);

        ErrorObject::owned(
            jsonrpsee::types::error::INVALID_PARAMS_CODE,
            "Invalid parameters",
            Some(format!("Failed to parse parameters: {}", e)),
        )
    })?;

    match (Secret {
        relative_path: update_params.path.clone().into(),
    }).update(
        Some(&update_params.content),
        Some(&update_params.metadata),
        update_params.public_key.as_deref(),
    ) {
        Ok(_) => {
            info!("Successfully updated secret {}", update_params.path);

            Ok(format!("Successfully updated secret {}", update_params.path))
        }
        Err(e) => {
            error!("Failed to update secret {}: {}", update_params.path, e);

            Err(ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                format!("Failed to update secret {}", update_params.path),
                Some(e.to_string()),
            ))
        }
    }
}
