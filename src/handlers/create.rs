use jsonrpsee::Extensions;
use jsonrpsee::types::{ErrorObject, Params};
use log::{error, info};
use passd::models::{metadata::BaseMetadata, secret::Secret};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct CreateParams {
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
    let create_params: CreateParams = params.parse().map_err(|e| {
        error!("Failed to parse create parameters: {}", e);

        ErrorObject::owned(
            jsonrpsee::types::error::INVALID_PARAMS_CODE,
            "Invalid parameters",
            Some(format!("Failed to parse parameters: {}", e)),
        )
    })?;

    match (Secret {
        relative_path: create_params.path.into(),
    })
    .create(
        &create_params.content,
        &create_params.metadata,
        create_params.public_key.as_deref(),
    ) {
        Ok(_) => {
            info!("Create operation successful");

            Ok("Item created successfully".to_string())
        }
        Err(e) => {
            error!("Create operation failed: {}", e);

            Err(ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                "Create operation failed",
                Some(e.to_string()),
            ))
        }
    }
}
