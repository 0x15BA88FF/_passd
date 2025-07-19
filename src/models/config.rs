use crate::models::metadata::BaseMetadata;
use dirs;
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

    pub port: u16,
    pub enable_tls: bool,

    #[serde(default)]
    pub metadata_template: Option<BaseMetadata>,
}

impl Default for Config {
    fn default() -> Self {
        let base = dirs::home_dir()
            .map(|home| home.join(".passd"))
            .unwrap_or_else(|| PathBuf::from(".passd"));

        let vault_dir = base.join("vault");
        let keys_dir = base.join(".keys");
        let log_file = base.join("logs/passd.log");
        let metadata_dir = vault_dir.join(".metadata");

        Self {
            vault_dir: vault_dir,
            metadata_dir: metadata_dir,

            log_file: logs_file,
            log_level: "info".to_string(),

            public_key_path: keys_dir.join("public.pem"),
            private_key_path: keys_dir.join("private.pem"),

            port: 7117,
            enable_tls: true,

            metadata_template: Some(BaseMetadata::default()),
        }
    }
}
