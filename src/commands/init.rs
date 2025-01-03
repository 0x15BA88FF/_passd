use std::{fs, io, path::PathBuf};

pub fn init(store_path: PathBuf, pgp_keys: Vec<String>) -> Result<(), io::Error> {
    let mut gpg_id_path = store_path.clone();
    gpg_id_path.push(".gpg-id");

    fs::create_dir_all(&store_path)?;
    fs::write(&gpg_id_path, pgp_keys.join("\n"))?;
    Ok(())
}
