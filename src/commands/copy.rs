use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn copy(source: &Path, destination: &Path, recursive: Option<bool>) -> Result<(), io::Error> {
    if !source.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "The source directory / file does not exist.",
        ));
    }

    if let Some(parent) = destination.parent() {
        if !parent.exists() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "The destination directory does not exist.",
            ));
        }
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "The destination directory does not exist.",
        ));
    }

    if source.is_file() {
        let destination = if destination.is_dir() {
            destination.join(source.file_name().unwrap())
        } else {
            destination.to_path_buf()
        };

        fs::copy(source, destination)?;
    } else {
        if let Some(true) = recursive {
            if !destination.exists() {
                fs::create_dir_all(destination)?;
            }

            for entry in fs::read_dir(source)? {
                let entry = entry?;
                let entry_path = entry.path();
                let mut dest_path = PathBuf::from(destination);
                dest_path.push(entry.file_name());

                copy(&entry_path, &dest_path, recursive)?;
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Cannot copy directory without resursive parameter.",
            ));
        }
    }

    Ok(())
}
