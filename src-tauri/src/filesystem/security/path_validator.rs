use crate::filesystem::errors::SecurityError;
use std::path::{Path, PathBuf};

// Internal only
pub(crate) struct PathValidator;

impl PathValidator {
    pub(crate) fn validate(path: &Path) -> Result<PathBuf, SecurityError> {
        if path.to_string_lossy().contains("..") {
            return Err(SecurityError::PathTraversal(path.display().to_string()));
        }
        Ok(path.to_path_buf())
    }
}
