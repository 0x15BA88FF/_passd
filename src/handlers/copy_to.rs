use crate::Config;
use jsonrpsee::Extensions;
use jsonrpsee::types::{ErrorObject, Params};
use log::{error, info};
use passd::models::secret::Secret;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct CopyParams {
    from_path: String,
    to_path: String,
}

pub fn handler(
    params: Params,
    ctx: &Arc<Config>,
    _ext: &Extensions,
) -> Result<String, ErrorObject<'static>> {
    let copy_params: CopyParams = params.parse().map_err(|e| {
        error!("Failed to parse parameters: {}", e);

        ErrorObject::owned(
            jsonrpsee::types::error::INVALID_PARAMS_CODE,
            "Invalid parameters",
            Some(format!("Failed to parse parameters: {}", e)),
        )
    })?;

    match (Secret {
        relative_path: copy_params.from_path.clone().into(),
        config: Arc::clone(ctx),
    })
    .copy_to(copy_params.to_path.clone().into())
    {
        Ok(_) => {
            info!(
                "Successfully copyd secret from {} to {}",
                copy_params.from_path, copy_params.to_path
            );

            Ok(format!(
                "Successfully copyd secret from {} to {}",
                copy_params.from_path, copy_params.to_path
            ))
        }
        Err(e) => {
            error!(
                "Failed to copy secret from {} to {}: {}",
                copy_params.from_path, copy_params.to_path, e
            );

            Err(ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                format!(
                    "Failed to copy secret from {} to {}",
                    copy_params.from_path, copy_params.to_path
                ),
                Some(e.to_string()),
            ))
        }
    }
}
