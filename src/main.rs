use anyhow::Result;
use jsonrpsee::{RpcModule, server::ServerBuilder};
use log::info;
use passd::{configs::load_config, utils::logger::init_logger};
use std::net::SocketAddr;

mod handlers;

#[tokio::main]
async fn main() -> Result<()> {
    let config = load_config().expect("Failed to load config");
    let _ = init_logger(&config.log_file, &config.log_level)
        .expect("Failed to initialize logger");
    let addr: SocketAddr = format!("127.0.0.1:{}", &config.port).parse()?;
    let mut module = RpcModule::new(());

    handlers::register_handlers(&mut module)?;

    let server = ServerBuilder::default().build(addr).await?;
    let handle = server.start(module);

    info!("Server running on {}", addr);

    handle.stopped().await;

    Ok(())
}
