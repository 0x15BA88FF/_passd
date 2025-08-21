use crate::{
    models::{config::Config, metadata::Metadata, secret::Secret},
    utils::checksum::compute_checksum_from_content,
    utils::fs::{is_secure_dir, is_secure_file},
};
use anyhow::Result;
use sequoia_openpgp::{Cert, parse::Parse};
use serde::Serialize;
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
    sync::Arc,
};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Secrets {
    pub config: Arc<Config>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticStatus {
    Warning,
    Error,
}

#[derive(Debug, Serialize)]
pub enum IssueType {
    UnexpectedError,
    RougeFile,
    OrphanSecret,
    OrphanMetadata,
    UnsafeFilePermissions,
    UnsafeDirectoryPermissions,
    InvalidMetadata,
    InvalidTimestamps,
    MissingAttachment,
    MetadataChecksumMismatch,
    ModificationCountMismatch,
    SecretPathMismatch,
    SecretChecksumMismatch,
    SecretFingerprintMismatch,
}

#[derive(Debug, Serialize)]
pub struct DiagnosticResult {
    pub status: DiagnosticStatus,
    pub issue: IssueType,
    pub message: String,
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
            .map(|meta| meta.path.clone())
            .collect();

        Ok(response)
    }

    pub fn diagnose(&self) -> Result<Vec<DiagnosticResult>> {
        let mut diagnostics = Vec::new();

        for dir_entry in WalkDir::new(&self.config.metadata_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.file_type().is_dir())
        {
            let dir_path = dir_entry.path();

            if !is_secure_dir(&dir_path) {
                diagnostics.push(DiagnosticResult {
                    status: DiagnosticStatus::Warning,
                    issue: IssueType::UnsafeDirectoryPermissions,
                    message: format!(
                        "Directory '{}' has unsafe permissions",
                        dir_path.display()
                    )
                    .to_string(),
                });
            }

            for file_entry in std::fs::read_dir(dir_path)
                .unwrap()
                .filter_map(Result::ok)
                .filter(|e| e.path().is_file())
            {
                let file_path = file_entry.path();

                if !is_secure_file(&file_path) {
                    diagnostics.push(DiagnosticResult {
                        status: DiagnosticStatus::Warning,
                        issue: IssueType::UnsafeFilePermissions,
                        message: format!(
                            "File '{}' has unsafe permissions",
                            file_path.display()
                        )
                        .to_string(),
                    });
                }

                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default();

                let relative_path = PathBuf::from(
                    file_path
                        .strip_prefix(&self.config.metadata_dir)
                        .unwrap()
                        .to_string_lossy()
                        .strip_suffix(".meta.toml")
                        .unwrap_or_default(),
                );
                let secret = Secret {
                    relative_path: relative_path.clone(),
                    config: Arc::clone(&self.config),
                };

                if self.config.metadata_dir == self.config.secrets_dir {
                    if !file_name.ends_with(".meta.toml") {
                        if !file_name.ends_with(".pgp") {
                            diagnostics.push(DiagnosticResult {
                                status: DiagnosticStatus::Warning,
                                issue: IssueType::RougeFile,
                                message: format!(
                                    "Unexpected file type '{}'",
                                    file_path.display(),
                                ),
                            });

                            continue;
                        } else {
                            if !secret.metadata_path()?.exists() {
                                diagnostics.push(DiagnosticResult {
                                    status: DiagnosticStatus::Warning,
                                    issue: IssueType::OrphanSecret,
                                    message: format!(
                                        "Secret has no corresponding metadata '{}'",
                                        file_path.display(),
                                    ),
                                });
                            }
                        }

                        continue;
                    }
                } else {
                    if !file_name.ends_with(".meta.toml") {
                        diagnostics.push(DiagnosticResult {
                            status: DiagnosticStatus::Warning,
                            issue: IssueType::RougeFile,
                            message: format!(
                                "Unexpected file type '{}'",
                                file_path.display(),
                            ),
                        });

                        continue;
                    }
                }

                let mut metadata = match secret.metadata() {
                    Ok(m) => m,
                    Err(_) => {
                        diagnostics.push(DiagnosticResult {
                            status: DiagnosticStatus::Error,
                            issue: IssueType::InvalidMetadata,
                            message: format!(
                                "Failed to read metadata '{}'",
                                file_path.display(),
                            ),
                        });

                        continue;
                    }
                };

                if metadata.path != secret.relative_path {
                    diagnostics.push(DiagnosticResult {
                        status: DiagnosticStatus::Error,
                        issue: IssueType::SecretPathMismatch,
                        message: format!(
                            "Metadata '{}' has an invalid 'path' value",
                            file_path.display(),
                        ),
                    });
                }

                for attachment in
                    metadata.template.attachments.as_ref().unwrap_or(&vec![])
                {
                    let mut attachment_path = self.config.secrets_dir.clone();
                    attachment_path.push(&attachment);

                    if Path::new(&attachment).exists() {
                        continue;
                    }

                    diagnostics.push(DiagnosticResult {
                        status: DiagnosticStatus::Error,
                        issue: IssueType::MissingAttachment,
                        message: format!(
                            "Failed to find attachment '{}'",
                            attachment_path.display()
                        ),
                    });
                }

                let time_diff =
                    (metadata.updated_at - metadata.created_at).num_seconds();

                match (
                    time_diff < 0,
                    time_diff > 0 && metadata.modifications <= 0,
                ) {
                    (true, _) => {
                        diagnostics.push(DiagnosticResult {
                            status: DiagnosticStatus::Error,
                            issue: IssueType::InvalidTimestamps,
                            message: format!(
                                "Unexpected timestamp difference of '{}'s",
                                time_diff
                            ),
                        });
                    }
                    (false, true) => {
                        diagnostics.push(DiagnosticResult {
                            status: DiagnosticStatus::Error,
                            issue: IssueType::ModificationCountMismatch,
                            message: format!(
                                "Unexpected modification count '{}'",
                                metadata.modifications
                            )
                            .into(),
                        });
                    }
                    (false, false) => {}
                }

                if !secret.secret_path().unwrap().exists() {
                    diagnostics.push(DiagnosticResult {
                        status: DiagnosticStatus::Warning,
                        issue: IssueType::OrphanMetadata,
                        message: "Metadata has no corresponding secret '{}'"
                            .into(),
                    });

                    continue;
                }

                if metadata.fingerprint
                    != Cert::from_reader(
                        secret.plaintext_content()?.as_bytes(),
                    )?
                    .fingerprint()
                    .to_hex()
                    .to_uppercase()
                {
                    diagnostics.push(DiagnosticResult {
                        status: DiagnosticStatus::Error,
                        issue: IssueType::SecretFingerprintMismatch,
                        message: "Secret fingerprint does not match metadata fingerprint '{}'"
                            .into(),
                    });
                }

                if let Ok(plaintext) = secret.plaintext_content() {
                    let computed_checksum =
                        compute_checksum_from_content(&plaintext);
                    if computed_checksum != metadata.checksum_main {
                        diagnostics.push(DiagnosticResult {
                            status: DiagnosticStatus::Error,
                            issue: IssueType::SecretChecksumMismatch,
                            message: format!(
                                "Secret checksum mismatch for '{}'",
                                file_path.display(),
                            ),
                        });
                    }
                }

                metadata.checksum_meta = "".to_string();

                if compute_checksum_from_content(
                    &toml::to_string(&metadata).unwrap(),
                ) != metadata.checksum_meta
                {
                    diagnostics.push(DiagnosticResult {
                        status: DiagnosticStatus::Error,
                        issue: IssueType::MetadataChecksumMismatch,
                        message: format!(
                            "Metadata checksum mismatch for '{}'",
                            file_path.display(),
                        ),
                    });
                }
            }
        }

        if self.config.metadata_dir != self.config.secrets_dir {
            for dir_entry in WalkDir::new(&self.config.secrets_dir)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_dir())
            {
                let dir_path = dir_entry.path();

                if !is_secure_dir(&dir_path) {
                    diagnostics.push(DiagnosticResult {
                        status: DiagnosticStatus::Warning,
                        issue: IssueType::UnsafeDirectoryPermissions,
                        message: format!(
                            "Directory '{}' has unsafe permissions",
                            dir_path.display()
                        ),
                    });
                }

                for file_entry in std::fs::read_dir(dir_path)
                    .unwrap()
                    .filter_map(Result::ok)
                    .filter(|e| e.path().is_file())
                {
                    let file_path = file_entry.path();

                    if !is_secure_file(&file_path) {
                        diagnostics.push(DiagnosticResult {
                            status: DiagnosticStatus::Warning,
                            issue: IssueType::UnsafeFilePermissions,
                            message: format!(
                                "File '{}' has unsafe permissions",
                                file_path.display()
                            ),
                        });
                    }

                    let file_name = file_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or_default();

                    if !file_name.ends_with(".pgp") {
                        diagnostics.push(DiagnosticResult {
                            status: DiagnosticStatus::Warning,
                            issue: IssueType::RougeFile,
                            message: format!(
                                "Unexpected file type '{}'",
                                file_path.display(),
                            ),
                        });

                        continue;
                    }

                    let relative_path = PathBuf::from(
                        file_path
                            .strip_prefix(&self.config.secrets_dir)
                            .unwrap()
                            .to_string_lossy()
                            .strip_suffix(".pgp")
                            .unwrap(),
                    );

                    let secret = Secret {
                        relative_path,
                        config: Arc::clone(&self.config),
                    };

                    if !secret.metadata_path()?.exists() {
                        diagnostics.push(DiagnosticResult {
                            status: DiagnosticStatus::Warning,
                            issue: IssueType::OrphanSecret,
                            message: format!(
                                "Secret has no corresponding metadata '{}'",
                                file_path.display(),
                            ),
                        });
                    }
                }
            }
        }

        Ok(diagnostics)
    }
}
