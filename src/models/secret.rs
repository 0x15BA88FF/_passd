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
    serialize::stream::{
        Armorer, Encryptor, LiteralWriter, Message, Recipient,
    },
};
use std::{
    fs::{copy, read, read_to_string, remove_file, rename},
    io::Write,
    path::PathBuf,
    sync::Arc,
};
use toml;

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

#[derive(Debug)]
pub struct Secret {
    pub relative_path: PathBuf,
    pub config: Arc<Config>,
}

impl Secret {
    pub fn new(relative_path: PathBuf, config: Arc<Config>) -> Self {
        Self {
            relative_path,
            config,
        }
    }

    pub fn metadata_path(&self) -> Result<PathBuf> {
        Ok(self
            .config
            .metadata_dir
            .join(&self.relative_path)
            .with_extension("meta.toml"))
    }

    pub fn secret_path(&self) -> Result<PathBuf> {
        Ok(self
            .config
            .secrets_dir
            .join(&self.relative_path)
            .with_extension("pgp"))
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
        let secret_path = self.secret_path()?;
        let ciphertext = read(&secret_path).with_context(|| {
            format!("Failed to read encrypted file {}", secret_path.display())
        })?;

        let policy = &StandardPolicy::new();
        let cert = Cert::from_bytes(
            match private_key {
                Some(key) => key.to_string(),
                None => read_to_string(&self.config.private_key_path)
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
            secure_create_dir_all(parent, &self.config.secrets_dir).context(
                format!(
                    "Failed to create secret directory {}",
                    parent.display(),
                ),
            )?;
        }
        if let Some(parent) = metadata_path.parent() {
            secure_create_dir_all(parent, &self.config.metadata_dir)
                .context("Failed to create metadata directory")?;
        }

        let policy = &StandardPolicy::new();
        let cert = Cert::from_bytes(
            match public_key {
                Some(key) => key.to_string(),
                None => read_to_string(&self.config.public_key_path)
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

        let message = Armorer::new(message)
            .build()
            .expect("Trying to armor the message");
        let message = Encryptor::for_recipients(message, recipients)
            .build()
            .expect("Trying to build encrypted message");
        let mut message = LiteralWriter::new(message)
            .build()
            .expect("Trying to build armored ascii Writer");

        message
            .write_all(content.as_bytes())
            .expect("Trying to write out data to encrypted stream");
        message.finalize().expect("Trying to finalize encryption");

        secure_write(&secret_path, encrypted)
            .context("Failed to write encrypted secret file")?;

        let checksum_main = compute_checksum(&secret_path)
            .context("Failed to compute secret file checksum")?;
        let mut meta = Metadata {
            path: Some(secret_path),
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
                    None => read_to_string(&self.config.public_key_path)
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
            let message = Armorer::new(message)
                .build()
                .expect("Trying to armor the message");
            let message = Encryptor::for_recipients(message, recipients)
                .build()
                .expect("Trying to build encrypted message");
            let mut message = LiteralWriter::new(message)
                .build()
                .expect("Trying to build armored ascii Writer");

            message
                .write_all(content.as_bytes())
                .expect("Trying to write out data to encrypted stream");
            message.finalize().expect("Trying to finalize encryption");

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
            config: Arc::clone(&self.config),
        };
        let current_secret_path = self.secret_path()?;
        let current_metadata_path = self.metadata_path()?;
        let dest_secret_path = destination_secret.secret_path()?;
        let dest_metadata_path = destination_secret.metadata_path()?;

        if let Some(parent) = dest_secret_path.parent() {
            secure_create_dir_all(parent, &self.config.secrets_dir)
                .context("Failed to create destination secret directory")?;
        }
        if let Some(parent) = dest_metadata_path.parent() {
            secure_create_dir_all(parent, &self.config.metadata_dir)
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
            config: Arc::clone(&self.config),
        };
        let current_secret_path = self.secret_path()?;
        let current_metadata_path = self.metadata_path()?;
        let dest_secret_path = destination_secret.secret_path()?;
        let dest_metadata_path = destination_secret.metadata_path()?;

        if let Some(parent) = dest_secret_path.parent() {
            secure_create_dir_all(parent, &self.config.secrets_dir)
                .context("Failed to create destination secret directory")?;
        }
        if let Some(parent) = dest_metadata_path.parent() {
            secure_create_dir_all(parent, &self.config.metadata_dir)
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
            config: Arc::clone(&self.config),
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
}
