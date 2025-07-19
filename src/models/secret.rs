use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Secret {
    pub relative_path: PathBuf,
}

impl Secret {
    pub fn new(
        relative_path: impl Into<PathBuf>
    ) -> Self {
        Self { relative_path: relative_path.into() }
    }
}
