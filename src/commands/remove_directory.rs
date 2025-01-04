use std::{fs, io, path::Path};

pub fn remove_directory(path: &Path, force: Option<bool>) -> Result<(), io::Error> {
    if path.is_dir() {
        if path.read_dir()?.next().is_some() {
            if force.unwrap_or(false) {
                fs::remove_dir_all(path)?;
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::DirectoryNotEmpty,
                    "Non-empty directory cannot be removed without force",
                ));
            }
        } else {
            fs::remove_dir(path)?;
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The specified path is not a directory or does not exist",
        ));
    }

    Ok(())
}
