use jsonrpsee::Extensions;
use jsonrpsee::types::{ErrorObject, Params};
use log::{error, info};
use passd::models::secret::Secret;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DeleteParams {
    path: String,
}

pub fn handler(
    params: Params,
    _ctx: &(),
    _ext: &Extensions,
) -> Result<String, ErrorObject<'static>> {
    let delete_params: DeleteParams = params.parse().map_err(|e| {
        error!("Failed to parse parameters: {}", e);

        ErrorObject::owned(
            jsonrpsee::types::error::INVALID_PARAMS_CODE,
            "Invalid parameters",
            Some(format!("Failed to parse parameters: {}", e)),
        )
    })?;

    match (Secret {
        relative_path: delete_params.path.clone().into(),
    }).remove() {
        Ok(_) => {
            info!("Successfully deleted secret {}", delete_params.path);

            Ok(format!("Successfully deleted secret {}", delete_params.path))
        }
        Err(e) => {
            error!("Failed to delete secret {}: {}", delete_params.path, e);

            Err(ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                format!("Failed to delete secret {}", delete_params.path),
                Some(e.to_string()),
            ))
        }
    }
}
