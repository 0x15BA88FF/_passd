use serde::Deserialize;
use crate::model::metadata::BaseMetadata;

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
