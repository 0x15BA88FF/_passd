use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use toml::{self, Value as TomlValue};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseMetadata {
    pub path: PathBuf,
    pub r#type: String,
    pub category: String,
    pub tags: Vec<String>,
    pub description: String,
    #[serde(default)]
    pub attachments: Vec<String>,
    #[serde(flatten)]
    pub extra: Option<HashMap<String, TomlValue>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    #[serde(flatten)]
    pub template: BaseMetadata,
    pub modifications: u32,
    pub fingerprint: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub checksum_main: String,
    pub checksum_meta: String,
}

impl Default for BaseMetadata {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
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
        let now = Utc::now();

        Self {
            template: BaseMetadata::default(),
            modifications: 0,
            fingerprint: String::new(),
            created_at: now,
            updated_at: now,
            checksum_main: String::new(),
            checksum_meta: String::new(),
        }
    }
}

impl From<Metadata> for BaseMetadata {
    fn from(metadata: Metadata) -> Self {
        metadata.template
    }
}

impl From<BaseMetadata> for Metadata {
    fn from(base_metadata: BaseMetadata) -> Self {
        let mut default = Self::default();

        default.template = base_metadata;
        default
    }
}

impl Metadata {
    pub fn to_base(&self) -> BaseMetadata {
        self.template.clone()
    }

    pub fn merge(&self, other: &Metadata) -> Result<Self> {
        let mut self_value = toml::Value::try_from(self.clone())
            .context("Failed to convert current metadata to TOML value")?;

        let other_value = toml::Value::try_from(other.clone())
            .context("Failed to convert other metadata to TOML value")?;

        merge_toml(&mut self_value, &other_value);

        let merged: Metadata = toml::from_str(&self_value.to_string())
            .context("Failed to deserialize merged metadata from TOML")?;

        Ok(merged)
    }
}

fn merge_toml(base: &mut TomlValue, other: &TomlValue) {
    match (base, other) {
        (TomlValue::Table(base_table), TomlValue::Table(other_table)) => {
            for (key, value) in other_table {
                match base_table.get_mut(key) {
                    Some(existing_value) => merge_toml(existing_value, value),
                    None => {
                        base_table.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (TomlValue::Array(base_array), TomlValue::Array(other_array)) => {
            base_array.extend(other_array.clone());
        }
        (base_val, other_val) => {
            *base_val = other_val.clone();
        }
    }
}
