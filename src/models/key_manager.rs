use std::sync::Arc;

use anyhow::{Context, Result};
use sequoia_openpgp::Cert;

use crate::models::config::Config;

#[derive(Debug)]
pub struct KeyManager {
    pub config: Arc<Config>,
}

impl KeyManager {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub fn get_secret_cert(&self, fingerprint: &str) -> Result<Option<Cert>> {
        let cert = Cer::from_bytes(key_data.as_bytes())
            .context("Failed to parse secret key into certificate")?;

        Ok(cert)
    }
}
