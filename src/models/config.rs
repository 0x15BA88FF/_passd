use crate::{
    models::metadata::BaseMetadata, utils::config::resolve_config_paths,
};
use anyhow::{Context, Result};
use config::{Config as RawConfig, File};
use dirs;
use log::{info, warn};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub vault_dir: PathBuf,
    pub metadata_dir: PathBuf,
    pub log_file: PathBuf,
    pub log_level: String,
    pub public_key_path: PathBuf,
    pub private_key_path: PathBuf,
    pub address: String,
    pub port: u16,
    pub enable_tls: bool,
    #[serde(default)]
    pub metadata_template: Option<BaseMetadata>,
}

impl Default for Config {
    fn default() -> Self {
        let base_dir = match dirs::home_dir() {
            Some(home) => home.join(".passd"),
            None => PathBuf::from(".passd"),
        };

        Self {
            vault_dir: base_dir.join("secrets"),
            metadata_dir: base_dir.join(".metadata"),
            log_file: base_dir.join(".passd.log"),
            log_level: "info".to_string(),
            public_key_path: base_dir.join(".keys/public.pem"),
            private_key_path: base_dir.join(".keys/private.pem"),
            address: "127.0.0.1".to_string(),
            port: 7117,
            enable_tls: true,
            metadata_template: Some(BaseMetadata::default()),
        }
    }
}

impl Config {
    pub fn load_config() -> Result<Self> {
        let config_path = match resolve_config_paths() {
            Some(path) => {
                info!("Successfully resolved configuration {}", path.display());
                path
            }
            None => {
                warn!("Failed to resolve configuration file, using defaults");
                return Ok(Self::default());
            }
        };

        let raw = RawConfig::builder()
            .add_source(File::from(config_path))
            .build()
            .context("Failed to build configuration")?;

        let config: Self = raw
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        info!("Configuration loaded successfully");
        Ok(config)
    }
}
