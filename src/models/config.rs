use crate::models::metadata::BaseMetadata;
use dirs;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub vault_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub metadata_dir: PathBuf,

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
        let logs_dir = base.join("logs");
        let keys_dir = base.join(".keys");
        let metadata_dir = vault_dir.join(".metadata");

        Self {
            vault_dir: vault_dir,
            logs_dir: logs_dir,
            metadata_dir: metadata_dir,

            public_key_path: keys_dir.join("public.pem"),
            private_key_path: keys_dir.join("private.pem"),

            port: 7117,
            enable_tls: true,

            metadata_template: Some(BaseMetadata::default()),
        }
    }
}
