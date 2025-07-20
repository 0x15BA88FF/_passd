use anyhow::Result;
use jsonrpsee::RpcModule;

pub mod create;
pub mod update;

pub fn register_handlers(module: &mut RpcModule<()>) -> Result<()> {
    module.register_method("create", create::handler)?;
    module.register_method("update", update::handler)?;

    Ok(())
}
