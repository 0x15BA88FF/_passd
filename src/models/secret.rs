use crate::{
    configs::load_config,
    models::metadata::{BaseMetadata, Metadata},
};
use openpgp::{
    Cert, Result as PgpResult,
    crypto::{Password, SessionKey},
    parse::{
        Parse,
        stream::{DecryptorBuilder, MessageStructure},
    },
    policy::StandardPolicy,
    serialize::stream::{Encryptor, Message},
};
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};
use toml;

#[derive(Debug, Clone)]
pub struct Secret {
    pub relative_path: PathBuf,
}

impl Secret {
    pub fn new(relative_path: impl Into<PathBuf>) -> Self {
        Self {
            relative_path: relative_path.into(),
        }
    }

    pub fn secret_path(&self) -> PathBuf {
        let config = load_config()?;

        config
            .vault_dir
            .join(&self.relative_path)
            .with_extension("pgp")
    }

    pub fn metadata_path(&self) -> PathBuf {
        let config = load_config()?;

        config
            .metadata_path
            .join(&self.relative_path)
            .with_extension("meta.toml")
    }

    pub fn content(
        &self,
        private_key: &str,
        password: &str,
    ) -> Result<String, Box<dyn Error>> {
        let ciphertext = fs::read(&self.secret_path())?;
        let policy = &StandardPolicy::new();
        let (cert, _) = Cert::from_bytes(private_key.as_bytes())?;
        let keypair = cert
            .keys()
            .secret()
            .with_policy(policy, None)
            .alive()
            .revoked(false)
            .for_transport_decryption()
            .nth(0)
            .ok_or("No suitable decryption key found")?
            .key()
            .clone()
            .unlock(|| Password::from(password.to_string()))?
            .into_keypair()?;
        let mut decryptor =
            DecryptorBuilder::from_bytes(&ciphertext)?.build(|| Ok(keypair))?;

        let mut plaintext = Vec::new();

        std::io::copy(&mut decryptor, &mut plaintext)?;

        Ok(String::from_utf8(plaintext)?)
    }

    pub fn metadata(&self) -> Result<Metadata, Box<dyn Error>> {
        let text = fs::read_to_string(&self.metadata_path())?;
        let metadata: Metadata = toml::from_str(&text)?;

        Ok(metadata)
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

        let policy = &StandardPolicy::new();
        let cert = Cert::from_bytes(public_key.as_bytes())?;
        let fingerprint = cert.fingerprint().to_hex().to_uppercase();
        let recipients: Vec<_> = cert
            .keys()
            .with_policy(policy, None)
            .alive()
            .revoked(false)
            .for_transport_encryption()
            .collect();

        if recipients.is_empty() {
            return Err("No suitable encryption key found in public key".into());
        }

        let mut encrypted = Vec::new();
        let message = Message::new(&mut encrypted);
        let mut encryptor = Encryptor::for_recipients(
            message,
            recipients.iter().map(|r| r.key()),
        )?
        .build()?;

        encryptor.write_all(content.as_bytes())?;
        encryptor.finalize()?;

        fs::write(&self.secret_path(), encrypted)?;

        fs::write(
            &self.metadata_path(),
            toml::to_string_pretty(Metadata {
                fingerprint: fingerprint,
                checksum_main: "".to_string(),
                checksum_meta: "".to_string(),
                ..metadata.clone()
            })?,
        )?;

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
            fs::write(
                &self.metadata_path(),
                toml::to_string_pretty(Metadata {
                    modifications: metadata.modifications.saturating_add(1),
                    fingerprint: "".to_string(),
                    updated_at: Some(Utc::now()),
                    checksum_main: "".to_string(),
                    checksum_meta: "".to_string(),
                    ..metadata.clone()
                })?,
            )?;
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
}
