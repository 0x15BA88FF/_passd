use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct EntryInfo {
    pub name: String,
    pub path: PathBuf,
    pub r#type: EntryType,
    pub children: Option<Vec<EntryInfo>>,
}

#[derive(PartialEq, Debug)]
pub enum EntryType {
    File,
    Directory,
}

pub fn list_items(path: &Path, recursive: Option<bool>) -> Result<Vec<EntryInfo>, io::Error> {
    let mut items = Vec::new();
    let recursive = recursive.unwrap_or(false);

    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Provided path is not a directory",
        ));
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        let entry_type = if metadata.is_dir() {
            EntryType::Directory
        } else {
            EntryType::File
        };

        let children = if recursive && entry_type == EntryType::Directory {
            Some(list_items(&entry.path(), Some(true))?)
        } else {
            None
        };

        items.push(EntryInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path(),
            r#type: entry_type,
            children,
        });
    }

    Ok(items)
}
