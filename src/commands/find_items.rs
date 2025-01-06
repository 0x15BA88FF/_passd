use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub fn find_items<F>(path: &Path, filter: &F) -> Result<Vec<PathBuf>, io::Error>
where
    F: Fn(&Path) -> bool,
{
    let mut results = Vec::new();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if filter(&entry_path) {
                results.push(entry_path.clone());
            }

            if entry_path.is_dir() {
                results.extend(find_items(&entry_path, filter)?);
            }
        }
    }

    Ok(results)
}
