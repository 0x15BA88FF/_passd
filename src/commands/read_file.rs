use std::{fs, io, path::Path};

pub fn read_file(path: &Path) -> Result<String, io::Error> {
    if !path.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The specified path is not a file or does not exist",
        ));
    }

    let content = fs::read_to_string(path)?;

    Ok(content)
}
