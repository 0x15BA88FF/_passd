use std::{fs, io, path::Path};

pub fn remove_file(path: &Path) -> Result<(), io::Error> {
    if path.is_file() {
        fs::remove_file(path)?;
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The specified path is not a file or does not exist",
        ));
    };

    Ok(())
}
