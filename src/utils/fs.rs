use std::{
    fs::{self, Permissions},
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::Path,
};

pub fn secure_create_dir_all(
    path: &Path,
    base_path: &Path,
) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    let relative_path = match path.strip_prefix(base_path) {
        Ok(rel) => rel,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Path is not under the specified base path",
            ));
        }
    };
    let mut current = base_path.to_path_buf();
    for component in relative_path.components() {
        current.push(component);
        if current.exists() && current.is_dir() {
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

pub fn is_secure_dir(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.is_dir() {
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;

            return mode != 0o700;
        }
    }

    true
}

pub fn is_secure_file(path: &Path) -> bool {
    if let Ok(metadata) = fs::metadata(path) {
        if metadata.is_file() {
            let permissions = metadata.permissions();
            let mode = permissions.mode() & 0o777;

            return mode != 0o600;
        }
    }

    true
}

pub fn set_secure_dir_permissions<P: AsRef<Path>>(
    path: P,
) -> std::io::Result<()> {
    let perms = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, perms)
}

pub fn set_secure_file_permissions<P: AsRef<Path>>(
    path: P,
) -> std::io::Result<()> {
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)
}
