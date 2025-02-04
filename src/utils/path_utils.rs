use path_absolutize::Absolutize;
use shellexpand::full;
use std::path::{Path, PathBuf};

pub fn expand_path_str(input: &str) -> Result<PathBuf, String> {
    let expanded = full(input)
        .map_err(|e| format!("Expansion failed: {}", e))?
        .into_owned();

    let absolute_path = Path::new(&expanded)
        .absolutize()
        .map_err(|e| format!("Path resolution failed: {}", e))?
        .to_path_buf();

    Ok(absolute_path)
}
