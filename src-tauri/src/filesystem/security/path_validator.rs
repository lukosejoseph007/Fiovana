use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::permissions::PermissionsRationale;
use std::path::{Path, PathBuf};

// 🔹 Stub module for future permission escalation
mod permissions_escalation {
    pub struct PermissionEscalation;

    impl PermissionEscalation {
        /// Returns true if escalation is allowed for the given operation
        pub fn can_escalate(_operation: &str) -> bool {
            // Currently always true; implement real logic in the future
            true
        }
    }
}

use permissions_escalation::PermissionEscalation;

#[derive(Clone)]
pub(crate) struct PathValidator {
    config: SecurityConfig,
}

impl PathValidator {
    // Constructor
    pub(crate) fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    // Instance method for validating paths
    pub(crate) fn validate_import_path(&self, path: &Path) -> Result<PathBuf, SecurityError> {
        let path_str = path.to_string_lossy();

        // 1. Path length check
        if path_str.len() > self.config.max_path_length {
            return Err(SecurityError::PathTooLong {
                length: path_str.len(),
                max: self.config.max_path_length,
            });
        }

        // 2. Path traversal check
        if path_str.contains("..") {
            return Err(SecurityError::PathTraversal {
                path: path.display().to_string(),
            });
        }

        // 3. Prohibited characters check
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if filename
            .chars()
            .any(|c| self.config.prohibited_filename_chars.contains(&c))
        {
            return Err(SecurityError::ProhibitedCharacters {
                filename: filename.to_string(),
            });
        }

        // 4. Extension check
        let ext_with_dot = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()))
            .unwrap_or_else(|| "".to_string());

        if !self.config.allowed_extensions.contains(&ext_with_dot) {
            return Err(SecurityError::InvalidExtension {
                extension: ext_with_dot,
            });
        }

        // 🔹 Placeholder for future permission escalation
        if !PermissionEscalation::can_escalate("import_path") {
            // Currently does nothing, but ready for future rules
        }

        Ok(path.to_path_buf())
    }

    // Returns the permissions rationale
    pub fn permission_rationale(&self) -> &'static str {
        PermissionsRationale::explain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use std::thread;

    fn default_config() -> SecurityConfig {
        SecurityConfig::default()
    }

    #[test]
    fn test_valid_path() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/Users/test/document.txt");
        assert!(validator.validate_import_path(path).is_ok());
    }

    #[test]
    fn test_path_too_long() {
        let validator = PathValidator::new(default_config());
        let long_path = "a".repeat(300);
        let path = Path::new(&long_path);
        assert!(matches!(
            validator.validate_import_path(path),
            Err(SecurityError::PathTooLong { .. })
        ));
    }

    #[test]
    fn test_prohibited_characters() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/temp/inva|id.txt");
        assert!(matches!(
            validator.validate_import_path(path),
            Err(SecurityError::ProhibitedCharacters { .. })
        ));
    }

    #[test]
    fn test_invalid_extension() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/temp/file.exe");
        assert!(matches!(
            validator.validate_import_path(path),
            Err(SecurityError::InvalidExtension { .. })
        ));
    }

    #[test]
    fn test_path_traversal() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/Users/test/../Windows/System32/config.sys");
        assert!(matches!(
            validator.validate_import_path(path),
            Err(SecurityError::PathTraversal { .. })
        ));
    }

    #[test]
    fn test_empty_extension() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/temp/file");
        assert!(matches!(
            validator.validate_import_path(path),
            Err(SecurityError::InvalidExtension { .. })
        ));
    }

    #[test]
    fn test_mixed_case_extension() {
        let validator = PathValidator::new(default_config());
        let path = Path::new("C:/temp/Document.TXT");
        assert!(
            validator.validate_import_path(path).is_ok(),
            "Mixed-case extensions should be valid"
        );
    }

    #[test]
    fn test_concurrent_access() {
        let validator = PathValidator::new(default_config());
        let paths = vec![
            "C:/Users/test/doc1.txt",
            "C:/Users/test/doc2.txt",
            "C:/Users/test/doc3.txt",
        ];

        let handles: Vec<_> = paths
            .into_iter()
            .map(|p| {
                let validator = validator.clone();
                thread::spawn(move || {
                    let path = Path::new(p);
                    assert!(validator.validate_import_path(path).is_ok());
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
