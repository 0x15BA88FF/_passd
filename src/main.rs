use anyhow::{Context, Result};
use jsonrpsee::{RpcModule, server::ServerBuilder};
use log::info;
use passd::{models::config::Config, utils::logger::init_logger};
use std::{net::SocketAddr, sync::Arc};

mod handlers;

#[tokio::main]
async fn main() -> Result<()> {
    let config =
        Config::load_config().context("Failed to load configuration")?;

    init_logger(&config.log_file, &config.log_level)
        .context("Failed to initialize logger")?;

    let addr = SocketAddr::new(config.address, config.port);

    let mut module = RpcModule::new(Arc::new(config));

    handlers::register_handlers(&mut module)
        .context("Failed to register handlers")?;

    let server = ServerBuilder::default()
        .build(addr)
        .await
        .context("Failed to build server")?;

    let handle = server.start(module);

    info!("Server running on {}", addr);

    handle.stopped().await;
    Ok(())
}
