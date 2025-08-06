use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_value};
use std::{collections::HashMap, path::PathBuf};
use toml::{self, Value as TomlValue};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseMetadata {
    pub r#type: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    #[serde(default)]
    pub attachments: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: Option<HashMap<String, TomlValue>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    #[serde(flatten)]
    pub path: PathBuf,
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
            r#type: Some("general".to_string()),
            category: Some("uncategorized".to_string()),
            tags: Some(Vec::new()),
            description: Some(String::new()),
            attachments: Some(Vec::new()),
            extra: None,
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        let now = Utc::now();

        Self {
            path: Some(PathBuf::new()),
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

    pub fn get_field(
        &self,
        field_path: &str,
    ) -> serde_json::Result<Option<Value>> {
        let json_value = to_value(self)?;

        Ok(get_nested_field(&json_value, field_path))
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

fn get_nested_field(value: &Value, path: &str) -> Option<Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            _ => return None,
        }
    }

    Some(current.clone())
}
