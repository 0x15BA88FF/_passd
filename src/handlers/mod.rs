use anyhow::Result;
use jsonrpsee::RpcModule;

pub mod clone_to;
pub mod copy_to;
pub mod create;
pub mod delete;
pub mod move_to;
pub mod read;
pub mod update;

pub fn register_handlers(module: &mut RpcModule<()>) -> Result<()> {
    module.register_method("create", create::handler)?;
    module.register_method("update", update::handler)?;
    module.register_method("delete", delete::handler)?;
    module.register_method("read", read::handler)?;
    module.register_method("move", move_to::handler)?;
    module.register_method("copy", copy_to::handler)?;
    module.register_method("clone", clone_to::handler)?;

    Ok(())
}
