use serde::Deserialize;
use crate::model::metadata::BaseMetadata;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub dir: String,
    pub logs_dir: String,
    pub metadata_dir: String,

    pub public_key_path: String,
    pub private_key_path: String,

    pub port: u16,
    pub enable_tls: bool,

    #[serde(default)]
    pub metadata_template: Option<BaseMetadata>,
}
