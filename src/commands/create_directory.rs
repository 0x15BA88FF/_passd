use std::{fs, io, path::Path};

pub fn create_directory(path: &Path) -> Result<(), io::Error> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}
