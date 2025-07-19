use crate::configs::load_config;
use std::path::{Path, PathBuf};

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
}
