use crate::models::config::Config;
use config::{Config as RawConfig, File};
use directories::BaseDirs;
use dirs;
use log::{error, info, warn};
use std::{env, path::PathBuf};

pub fn resolve_config_path() -> Option<PathBuf> {
    [
        env::var("PASSD_CONFIG_DIR")
            .ok()
            .map(|dir: String| PathBuf::from(dir).join("config.toml")),
        BaseDirs::new()
            .map(|base: BaseDirs| base.config_dir().join("passd/config.toml")),
        dirs::home_dir().map(|home: PathBuf| home.join(".passd/config.toml")),
    ]
    .into_iter()
    .flatten()
    .find(|path| path.exists())
}

pub fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let Some(config_path) = resolve_config_path() else {
        warn!("No config file resolved, using default configuration");
        return Ok(Config::default());
    };

    info!("Resolved configuration from {}", config_path.display());

    let raw = RawConfig::builder()
        .add_source(File::from(config_path))
        .build()
        .map_err(|e| {
            error!("Failed to build config: {}", e);
            e
        })?;
    let config = raw.try_deserialize().map_err(|e| {
        error!("Failed to deserialize config: {}", e);
        e
    })?;

    info!("Config loaded successfully");

    Ok(config)
}
