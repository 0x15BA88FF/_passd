use std::{
    fs::{self, Permissions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

pub fn secure_create_dir_all(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;

    let mut current = PathBuf::new();

    for part in path.components() {
        current.push(part);
        if current.is_dir() {
            fs::set_permissions(&current, Permissions::from_mode(0o700))?;
        }
    }

    Ok(())
}

pub fn secure_write(
    path: &Path,
    content: impl AsRef<[u8]>,
) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::io::Write;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .open(path)?;

    file.write_all(content.as_ref())
}
