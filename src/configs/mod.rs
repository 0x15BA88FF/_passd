use crate::models::config::Config;
use config::{Config as RawConfig, File};
use directories::BaseDirs;
use dirs;
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
        return Ok(Config::default());
    };
    let raw = RawConfig::builder()
        .add_source(File::from(config_path))
        .build()?;
    let config = raw.try_deserialize()?;

    Ok(config)
}
