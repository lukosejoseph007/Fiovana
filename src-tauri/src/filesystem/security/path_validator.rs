use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use std::path::{Path, PathBuf};

pub(crate) struct PathValidator {
    config: SecurityConfig,
}

impl PathValidator {
    pub(crate) fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    pub(crate) fn validate_import_path(&self, path: &Path) -> Result<PathBuf, SecurityError> {
        // reuse your existing logic
        let path_str = path.to_string_lossy();
        if path_str.len() > self.config.max_path_length {
            return Err(SecurityError::PathTooLong {
                length: path_str.len(),
                max: self.config.max_path_length,
            });
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        for c in filename.chars() {
            if self.config.prohibited_filename_chars.contains(&c) {
                return Err(SecurityError::ProhibitedCharacters {
                    filename: filename.to_string(),
                });
            }
        }

        if path_str.contains("..") {
            return Err(SecurityError::PathTraversal {
                path: path.display().to_string(),
            });
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_with_dot = format!(".{}", ext.to_lowercase());
            if !self.config.allowed_extensions.contains(&ext_with_dot) {
                return Err(SecurityError::InvalidExtension {
                    extension: ext_with_dot,
                });
            }
        } else {
            return Err(SecurityError::InvalidExtension {
                extension: "".into(),
            });
        }

        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn default_config() -> SecurityConfig {
        SecurityConfig::default()
    }

    #[test]
    fn test_valid_path() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/Users/test/document.txt");
        let result = validator.validate_import_path(path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_too_long() {
        let validator = PathValidator::new(default_config());
        let long_path = "a".repeat(300);
        let path = Path::new(&long_path);
        let result = validator.validate_import_path(path);
        assert!(matches!(result, Err(SecurityError::PathTooLong { .. })));
    }

    #[test]
    fn test_prohibited_characters() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/temp/inva|id.txt");
        let result = validator.validate_import_path(path);
        assert!(matches!(
            result,
            Err(SecurityError::ProhibitedCharacters { .. })
        ));
    }

    #[test]
    fn test_invalid_extension() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/temp/file.exe");
        let result = validator.validate_import_path(path);
        assert!(matches!(
            result,
            Err(SecurityError::InvalidExtension { .. })
        ));
    }
}
