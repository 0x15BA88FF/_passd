use chrono::Utc;
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

impl Default for BaseMetadata {
    fn default() -> Self {
        Self {
            r#type: "general".to_string(),
            category: "uncategorized".to_string(),
            tags: Vec::new(),
            description: String::new(),

            attachments: Vec::new(),

            extra: None,
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now().to_rfc3339();

        Self {
            template: BaseMetadata::default(),

            modifications: 0,
            fingerprint: String::new(),

            created_at: now.clone(),
            updated_at: now,

            checksum_main: String::new(),
            checksum_meta: String::new(),
        }
    }
}
