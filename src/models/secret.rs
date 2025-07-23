use crate::{
    configs::load_config,
    models::metadata::{BaseMetadata, Metadata},
    utils::checksum::compute_checksum,
    utils::fs::{secure_create_dir_all, secure_write},
};
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
use serde::Serialize;
use std::{
    cmp::Ordering,
    error::Error,
    fs::{copy, read, read_to_string, remove_file, rename},
    io::Write,
    path::PathBuf,
};
use toml;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct VaultSecret {
    pub path: String,
    pub metadata: Metadata,
}

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

    pub fn secret_path(&self) -> PathBuf {
        let config = load_config().expect("Failed to load config");

        config
            .vault_dir
            .join(&self.relative_path)
            .with_extension("pgp")
    }

    pub fn metadata_path(&self) -> PathBuf {
        let config = load_config().expect("Failed to load config");

        config
            .metadata_dir
            .join(&self.relative_path)
            .with_extension("meta.toml")
    }

    pub fn plaintext_content(&self) -> Result<String, Box<dyn Error>> {
        let secret_path = self.secret_path();

        Ok(read_to_string(&secret_path)?)
    }

    pub fn metadata(&self) -> Result<Metadata, Box<dyn Error>> {
        let metadata_path = self.metadata_path();
        let text = read_to_string(&metadata_path)?;
        let metadata: Metadata = toml::from_str(&text)?;

        Ok(metadata)
    }

    pub fn content(
        &self,
        private_key: Option<&str>,
        password: &str,
    ) -> Result<String, Box<dyn Error>> {
        let config = load_config()?;
        let secret_path = self.secret_path();
        let ciphertext = read(&secret_path)?;
        let policy = &StandardPolicy::new();
        let cert = Cert::from_bytes(
            match private_key {
                Some(key) => key.to_string(),
                None => std::fs::read_to_string(&config.private_key_path)?,
            }
            .as_bytes(),
        )?;
        let keypair = cert
            .keys()
            .secret()
            .with_policy(policy, None)
            .alive()
            .revoked(false)
            .for_storage_encryption()
            .nth(0)
            .ok_or("No suitable decryption key found")?
            .key()
            .clone()
            .parts_into_secret()?
            .decrypt_secret(&Password::from(password.to_string()))?
            .into_keypair()?;
        let helper = DecryptHelper::new(keypair);
        let mut decryptor = DecryptorBuilder::from_bytes(&ciphertext)?
            .with_policy(policy, None, helper)?;
        let mut plaintext = Vec::new();

        std::io::copy(&mut decryptor, &mut plaintext)?;

        let result = String::from_utf8(plaintext)?;

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
    ) -> Result<&Self, Box<dyn Error>> {
        let secret_path = self.secret_path();
        let metadata_path = self.metadata_path();

        if secret_path.exists() || metadata_path.exists() {
            return Err("Secret or metadata file already exists".into());
        }

        if let Some(parent) = secret_path.parent() {
            secure_create_dir_all(parent)?;
        }

        if let Some(parent) = metadata_path.parent() {
            secure_create_dir_all(parent)?;
        }

        let config = load_config()?;
        let policy = &StandardPolicy::new();
        let cert = Cert::from_bytes(
            match public_key {
                Some(key) => key.to_string(),
                None => std::fs::read_to_string(&config.public_key_path)?,
            }
            .as_bytes(),
        )?;
        let recipients: Vec<Recipient> = cert
            .keys()
            .with_policy(policy, None)
            .alive()
            .revoked(false)
            .for_storage_encryption()
            .map(|ka| ka.into())
            .collect();

        if recipients.is_empty() {
            return Err("No suitable encryption key found in public key".into());
        }

        let mut encrypted = Vec::new();
        let message = Message::new(&mut encrypted);
        let mut encryptor =
            Encryptor::for_recipients(message, recipients).build()?;

        encryptor.write(content.as_bytes())?;
        encryptor.finalize()?;

        secure_write(&secret_path, encrypted)?;

        let checksum_main = compute_checksum(&secret_path)?;
        let temp_meta = Metadata {
            fingerprint: cert.fingerprint().to_hex().to_uppercase(),
            checksum_main,
            checksum_meta: "".to_string(),
            ..metadata.clone().into()
        };

        secure_write(&metadata_path, toml::to_string_pretty(&temp_meta)?)?;

        let final_meta = Metadata {
            checksum_meta: compute_checksum(&metadata_path)?,
            ..temp_meta
        };

        secure_write(&metadata_path, toml::to_string_pretty(&final_meta)?)?;

        info!("Created secret: {}", self.relative_path.display());

        Ok(self)
    }

    pub fn update(
        &self,
        content: Option<&str>,
        metadata: Option<&BaseMetadata>,
        public_key: Option<&str>,
    ) -> Result<&Self, Box<dyn Error>> {
        let secret_path = self.secret_path();
        let metadata_path = self.metadata_path();

        if !secret_path.exists() || !metadata_path.exists() {
            return Err("Secret or metadata file does not exist".into());
        }

        let config = load_config()?;
        let mut updated_metadata: Metadata =
            toml::from_str(&read_to_string(&metadata_path)?)?;

        if let Some(base) = metadata {
            let base_as_metadata: Metadata = base.clone().into();

            updated_metadata = updated_metadata.merge(&base_as_metadata)?;
        }

        if let Some(content) = content {
            let policy = &StandardPolicy::new();
            let cert = Cert::from_bytes(
                match public_key {
                    Some(key) => key.to_string(),
                    None => std::fs::read_to_string(&config.public_key_path)?,
                }
                .as_bytes(),
            )?;
            let recipients: Vec<Recipient> = cert
                .keys()
                .with_policy(policy, None)
                .alive()
                .revoked(false)
                .for_storage_encryption()
                .map(|ka| ka.into())
                .collect();

            if recipients.is_empty() {
                return Err(
                    "No suitable encryption key found in public key".into()
                );
            }

            let mut encrypted = Vec::new();
            let message = Message::new(&mut encrypted);
            let mut encryptor =
                Encryptor::for_recipients(message, recipients).build()?;

            encryptor.write(content.as_bytes())?;
            encryptor.finalize()?;

            secure_write(&secret_path, encrypted)?;

            updated_metadata.fingerprint =
                cert.fingerprint().to_hex().to_uppercase();
            updated_metadata.checksum_main = compute_checksum(&secret_path)?;
        }

        updated_metadata.modifications =
            updated_metadata.modifications.saturating_add(1);
        updated_metadata.updated_at = Utc::now().to_string();
        updated_metadata.checksum_meta = "".to_string();

        secure_write(
            &metadata_path,
            toml::to_string_pretty(&updated_metadata)?,
        )?;

        updated_metadata.checksum_meta = compute_checksum(&metadata_path)?;

        secure_write(
            &metadata_path,
            toml::to_string_pretty(&updated_metadata)?,
        )?;

        info!(
            "Updated secret: {} (modifications: {})",
            self.relative_path.display(),
            updated_metadata.modifications
        );

        Ok(self)
    }

    pub fn remove(&self) -> Result<(), Box<dyn Error>> {
        let paths = [self.secret_path(), self.metadata_path()];
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
            return Err(errors.join("; ").into());
        }

        info!("Removed secret: {}", self.relative_path.display());

        Ok(())
    }

    pub fn move_to(
        &self,
        destination: PathBuf,
    ) -> Result<Secret, Box<dyn Error>> {
        let destination_secret = Secret {
            relative_path: destination,
        };
        let current_secret_path = self.secret_path();
        let current_metadata_path = self.metadata_path();
        let dest_secret_path = destination_secret.secret_path();
        let dest_metadata_path = destination_secret.metadata_path();

        if let Some(parent) = dest_secret_path.parent() {
            secure_create_dir_all(parent)?;
        }
        if let Some(parent) = dest_metadata_path.parent() {
            secure_create_dir_all(parent)?;
        }

        rename(&current_secret_path, &dest_secret_path)?;
        rename(&current_metadata_path, &dest_metadata_path)?;

        Ok(destination_secret)
    }

    pub fn copy_to(
        &self,
        destination: PathBuf,
    ) -> Result<Secret, Box<dyn Error>> {
        let destination_secret = Secret {
            relative_path: destination,
        };
        let current_secret_path = self.secret_path();
        let current_metadata_path = self.metadata_path();
        let dest_secret_path = destination_secret.secret_path();
        let dest_metadata_path = destination_secret.metadata_path();

        if let Some(parent) = dest_secret_path.parent() {
            secure_create_dir_all(parent)?;
        }
        if let Some(parent) = dest_metadata_path.parent() {
            secure_create_dir_all(parent)?;
        }

        copy(&current_secret_path, &dest_secret_path)?;
        copy(&current_metadata_path, &dest_metadata_path)?;

        Ok(destination_secret)
    }

    pub fn clone_to(
        &self,
        destination: PathBuf,
        public_key: &str,
        private_key: Option<&str>,
        password: &str,
    ) -> Result<Secret, Box<dyn Error>> {
        let destination_secret = Secret {
            relative_path: destination,
        };
        let decrypted_content = self.content(private_key, password)?;
        let metadata = self.metadata()?;

        destination_secret.create(
            &decrypted_content,
            &metadata.to_base(),
            Some(public_key),
        )?;

        Ok(destination_secret)
    }

    pub fn find<F, C>(
        &self,
        filter: Option<F>,
        mut sort: Option<C>,
        skip: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<VaultSecret>, Box<dyn Error>>
    where
        F: Fn(&Metadata) -> bool,
        C: FnMut(&Metadata, &Metadata) -> Ordering,
    {
        let mut results = Vec::new();
        let config = load_config().expect("Failed to load config");

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

            let content = match read_to_string(full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let metadata: Metadata = match toml::from_str(&content) {
                Ok(m) => m,
                Err(_) => continue,
            };

            if let Some(ref filter_fn) = filter {
                if !filter_fn(&metadata) {
                    continue;
                }
            }

            results.push((base, metadata));
        }

        if let Some(ref mut cmp_fn) = sort {
            results.sort_by(|a, b| cmp_fn(&a.1, &b.1));
        }

        let total = results.len();
        let start = skip.unwrap_or(0).min(total);
        let end = limit.map_or(total, |l| (start + l).min(total));
        let response: Vec<VaultSecret> = results[start..end]
            .iter()
            .map(|(path, metadata)| VaultSecret {
                path: path.to_string_lossy().to_string(),
                metadata: metadata.clone(),
            })
            .collect();

        Ok(response)
    }
}
