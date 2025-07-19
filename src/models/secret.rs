use crate::{configs::load_config, models::metadata::{BaseMetadata, Metadata}};
use std::{error::Error, fs, path::{Path, PathBuf}};
use toml;

#[derive(Debug, Clone)]
pub struct Secret {
    pub relative_path: PathBuf,
}

impl Secret {
    pub fn new(
        relative_path: impl Into<PathBuf>
    ) -> Self {
        Self { relative_path: relative_path.into() }
    }

    pub fn secret_path(&self) -> PathBuf {
        let config = load_config()?;

        config.vault_dir.join(&self.relative_path)
            .with_extension("pgp")
    }

    pub fn metadata_path(&self) -> PathBuf {
        let config = load_config()?;

        config.metadata_path.join(&self.relative_path)
            .with_extension("meta.toml")
    }

    pub fn create(
        &self,
        content: &str,
        metadata: &BaseMetadata,
        public_key: &str,
    ) -> Result<&Self, Box<dyn Error>> {
        if self.secret_path().exists() || self.metadata_path().exists() {
            return Err("Secret or metadata file already exists".into());
        }

        if let Some(parent) = self.secret_path().parent() {
            fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.metadata_path().parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.secret_path(), content)?;
        fs::write(&self.metadata_path(), toml::to_string_pretty(Metadata {
            fingerprint: "".to_string(),
            checksum_main: "".to_string(),
            checksum_meta: "".to_string(),
            ..metadata.clone()
        })?)?;

        Ok(self)
    }

    pub fn update(
        &self,
        content: Option<&str>,
        metadata: Option<&BaseMetadata>,
        public_key: Option<&str>,
    ) -> Result<&Self, Box<dyn Error>> {
        if !self.secret_path().exists() || !self.metadata_path().exists() {
            return Err("Secret or metadata file does not exists".into());
        }

        match (content, public_key) {
            (Some(content), Some(public_key)) => {
                fs::write(&self.secret_path(), content)?;
            }
            (Some(_), _) => {
                return Err("Public key required to update secret".into());
            }
            _ => {}
        }

        if let Some(metadata) = metadata {
            fs::write(&self.metadata_path(), toml::to_string_pretty(Metadata {
                modifications: metadata.modifications.saturating_add(1),
                fingerprint: "".to_string(),
                updated_at: Some(Utc::now()),
                checksum_main: "".to_string(),
                checksum_meta: "".to_string(),
                ..metadata.clone()
            })?)?;
        }

        Ok(self)
    }

    pub fn remove(&self) -> Result<(), Box<dyn Error>> {
        for path in [&self.secret_path(), &self.metadata_path()] {
            match fs::remove_file(path) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(Box::new(e)),
            }
        }

        Ok(())
    }

    pub fn content(
        &self,
        private_key: &str
    ) -> Result<String, Box<dyn Error>> {
        let content = fs::read_to_string(&self.secret_path())?;

        Ok(content)
    }

    pub fn metadata(&self) -> Result<Metadata, Box<dyn Error>> {
        let text = fs::read_to_string(&self.metadata_path())?;
        let metadata: Metadata = toml::from_str(&text)?;

        Ok(metadata)
    }
}
