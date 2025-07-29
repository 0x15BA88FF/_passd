use crate::models::{config::Config, metadata::Metadata, secret::Secret};
use anyhow::Result;
use std::{cmp::Ordering, path::PathBuf, sync::Arc};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Secrets {
    pub config: Arc<Config>,
}

impl Secrets {
    pub fn new(config: Arc<Config>) -> Self {
        Self { config }
    }

    pub fn find<F, C>(
        &self,
        filter: Option<F>,
        mut sort: Option<C>,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> Result<Vec<PathBuf>>
    where
        F: Fn(&Metadata) -> bool,
        C: FnMut(&Metadata, &Metadata) -> Ordering,
    {
        let mut results = Vec::new();

        for entry in WalkDir::new(&self.config.metadata_dir)
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
            let relative_path =
                match full_path.strip_prefix(&self.config.metadata_dir) {
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

            base.set_file_name(file_stem);

            let secret = Secret {
                relative_path: base,
                config: Arc::clone(&self.config),
            };

            let metadata = match secret.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            if let Some(ref filter_fn) = filter {
                if !filter_fn(&metadata) {
                    continue;
                }
            }

            results.push(metadata);
        }

        if let Some(ref mut cmp_fn) = sort {
            results.sort_by(|a, b| cmp_fn(a, b));
        }

        let total = results.len();
        let start = offset.unwrap_or(0).min(total);
        let end = limit.map_or(total, |l| (start + l).min(total));

        let response = results[start..end]
            .iter()
            .map(|meta| meta.template.path.clone().unwrap())
            .collect();

        Ok(response)
    }
}
