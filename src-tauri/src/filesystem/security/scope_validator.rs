use crate::filesystem::errors::SecurityError;
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct ScopeValidator {
    #[allow(dead_code)]
    allowed_paths: Vec<PathBuf>,
}

#[allow(dead_code)]
impl ScopeValidator {
    pub fn new(allowed_paths: Vec<PathBuf>) -> Self {
        Self { allowed_paths }
    }

    /// Validate that a path is inside the allowed scope.
    pub fn validate(&self, path: &Path) -> Result<(), SecurityError> {
        if !self
            .allowed_paths
            .iter()
            .any(|allowed| self.path_starts_with(path, allowed))
        {
            return Err(SecurityError::PathOutsideWorkspace {
                path: path.display().to_string(),
            });
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use crate::filesystem::security::path_validator::validate_multiple_extensions;

    #[test]
    fn test_validate_multiple_extensions_valid_cases() {
        let allowed_extensions = vec![
            ".txt".to_string(),
            ".md".to_string(),
            ".pdf".to_string(),
            ".csv".to_string(),
            ".docx".to_string(),
            ".json".to_string(),
            ".zip".to_string(),
            ".tar.gz".to_string(),
        ];

        assert!(validate_multiple_extensions(
            "archive.tar.gz",
            &allowed_extensions
        ));
        assert!(validate_multiple_extensions(
            "data.backup.zip",
            &allowed_extensions
        ));
        assert!(validate_multiple_extensions(
            "file.txt",
            &allowed_extensions
        ));
    }

    #[test]
    fn test_validate_multiple_extensions_invalid_cases() {
        let allowed_extensions = vec![
            ".txt".to_string(),
            ".md".to_string(),
            ".pdf".to_string(),
            ".csv".to_string(),
            ".docx".to_string(),
            ".json".to_string(),
            ".zip".to_string(),
            ".tar.gz".to_string(),
        ];

        assert!(!validate_multiple_extensions(
            "malicious.bin.exe",
            &allowed_extensions
        ));
        assert!(!validate_multiple_extensions(
            "unsafe.zip.rar",
            &allowed_extensions
        ));
        assert!(!validate_multiple_extensions(
            "double..dots",
            &allowed_extensions
        ));
    }
}
