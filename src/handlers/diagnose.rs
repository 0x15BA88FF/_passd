use crate::Config;
use jsonrpsee::{
    Extensions,
    types::{ErrorObject, Params},
};
use passd::models::secret_manager::Secrets;
use serde_json::Value;
use std::sync::Arc;

pub fn handler(
    _params: Params,
    ctx: &Arc<Config>,
    _ext: &Extensions,
) -> Result<Value, ErrorObject<'static>> {
    match (Secrets {
        config: Arc::clone(ctx),
    }
    .diagnose())
    {
        Ok(diagnostics) => serde_json::to_value(diagnostics).map_err(|e| {
            ErrorObject::owned(
                jsonrpsee::types::error::INTERNAL_ERROR_CODE,
                "Failed to serialize diagnostics",
                Some(e.to_string()),
            )
        }),
        Err(e) => Err(ErrorObject::owned(
            jsonrpsee::types::error::INTERNAL_ERROR_CODE,
            format!("Unexpected diagnostics error {e}"),
            Some(e.to_string()),
        )),
    }
}
