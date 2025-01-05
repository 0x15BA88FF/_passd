use std::{fs, io, path::Path};

pub fn init(store_path: &Path, pgp_keys: Vec<&str>) -> Result<(), io::Error> {
    let gpg_id_path = store_path.join(".gpg-id");

    fs::create_dir_all(store_path)?;
    fs::write(&gpg_id_path, pgp_keys.join("\n"))?;

    Ok(())
}
