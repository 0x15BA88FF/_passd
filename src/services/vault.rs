use crate::{
    configs::load_config,
    models::{metadata::Metadata, secret::Secret},
}
use log::{debug, info, warn};
use std::{cmp::Ordering, error::Error, path::{Path, PathBuf}};
use walkdir::WalkDir;

pub fn find<F, C>(
    filter: Option<F>,
    mut sort: Option<C>,
    skip: Option<usize>,
    limit: Option<usize>,
) -> Result<Vec<SecretResponse>, Box<dyn Error>>
where
    F: Fn(&Metadata) -> bool,
    C: FnMut(&Metadata, &Metadata) -> Ordering,
{
    let config = load_config()?;
    let mut results = Vec::new();

    for entry in WalkDir::new(config.metadata_path())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| name.ends_with(".meta.toml"))
                    .unwrap_or(false)
        })
    {
        let full_path = entry.path();
        let relative_path = match full_path.strip_prefix(config.metadata_path()) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let file_stem = match relative_path
            .file_name()
            .and_then(|f| f.to_str())
            .and_then(|f| f.strip_suffix(".meta.toml"))
        {
            Some(stem) => stem,
            None => continue,
        };
        let mut base = relative_path.to_path_buf();

        base.set_file_name(stem);

        let content = match std::fs::read_to_string(full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let metadata: Metadata = match toml::from_str(&content) {
            Ok(m) => m,
            Err(_) => continue,
        };
        if let Some(ref filter_fn) = filter {
            if !filter_fn(&metadata) {
                continue;
            }
        }

        results.push((base, metadata));
    }

    if let Some(ref mut cmp_fn) = sort {
        results.sort_by(|a, b| cmp_fn(&a.1, &b.1));
    }
    let total = results.len();
    let start = skip.unwrap_or(0).min(total);
    let end = limit.map_or(total, |l| (start + l).min(total));
    let response: Vec<SecretResponse> = results[start..end]
        .iter()
        .map(|(path, metadata)| SecretResponse {
            path: path.to_string_lossy().to_string(),
            metadata: metadata.clone(),
        })
        .collect();

    Ok(response)
}
