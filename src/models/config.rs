use std::{env, net::IpAddr, path::PathBuf};

use anyhow::{Context, Result};
use config::{Config as RawConfig, File};
use directories::BaseDirs;
use log::LevelFilter;
use serde::Deserialize;

use crate::models::metadata::BaseMetadata;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Error,
    Warn,
    Trace,
    Info,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Info => LevelFilter::Info,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub base_dir: PathBuf,
    pub secrets_dir: PathBuf,
    pub metadata_dir: PathBuf,
    pub keys_dir: PathBuf,
    pub log_file: PathBuf,
    pub log_level: LogLevel,
    pub address: IpAddr,
    pub port: u16,
    pub metadata_template: Option<BaseMetadata>,
}

impl Default for Config {
    fn default() -> Self {
        let base_dir = match dirs::home_dir() {
            Some(home) => home.join(".passd"),
            None => PathBuf::from(".passd"),
        };

        Self {
            base_dir: base_dir.clone(),
            secrets_dir: base_dir.join("secrets"),
            metadata_dir: base_dir.join(".metadata"),
            keys_dir: base_dir.join(".keys"),
            log_file: base_dir.join(".passd.log"),
            log_level: LogLevel::Info,
            address: "127.0.0.1".parse().unwrap(),
            port: 7117,
            metadata_template: Some(BaseMetadata::default()),
        }
    }
}

impl Config {
    pub fn load_config() -> Result<Self> {
        let default_config = Self::default();
        let config_path = [
            env::var("PASSD_CONFIG_DIR")
                .ok()
                .map(|dir| PathBuf::from(dir).join("config.toml")),
            BaseDirs::new()
                .map(|base| base.config_dir().join("passd/config.toml")),
            dirs::home_dir().map(|home| home.join(".passd/config.toml")),
        ]
        .into_iter()
        .flatten()
        .find(|path| path.exists());

        if config_path.is_none() {
            log::warn!(
                "Failed to resolve configuration file, using default configuration"
            );
            return Ok(default_config);
        }

        log::info!(
            "Successfully resolved configuration {}",
            config_path.clone().unwrap().display()
        );

        let raw = RawConfig::builder()
            .add_source(File::from(config_path.unwrap()))
            .build()
            .context("Failed to build configuration")?;
        let mut config: Self = raw
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        match (
            config.metadata_dir == config.secrets_dir,
            config.metadata_dir == config.keys_dir,
            config.secrets_dir == config.keys_dir,
        ) {
            (false, false, false) => {}
            _ => {
                log::warn!(
                    "Configuration directories paths conflict, using to default paths"
                );
                config.metadata_dir = default_config.metadata_dir;
                config.secrets_dir = default_config.secrets_dir;
                config.keys_dir = default_config.keys_dir;
            }
        }

        log::info!("Configuration loaded successfully");
        Ok(config)
    }
}
