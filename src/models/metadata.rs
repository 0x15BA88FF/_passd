use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseMetadata {
    pub r#type: String,
    pub category: String,
    pub tags: Vec<String>,
    pub description: String,

    #[serde(default)]
    pub attachments: Vec<String>,

    #[serde(flatten)]
    pub extra: Option<HashMap<String, toml::Value>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    #[serde(flatten)]
    pub template: BaseMetadata,

    pub modifications: u32,
    pub fingerprint: String,

    pub created_at: String,
    pub updated_at: String,

    pub checksum_main: String,
    pub checksum_meta: String,
}
