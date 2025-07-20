use anyhow::Result;
use jsonrpsee::RpcModule;

pub mod create;

pub fn register_handlers(module: &mut RpcModule<()>) -> Result<()> {
    module.register_method("create", create::handler)?;

    Ok(())
}
