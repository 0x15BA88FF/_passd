use crate::{
    models::{
        config::Config,
        metadata::{BaseMetadata, Metadata},
    },
    utils::checksum::compute_checksum,
    utils::fs::{secure_create_dir_all, secure_write},
};
use anyhow::{Context, Result};
use chrono::Utc;
use log::{error, info};
use sequoia_openpgp::{
    Cert, KeyHandle, Result as SequoiaResult,
    crypto::{KeyPair, Password, SessionKey, SymmetricAlgorithm},
    packet::{PKESK, SKESK},
    parse::{
        Parse,
        stream::{
            DecryptionHelper, DecryptorBuilder, MessageStructure,
            VerificationHelper,
        },
    },
    policy::StandardPolicy,
    serialize::stream::{Encryptor, Message, Recipient},
};
use std::{
    cmp::Ordering,
    fs::{copy, read, read_to_string, remove_file, rename},
    io::Write,
    path::PathBuf,
};
use toml;
use walkdir::WalkDir;

struct DecryptHelper {
    keypair: KeyPair,
}

impl DecryptHelper {
    fn new(keypair: KeyPair) -> Self {
        Self { keypair }
    }
}

impl VerificationHelper for DecryptHelper {
    fn get_certs(&mut self, _ids: &[KeyHandle]) -> SequoiaResult<Vec<Cert>> {
        Ok(Vec::new())
    }

    fn check(&mut self, _structure: MessageStructure) -> SequoiaResult<()> {
        Ok(())
    }
}

impl DecryptionHelper for DecryptHelper {
    fn decrypt(
        &mut self,
        pkesks: &[PKESK],
        _skesks: &[SKESK],
        _sym_algo: Option<SymmetricAlgorithm>,
        decrypt: &mut dyn FnMut(
            Option<SymmetricAlgorithm>,
            &SessionKey,
        ) -> bool,
    ) -> SequoiaResult<Option<Cert>> {
        for pkesk in pkesks.iter() {
            if let Some((sym_algo, session_key)) =
                pkesk.decrypt(&mut self.keypair, None)
            {
                if decrypt(sym_algo, &session_key) {
                    return Ok(None);
                }
            }
        }

        Ok(None)
    }
}

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

    pub fn metadata_path(&self) -> Result<PathBuf> {
        let config = Config::load_config()
            .context("Failed to load config for metadata path")?;

        Ok(config
            .metadata_dir
            .join(&self.relative_path)
            .with_extension("meta.toml"))
    }

    pub fn secret_path(&self) -> Result<PathBuf> {
        Ok(self.metadata()?.template.path)
    }

    pub fn plaintext_content(&self) -> Result<String> {
        let secret_path = self.secret_path()?;

        read_to_string(&secret_path).with_context(|| {
            format!("Failed to read plaintext from {}", secret_path.display())
        })
    }

    pub fn metadata(&self) -> Result<Metadata> {
        let metadata_path = self.metadata_path()?;
        let text = read_to_string(&metadata_path).with_context(|| {
            format!("Failed to read metadata from {}", metadata_path.display())
        })?;

        let metadata: Metadata =
            toml::from_str(&text).context("Failed to parse metadata TOML")?;

        Ok(metadata)
    }

    pub fn content(
        &self,
        private_key: Option<&str>,
        password: &str,
    ) -> Result<String> {
        let config = Config::load_config()
            .context("Failed to load config for decryption")?;
        let secret_path = self.secret_path()?;
        let ciphertext = read(&secret_path).with_context(|| {
            format!("Failed to read encrypted file {}", secret_path.display())
        })?;

        let policy = &StandardPolicy::new();
        let cert = Cert::from_bytes(
            match private_key {
                Some(key) => key.to_string(),
                None => std::fs::read_to_string(&config.private_key_path)
                    .context("Failed to read private key file")?,
            }
            .as_bytes(),
        )
        .context("Failed to parse certificate")?;

        let keypair = cert
            .keys()
            .secret()
            .with_policy(policy, None)
            .alive()
            .revoked(false)
            .for_storage_encryption()
            .nth(0)
            .context("No suitable decryption key found")?
            .key()
            .clone()
            .parts_into_secret()
            .context("Failed to get secret key parts")?
            .decrypt_secret(&Password::from(password.to_string()))
            .context("Failed to decrypt secret key with password")?
            .into_keypair()
            .context("Failed to create keypair")?;

        let helper = DecryptHelper::new(keypair);
        let mut decryptor = DecryptorBuilder::from_bytes(&ciphertext)
            .context("Failed to create decryptor")?
            .with_policy(policy, None, helper)
            .context("Failed to configure decryptor policy")?;

        let mut plaintext = Vec::new();

        std::io::copy(&mut decryptor, &mut plaintext)
            .context("Failed to decrypt content")?;

        let result = String::from_utf8(plaintext)
            .context("Decrypted content is not valid UTF-8")?;

        info!(
            "Successfully decrypted secret: {}",
            self.relative_path.display()
        );

        Ok(result)
    }

    pub fn create(
        &self,
        content: &str,
        metadata: &BaseMetadata,
        public_key: Option<&str>,
    ) -> Result<&Self> {
        let secret_path = self.secret_path()?;
        let metadata_path = self.metadata_path()?;

        if secret_path.exists() || metadata_path.exists() {
            return Err(anyhow::anyhow!(
                "Secret or metadata file already exists"
            ));
        }

        if let Some(parent) = secret_path.parent() {
            secure_create_dir_all(parent)
                .context("Failed to create secret directory")?;
        }

        if let Some(parent) = metadata_path.parent() {
            secure_create_dir_all(parent)
                .context("Failed to create metadata directory")?;
        }

        let config = Config::load_config()
            .context("Failed to load config for encryption")?;
        let policy = &StandardPolicy::new();
        let cert = Cert::from_bytes(
            match public_key {
                Some(key) => key.to_string(),
                None => std::fs::read_to_string(&config.public_key_path)
                    .context("Failed to read public key file")?,
            }
            .as_bytes(),
        )
        .context("Failed to parse public key certificate")?;

        let recipients: Vec<Recipient> = cert
            .keys()
            .with_policy(policy, None)
            .alive()
            .revoked(false)
            .for_storage_encryption()
            .map(|ka| ka.into())
            .collect();

        if recipients.is_empty() {
            return Err(anyhow::anyhow!(
                "No suitable encryption key found in public key"
            ));
        }

        let mut encrypted = Vec::new();
        let message = Message::new(&mut encrypted);
        let mut encryptor = Encryptor::for_recipients(message, recipients)
            .build()
            .context("Failed to create encryptor")?;

        encryptor
            .write(content.as_bytes())
            .context("Failed to write content to encryptor")?;
        encryptor
            .finalize()
            .context("Failed to finalize encryption")?;

        secure_write(&secret_path, encrypted)
            .context("Failed to write encrypted secret file")?;

        let checksum_main = compute_checksum(&secret_path)
            .context("Failed to compute secret file checksum")?;
        let mut meta = Metadata {
            fingerprint: cert.fingerprint().to_hex().to_uppercase(),
            checksum_main,
            checksum_meta: "".to_string(),
            ..metadata.clone().into()
        };

        secure_write(
            &metadata_path,
            toml::to_string_pretty(&meta)
                .context("Failed to serialize metadata")?,
        )
        .context("Failed to write metadata file")?;

        meta.template.path = secret_path;
        meta.checksum_meta = compute_checksum(&metadata_path)
            .context("Failed to compute metadata file checksum")?;

        secure_write(
            &metadata_path,
            toml::to_string_pretty(&meta)
                .context("Failed to serialize updated metadata")?,
        )
        .context("Failed to write updated metadata file")?;

        info!("Created secret: {}", self.relative_path.display());

        Ok(self)
    }

    pub fn update(
        &self,
        content: Option<&str>,
        metadata: Option<&BaseMetadata>,
        public_key: Option<&str>,
    ) -> Result<&Self> {
        let secret_path = self.secret_path()?;
        let metadata_path = self.metadata_path()?;

        if !secret_path.exists() || !metadata_path.exists() {
            return Err(anyhow::anyhow!(
                "Secret or metadata file does not exist"
            ));
        }

        let config = Config::load_config()
            .context("Failed to load config for update")?;
        let mut updated_metadata: Metadata = toml::from_str(
            &read_to_string(&metadata_path)
                .context("Failed to read existing metadata")?,
        )
        .context("Failed to parse existing metadata")?;

        if let Some(base) = metadata {
            let base_as_metadata: Metadata = base.clone().into();

            updated_metadata = updated_metadata
                .merge(&base_as_metadata)
                .context("Failed to merge metadata")?;
        }

        if let Some(content) = content {
            let policy = &StandardPolicy::new();
            let cert = Cert::from_bytes(
                match public_key {
                    Some(key) => key.to_string(),
                    None => std::fs::read_to_string(&config.public_key_path)
                        .context("Failed to read public key file")?,
                }
                .as_bytes(),
            )
            .context("Failed to parse public key certificate")?;

            let recipients: Vec<Recipient> = cert
                .keys()
                .with_policy(policy, None)
                .alive()
                .revoked(false)
                .for_storage_encryption()
                .map(|ka| ka.into())
                .collect();

            if recipients.is_empty() {
                return Err(anyhow::anyhow!(
                    "No suitable encryption key found in public key"
                ));
            }

            let mut encrypted = Vec::new();
            let message = Message::new(&mut encrypted);
            let mut encryptor = Encryptor::for_recipients(message, recipients)
                .build()
                .context("Failed to create encryptor for update")?;

            encryptor
                .write(content.as_bytes())
                .context("Failed to write updated content to encryptor")?;
            encryptor
                .finalize()
                .context("Failed to finalize encryption for update")?;

            secure_write(&secret_path, encrypted)
                .context("Failed to write updated encrypted secret file")?;

            updated_metadata.fingerprint =
                cert.fingerprint().to_hex().to_uppercase();
            updated_metadata.checksum_main = compute_checksum(&secret_path)
                .context("Failed to compute updated secret file checksum")?;
        }

        updated_metadata.modifications =
            updated_metadata.modifications.saturating_add(1);
        updated_metadata.updated_at = Utc::now();
        updated_metadata.checksum_meta = "".to_string();

        secure_write(
            &metadata_path,
            toml::to_string_pretty(&updated_metadata)
                .context("Failed to serialize updated metadata")?,
        )
        .context("Failed to write updated metadata file")?;

        updated_metadata.checksum_meta = compute_checksum(&metadata_path)
            .context("Failed to compute updated metadata file checksum")?;

        secure_write(
            &metadata_path,
            toml::to_string_pretty(&updated_metadata)
                .context("Failed to serialize final metadata")?,
        )
        .context("Failed to write final metadata file")?;

        info!(
            "Updated secret: {} (modifications: {})",
            self.relative_path.display(),
            updated_metadata.modifications
        );

        Ok(self)
    }

    pub fn remove(&self) -> Result<()> {
        let paths = [self.secret_path()?, self.metadata_path()?];
        let mut errors = Vec::new();

        for path in &paths {
            match remove_file(path) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => errors.push(format!(
                    "Failed to remove {}: {}",
                    path.display(),
                    e
                )),
            }
        }

        if !errors.is_empty() {
            error!("Removal errors: {}", errors.join("; "));
            return Err(anyhow::anyhow!(
                "Removal errors: {}",
                errors.join("; ")
            ));
        }

        info!("Removed secret: {}", self.relative_path.display());

        Ok(())
    }

    pub fn move_to(&self, destination: PathBuf) -> Result<Secret> {
        let destination_secret = Secret {
            relative_path: destination,
        };
        let current_secret_path = self.secret_path()?;
        let current_metadata_path = self.metadata_path()?;
        let dest_secret_path = destination_secret.secret_path()?;
        let dest_metadata_path = destination_secret.metadata_path()?;

        if let Some(parent) = dest_secret_path.parent() {
            secure_create_dir_all(parent)
                .context("Failed to create destination secret directory")?;
        }
        if let Some(parent) = dest_metadata_path.parent() {
            secure_create_dir_all(parent)
                .context("Failed to create destination metadata directory")?;
        }

        rename(&current_secret_path, &dest_secret_path)
            .context("Failed to move secret file")?;
        rename(&current_metadata_path, &dest_metadata_path)
            .context("Failed to move metadata file")?;

        Ok(destination_secret)
    }

    pub fn copy_to(&self, destination: PathBuf) -> Result<Secret> {
        let destination_secret = Secret {
            relative_path: destination,
        };
        let current_secret_path = self.secret_path()?;
        let current_metadata_path = self.metadata_path()?;
        let dest_secret_path = destination_secret.secret_path()?;
        let dest_metadata_path = destination_secret.metadata_path()?;

        if let Some(parent) = dest_secret_path.parent() {
            secure_create_dir_all(parent)
                .context("Failed to create destination secret directory")?;
        }
        if let Some(parent) = dest_metadata_path.parent() {
            secure_create_dir_all(parent)
                .context("Failed to create destination metadata directory")?;
        }

        copy(&current_secret_path, &dest_secret_path)
            .context("Failed to copy secret file")?;
        copy(&current_metadata_path, &dest_metadata_path)
            .context("Failed to copy metadata file")?;

        Ok(destination_secret)
    }

    pub fn clone_to(
        &self,
        destination: PathBuf,
        public_key: &str,
        private_key: Option<&str>,
        password: &str,
    ) -> Result<Secret> {
        let destination_secret = Secret {
            relative_path: destination,
        };
        let decrypted_content = self
            .content(private_key, password)
            .context("Failed to decrypt content for cloning")?;
        let metadata = self
            .metadata()
            .context("Failed to read metadata for cloning")?;

        destination_secret
            .create(&decrypted_content, &metadata.to_base(), Some(public_key))
            .context("Failed to create cloned secret")?;

        Ok(destination_secret)
    }

    pub fn find<F, C>(
        &self,
        filter: Option<F>,
        mut sort: Option<C>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<Secret>>
    where
        F: Fn(&PathBuf, &Metadata) -> bool,
        C: FnMut(&PathBuf, &Metadata) -> Ordering,
    {
        let mut results = Vec::new();
        let config = Config::load_config()
            .context("Failed to load config for search")?;

        for entry in WalkDir::new(&config.metadata_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|name| name.ends_with(".meta.toml"))
                        .unwrap_or(false)
            })
        {
            let full_path = entry.path();
            let relative_path =
                match full_path.strip_prefix(&config.metadata_dir) {
                    Ok(r) => r,
                    Err(_) => continue,
                };

            let file_stem = match relative_path
                .file_name()
                .and_then(|f| f.to_str())
                .and_then(|f| f.strip_suffix(".meta.toml"))
            {
                Some(stem) => stem,
                None => continue,
            };

            let mut base = relative_path.to_path_buf();

            base.set_file_name(file_stem);

            let secret = Secret {
                relative_path: base.clone(),
            };

            let metadata = match secret.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if let Some(ref filter_fn) = filter {
                if !filter_fn(&secret.relative_path, &metadata) {
                    continue;
                }
            }

            results.push((secret, metadata));
        }

        if let Some(ref mut cmp_fn) = sort {
            results.sort_by(|(a, a_meta), (b, b_meta)| {
                cmp_fn(&a.relative_path, a_meta)
                    .then_with(|| cmp_fn(&b.relative_path, b_meta))
            });
        }

        let total = results.len();
        let start = offset.unwrap_or(0).min(total);
        let end = limit.map_or(total, |l| (start + l).min(total));

        let response = results[start..end]
            .iter()
            .map(|(secret, _)| secret.clone())
            .collect();

        Ok(response)
    }
}
