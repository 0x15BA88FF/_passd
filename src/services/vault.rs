use crate::{
    configs::load_config,
    models::{metadata::Metadata, secret::Secret},
};
use std::{cmp::Ordering, error::Error, path::{Path, PathBuf}};
use walkdir::WalkDir;

pub fn find_by_path(path: &Path) -> Result<Secret, Box<dyn Error>> {
    let config = load_config()?;

    let secret_path = config.vault_dir().join(path)
        .with_extension("pgp");
    let metadata_path = config.metadata_path().join(path)
        .with_extension("meta.toml");

    if !secret_path.exists() || !metadata_path.exists() {
        return Err(format!(
            "Secret or metadata not found for path: {}",
            path.display()
        ).into());
    }

    Ok(Secret {
        secret_path,
        metadata_path,
    })
}

pub fn find<F, C>(
    filter: F,
    mut sort: Option<C>,
    skip: usize,
    limit: Option<usize>,
) -> Result<Vec<Secret>, Box<dyn Error>>
where
    F: Fn(&Secret) -> bool,
    C: FnMut(&Secret, &Secret) -> Ordering,
{
    let config = load_config()?;
    let mut results = Vec::new();

    for entry in WalkDir::new(config.metadata_path())
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && e.path()
                .file_name()
                .and_then(|n| n.to_str())
                .map(|name| name.ends_with(".meta.toml"))
                .unwrap_or(false)
        })
    {
        let metadata_path = entry.path().to_path_buf();

        let secret_path = {
            let relative = match metadata_path.strip_prefix(
                config.metadata_path()
            ) {
                Ok(r) => r,
                Err(_) => continue,
            };

            let base = match metadata_path
                .file_name()
                .and_then(|f| f.to_str())
                .and_then(|f| f.strip_suffix(".meta.toml"))
            {
                Some(base_name) => relative.with_file_name(base_name),
                None => continue,
            };

            let mut path = config.vault_dir().join(base);

            path.set_extension("pgp");

            if !path.exists() {
                continue;
            }

            path
        };

        let secret = Secret {
            secret_path,
            metadata_path,
        };

        if filter(&secret) {
            results.push(secret);
        }
    }

    if let Some(ref mut cmp) = sort {
        results.sort_by(cmp);
    }

    let total = results.len();
    let start = skip.min(total);
    let end = limit.map_or(total, |l| (start + l).min(total));

    Ok(results[start..end].to_vec())
}
