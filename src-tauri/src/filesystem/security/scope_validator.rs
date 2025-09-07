use crate::filesystem::errors::SecurityError;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ScopeValidator {
    allowed_paths: Vec<PathBuf>,
}

impl ScopeValidator {
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    /// Validate that a path is inside the allowed scope.
    pub fn validate(&self, path: &Path) -> Result<(), SecurityError> {
        for allowed_path in &self.allowed_paths {
            if self.path_starts_with(path, allowed_path) {
                return Ok(());
            }
        }
        Err(SecurityError::PathOutsideWorkspace {
            path: path.to_string_lossy().to_string(),
        })
    }

    #[cfg(target_os = "windows")]
    fn path_starts_with(&self, path: &Path, prefix: &Path) -> bool {
        path.to_string_lossy()
            .to_lowercase()
            .starts_with(&prefix.to_string_lossy().to_lowercase())
    }

    #[cfg(not(target_os = "windows"))]
    fn path_starts_with(&self, path: &Path, prefix: &Path) -> bool {
        path.starts_with(prefix)
    }
}
