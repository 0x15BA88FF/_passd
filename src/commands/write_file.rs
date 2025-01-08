use std::{fs, io, path::Path};

pub fn write_file(path: &Path, content: &str) -> Result<(), io::Error> {
    if path.exists() && !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The specified path exists but is not a file",
        ));
    }

    fs::write(path, content)?;

    Ok(())
}
