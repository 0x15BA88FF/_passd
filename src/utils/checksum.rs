use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

pub fn compute_checksum(path: &Path) -> Result<String> {
    let data = fs::read(path).with_context(|| {
        format!("Failed to read file for checksum: {}", path.display())
    })?;
    let hash = Sha256::digest(&data);

    Ok(format!("{:x}", hash))
}
