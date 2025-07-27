use crate::Config;
use anyhow::Result;
use jsonrpsee::RpcModule;
use std::sync::Arc;

pub mod clone_to;
pub mod copy_to;
pub mod create;
pub mod delete;
pub mod find;
pub mod move_to;
pub mod read;
pub mod update;

macro_rules! register {
    ($module:ident, {
        $($name:literal => $handler:path),* $(,)?
    }) => {
        $($module.register_method($name, $handler)?;)*
    };
}

pub fn register_handlers(module: &mut RpcModule<Arc<Config>>) -> Result<()> {
    register!(module, {
        "create" => create::handler,
        "update" => update::handler,
        "delete" => delete::handler,
        "read"   => read::handler,
        "move"   => move_to::handler,
        "copy"   => copy_to::handler,
        "clone"  => clone_to::handler,
        "find"   => find::handler,
    });

    Ok(())
}
