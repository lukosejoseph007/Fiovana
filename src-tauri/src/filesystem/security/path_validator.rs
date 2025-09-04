use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use std::path::{Path, PathBuf};

pub(crate) struct PathValidator;

impl PathValidator {
    pub(crate) fn validate(path: &Path, config: &SecurityConfig) -> Result<PathBuf, SecurityError> {
        // 1. Path length check
        let path_str = path.to_string_lossy();
        if path_str.len() > config.max_path_length {
            return Err(SecurityError::PathTooLong {
                length: path_str.len(),
                max: config.max_path_length,
            });
        }

        // 2. Prohibited characters
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        for c in filename.chars() {
            if config.prohibited_filename_chars.contains(&c) {
                return Err(SecurityError::ProhibitedCharacters {
                    filename: filename.to_string(),
                });
            }
        }

        // 3. Path traversal check
        if path_str.contains("..") {
            return Err(SecurityError::PathTraversal {
                path: path.display().to_string(),
            });
        }

        // 4. Extension check
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            let ext_with_dot = format!(".{}", ext.to_lowercase());
            if !config.allowed_extensions.contains(&ext_with_dot) {
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
        let path = Path::new("C:/Users/test/document.txt");
        let result = PathValidator::validate(path, &default_config());
        assert!(result.is_ok());
    }

    #[test]
    fn test_path_too_long() {
        let long_path = "a".repeat(300);
        let path = Path::new(&long_path);
        let result = PathValidator::validate(path, &default_config());
        assert!(matches!(result, Err(SecurityError::PathTooLong { .. })));
    }

    #[test]
    fn test_prohibited_characters() {
        let path = Path::new("C:/temp/inva|id.txt");
        let result = PathValidator::validate(path, &default_config());
        assert!(matches!(
            result,
            Err(SecurityError::ProhibitedCharacters { .. })
        ));
    }

    #[test]
    fn test_invalid_extension() {
        let path = Path::new("C:/temp/file.exe");
        let result = PathValidator::validate(path, &default_config());
        assert!(matches!(
            result,
            Err(SecurityError::InvalidExtension { .. })
        ));
    }
}
