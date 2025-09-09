use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};

use anyhow::{Context, Result};
use chrono::Utc;
use log;
use sequoia_openpgp::{
    Cert, KeyHandle, KeyID, Message, Packet, Result as SequoiaResult,
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
        Armorer, Encryptor, LiteralWriter, Message as StreamMessage, Recipient,
    },
};
use toml;

use crate::{
    models::{
        config::Config,
        key_manager::KeyManager,
        metadata::{BaseMetadata, Metadata},
    },
    utils::{
        checksum::compute_checksum,
        fs::{secure_create_dir_all, secure_write},
    },
};

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

    fn recipient_certs(&self, key_manager: &KeyManager) -> Result<Vec<Cert>> {
        if !self.secret_path()?.exists() {
            return Err(anyhow::anyhow!("Secret file does not exist"));
        }

        let content = self.content()?;
        let message: Message = content
            .parse()
            .or_else(|_| Message::from_bytes(content.as_bytes()))
            .context("Failed to parse secret as message")?;
        let mut recipient_certs = Vec::new();

        for pkt in message.packets().descendants() {
            if let Packet::PKESK(pkesk) = pkt {
                let keyid: KeyID = pkesk.recipient().into();

                if let Some(cert) = key_manager.find_cert_by_keyid(&keyid) {
                    recipient_certs.push(cert);
                }
            }
        }

        Ok(recipient_certs)
    }

    fn unlock_keypair(
        &self,
        cert: &Cert,
        password: &str,
    ) -> Result<Option<KeyPair>> {
        let policy = &StandardPolicy::new();

        if let Some(key) = cert
            .keys()
            .secret()
            .with_policy(policy, None)
            .alive()
            .revoked(false)
            .for_storage_encryption()
            .next()
        {
            let kp = key
                .key()
                .clone()
                .parts_into_secret()
                .context("Failed to get secret key parts")?
                .decrypt_secret(&Password::from(password.to_string()));

            if let Ok(decrypted_secret) = kp {
                return Ok(Some(
                    decrypted_secret
                        .into_keypair()
                        .context("Failed to build keypair")?,
                ));
            }
        }

        Ok(None)
    }

    fn decrypt_with_keypair(&self, keypair: &KeyPair) -> Result<String> {
        let policy = &StandardPolicy::new();
        let helper = DecryptHelper::new(keypair.clone());
        let ciphertext = self.content()?;
        let mut decryptor = DecryptorBuilder::from_bytes(ciphertext.as_bytes())
            .context("Failed to create decryptor from ciphertext")?
            .with_policy(policy, None, helper)
            .context("Failed to configure decryptor with policy")?;
        let mut plaintext_buf: Vec<u8> = Vec::new();

        io::copy(&mut decryptor, &mut plaintext_buf)
            .context("Failed to decrypt content with keypair")?;

        let plaintext = String::from_utf8(plaintext_buf)
            .context("Decrypted content is not valid UTF-8")?;

        Ok(plaintext)
    }

    fn encrypt_with_certs(
        &self,
        content: &str,
        certs: &[Cert],
    ) -> Result<Vec<u8>> {
        let policy = &StandardPolicy::new();
        let mut recipients: Vec<Recipient> = Vec::new();

        for cert in certs {
            let new_recipients: Vec<Recipient> = cert
                .keys()
                .with_policy(policy, None)
                .alive()
                .revoked(false)
                .for_storage_encryption()
                .map(|ka| ka.into())
                .collect();

            recipients.extend(new_recipients);
        }

        if recipients.is_empty() {
            return Err(anyhow::anyhow!("No suitable encryption key found"));
        }

        let mut encrypted = Vec::new();
        let message = StreamMessage::new(&mut encrypted);
        let message = Armorer::new(message)
            .build()
            .context("Failed to armor message")?;
        let message = Encryptor::for_recipients(message, recipients)
            .build()
            .context("Failed to build encryptor")?;
        let mut message = LiteralWriter::new(message)
            .build()
            .context("Failed to build literal writer")?;

        message
            .write_all(content.as_bytes())
            .context("Failed to write plaintext")?;
        message
            .finalize()
            .context("Failed to finalize encryption")?;

        Ok(encrypted)
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

    pub fn metadata(&self) -> Result<Metadata> {
        let metadata_path = self.metadata_path()?;
        let text = fs::read_to_string(&metadata_path).with_context(|| {
            format!("Failed to read metadata from {}", metadata_path.display())
        })?;

        Ok(toml::from_str(&text).context("Failed to parse metadata TOML")?)
    }

    pub fn content(&self) -> Result<String> {
        let secret_path = self.secret_path()?;

        fs::read_to_string(&secret_path).with_context(|| {
            format!("Failed to read plaintext from {}", secret_path.display())
        })
    }

    pub fn plaintext_content(&self, password: &str) -> Result<String> {
        let key_manager = KeyManager {
            config: self.config.clone(),
        };

        for cert in self.recipient_certs(&key_manager).unwrap_or(vec![]) {
            let maybe_keypair = self.unlock_keypair(&cert, password)?;
            let keypair = match maybe_keypair {
                Some(kp) => kp,
                None => {
                    return Err(anyhow::anyhow!(
                        "Failed to unlock keypair with provided password"
                    ));
                }
            };
            let plaintext = self.decrypt_with_keypair(&keypair)?;

            log::info!(
                "Successfully decrypted secret: {}",
                self.relative_path.display()
            );

            return Ok(plaintext);
        }

        Err(anyhow::anyhow!("Failed to decrypt secret"))
    }

    pub fn create(
        &self,
        content: &str,
        metadata: &BaseMetadata,
        fingerprints: &[&str],
    ) -> Result<&Self> {
        let secret_path = self.secret_path()?;
        let metadata_path = self.metadata_path()?;

        if secret_path.exists() || metadata_path.exists() {
            return Err(anyhow::anyhow!(
                "Secret or metadata file already exists"
            ));
        }

        let certs = fingerprints
            .iter()
            .map(|fp| {
                KeyManager {
                    config: Arc::clone(&self.config),
                }
                .get_public_cert(fp)
            })
            .collect::<Result<Vec<_>>>()?;
        let encrypted = self.encrypt_with_certs(content, certs)?;
        let encrypted_str = String::from_utf8(encrypted.clone())
            .context("Encrypted data is not valid UTF-8")?;
        let checksum_main = compute_checksum(&encrypted_str);
        let mut meta = Metadata {
            path: self.relative_path.clone(),
            checksum_main,
            checksum_meta: String::new(),
            ..metadata.clone().into()
        };
        let mut metadata_str = toml::to_string_pretty(&meta)
            .context("Failed to serialize metadata")?;

        meta.checksum_meta = compute_checksum(&metadata_str);
        metadata_str = toml::to_string_pretty(&meta)
            .context("Failed to serialize metadata")?;

        // observe
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

        secure_write(&secret_path, encrypted)
            .context("Failed to write encrypted secret file")?;

        secure_write(&metadata_path, metadata_str)
            .context("Failed to write updated metadata file")?;

        log::info!("Created secret: {}", self.relative_path.display());

        Ok(self)
    }

    pub fn update(
        &self,
        content: Option<&str>,
        metadata: Option<&BaseMetadata>,
        fingerprints: Option<&[&str]>,
        password: &str,
    ) -> Result<&Self> {
        if content.is_none() && metadata.is_none() && fingerprints.is_none() {
            return Err(anyhow::anyhow!("No Changes were mode"));
        }

        let secret_path = self.secret_path()?;
        let metadata_path = self.metadata_path()?;

        if !secret_path.exists() || !metadata_path.exists() {
            return Err(anyhow::anyhow!(
                "Secret or metadata file does not exist"
            ));
        }

        if let Some(kfs) = fingerprints {
            if kfs.is_empty() {
                return Err(anyhow::anyhow!(
                    "Provided recipients must not be empty"
                ));
            }
        }

        let key_manager = KeyManager {
            config: Arc::clone(&self.config),
        };
        let exsting_certificates = self.recipient_certs(&key_manager)?;

        if exsting_certificates.is_empty() {
            return Err(anyhow::anyhow!(
                "No recipients found in encrypted secret"
            ));
        }

        let mut unlocked_key_pair: Option<KeyPair> = None;

        for exsting_certificate in exsting_certificates {
            let key_pair = self.unlock_keypair(&exsting_certificate, password);

            if key_pair.is_ok() {
                unlocked_key_pair = key_pair.unwrap_or(None);
                break;
            }
        }

        if unlocked_key_pair.is_none() {
            return Err(anyhow::anyhow!(
                "Password could not unlock any recipient key"
            ));
        }

        let mut updated_metadata = self
            .metadata()
            .context("Failed to read existing metadata for merging")?;
        let mut metadata_str = String::new();

        if let Some(base) = metadata {
            updated_metadata = updated_metadata.merge(
                &base.clone().into()
            ).context(
                "Failed to merge provided BaseMetadata into existing metadata",
            )?;
        }

        updated_metadata.checksum_meta = String::new();
        updated_metadata.updated_at = Utc::now();

        if content.is_some() || fingerprints.is_some() {
            let mut updated_recipient_certs = exsting_certificates;
            let mut updated_content = content.unwrap_or_default().to_string();

            if fingerprints.is_some() {
                updated_recipient_certs = fingerprints
                    .iter()
                    .map(|fp| key_manager.get_public_cert(fp))
                    .collect::<Result<Vec<_>>>()?;
            }

            if content.is_none() {
                updated_content =
                    self.decrypt_with_keypair(&unlocked_key_pair.unwrap())?;
            } else {
                updated_metadata.modifications =
                    updated_metadata.modifications.saturating_add(1);
            }

            let encrypted = self.encrypt_with_certs(
                &updated_content,
                &updated_recipient_certs,
            )?;
            let encrypted_str = String::from_utf8(encrypted.clone())
                .context("Encrypted data is not valid UTF-8")?;

            updated_metadata.checksum_main = compute_checksum(&encrypted_str);
            metadata_str = toml::to_string_pretty(&updated_metadata)
                .context("Failed to serialize metadata")?;
            updated_metadata.checksum_meta = compute_checksum(&metadata_str);
            metadata_str = toml::to_string_pretty(&updated_metadata)
                .context("Failed to serialize metadata")?;

            secure_write(&secret_path, encrypted)
                .context("Failed to write encrypted secret file")?;
        }

        secure_write(&metadata_path, metadata_str)
            .context("Failed to write updated metadata file")?;

        log::info!("Updated secret: {}", self.relative_path.display());

        Ok(self)
    }

    pub fn remove(&self, password: &str) -> Result<()> {
        let key_manager = KeyManager {
            config: Arc::clone(&self.config),
        };
        let exsting_certificates = self.recipient_certs(&key_manager)?;

        if exsting_certificates.is_empty() {
            return Err(anyhow::anyhow!(
                "No recipients found in encrypted secret"
            ));
        }

        let mut unlocked_key_pair: Option<KeyPair> = None;

        for exsting_certificate in exsting_certificates {
            let key_pair = self.unlock_keypair(&exsting_certificate, password);

            if key_pair.is_ok() {
                unlocked_key_pair = key_pair.unwrap_or(None);
                break;
            }
        }

        if unlocked_key_pair.is_none() {
            return Err(anyhow::anyhow!(
                "Password could not unlock any recipient key"
            ));
        }

        for path in [self.secret_path()?, self.metadata_path()?] {
            match fs::remove_file(&path) {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    log::warn!("File not found: {}", path.display());
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to remove {}: {}",
                        path.display(),
                        e
                    ));
                }
            }
        }

        log::info!("Removed secret: {}", self.relative_path.display());

        Ok(())
    }

    pub fn move_to(&self, destination: PathBuf) -> Result<Secret> {
        // TODO update metadata path
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

        fs::rename(&current_secret_path, &dest_secret_path)
            .context("Failed to move secret file")?;
        fs::rename(&current_metadata_path, &dest_metadata_path)
            .context("Failed to move metadata file")?;

        Ok(destination_secret)
    }

    pub fn copy_to(&self, destination: PathBuf) -> Result<Secret> {
        // TODO update metadata path
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

        fs::copy(&current_secret_path, &dest_secret_path)
            .context("Failed to copy secret file")?;
        fs::copy(&current_metadata_path, &dest_metadata_path)
            .context("Failed to copy metadata file")?;

        Ok(destination_secret)
    }
}
