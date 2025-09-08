use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::permissions::PermissionsRationale;
use crate::filesystem::security::scope_validator::ScopeValidator;
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
pub struct PathValidator {
    config: SecurityConfig,
    scope_validator: ScopeValidator,
}

impl PathValidator {
    pub fn new(config: SecurityConfig, allowed_paths: Vec<PathBuf>) -> Self {
        Self {
            config,
            scope_validator: ScopeValidator::new(allowed_paths),
        }
    }

    pub fn validate_import_path(&self, path: &Path) -> Result<PathBuf, SecurityError> {
        let path_str = path.to_string_lossy();

        if self.scope_validator.validate(path).is_err() {
            return Err(SecurityError::PathOutsideWorkspace {
                path: path.display().to_string(),
            });
        }

        if path_str.len() > self.config.max_path_length {
            return Err(SecurityError::PathTooLong {
                length: path_str.len(),
                max: self.config.max_path_length,
            });
        }

        if path_str.contains("..") {
            return Err(SecurityError::PathTraversal {
                path: path.display().to_string(),
            });
        }

        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if filename
            .chars()
            .any(|c| self.config.prohibited_filename_chars.contains(&c))
        {
            return Err(SecurityError::ProhibitedCharacters {
                filename: filename.to_string(),
            });
        }

        let ext_with_dot = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e.to_lowercase()))
            .unwrap_or_default();

        if !self.config.allowed_extensions.contains(&ext_with_dot) {
            return Err(SecurityError::InvalidExtension {
                extension: ext_with_dot,
            });
        }

        if !PermissionEscalation::can_escalate("import_path") {
            // Placeholder for future permission escalation
        }

        Ok(path.to_path_buf())
    }

    #[allow(dead_code)]
    pub fn permission_rationale() -> &'static str {
        PermissionsRationale::explain()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    fn default_config() -> (SecurityConfig, Vec<PathBuf>) {
        let config = SecurityConfig::default();
        let allowed_paths = vec![
            dirs::desktop_dir().unwrap(),
            dirs::document_dir().unwrap(),
            dirs::download_dir().unwrap(),
        ];
        (config, allowed_paths)
    }

    #[test]
    fn test_valid_path() {
        let (config, allowed_paths) = default_config();
        let validator = PathValidator::new(config, allowed_paths);
        let path = dirs::document_dir().unwrap().join("document.txt");
        assert!(validator.validate_import_path(&path).is_ok());
    }

    #[test]
    fn test_path_too_long() {
        let (config, allowed_paths) = default_config();
        let max_path_length = config.max_path_length;
        let validator = PathValidator::new(config, allowed_paths);

        let base_dir = dirs::document_dir().unwrap();
        let base_len = base_dir.to_string_lossy().len();
        let ext = ".txt";
        let ext_len = ext.len();

        let required_length = max_path_length + 1;
        let separator_len = 1;
        let filename_len = required_length.saturating_sub(base_len + separator_len + ext_len);

        let long_filename = format!("{}{}", "a".repeat(filename_len.max(1)), ext);
        let path = base_dir.join(long_filename);

        assert!(matches!(
            validator.validate_import_path(&path),
            Err(SecurityError::PathTooLong { .. })
        ));
    }

    #[test]
    fn test_prohibited_characters() {
        let (config, allowed_paths) = default_config();
        let validator = PathValidator::new(config, allowed_paths);
        let path = dirs::document_dir().unwrap().join("inva|id.txt");
        assert!(matches!(
            validator.validate_import_path(&path),
            Err(SecurityError::ProhibitedCharacters { .. })
        ));
    }

    #[test]
    fn test_invalid_extension() {
        let (config, allowed_paths) = default_config();
        let validator = PathValidator::new(config, allowed_paths);
        let path = dirs::document_dir().unwrap().join("file.exe");
        assert!(matches!(
            validator.validate_import_path(&path),
            Err(SecurityError::InvalidExtension { .. })
        ));
    }

    #[test]
    fn test_path_traversal() {
        let (config, allowed_paths) = default_config();
        let validator = PathValidator::new(config, allowed_paths);
        let path = dirs::document_dir().unwrap().join("../test.txt");
        assert!(matches!(
            validator.validate_import_path(&path),
            Err(SecurityError::PathTraversal { .. })
        ));
    }

    #[test]
    fn test_empty_extension() {
        let (config, allowed_paths) = default_config();
        let validator = PathValidator::new(config, allowed_paths);
        let path = dirs::document_dir().unwrap().join("file");
        assert!(matches!(
            validator.validate_import_path(&path),
            Err(SecurityError::InvalidExtension { .. })
        ));
    }

    #[test]
    fn test_mixed_case_extension() {
        let (config, allowed_paths) = default_config();
        let validator = PathValidator::new(config, allowed_paths);
        let path = dirs::document_dir().unwrap().join("Document.TXT");
        assert!(
            validator.validate_import_path(&path).is_ok(),
            "Mixed-case extensions should be valid"
        );
    }

    #[test]
    fn test_concurrent_access() {
        let (config, allowed_paths) = default_config();
        let validator = PathValidator::new(config, allowed_paths);
        let paths = vec![
            dirs::document_dir().unwrap().join("doc1.txt"),
            dirs::document_dir().unwrap().join("doc2.txt"),
            dirs::document_dir().unwrap().join("doc3.txt"),
        ];

        let handles: Vec<_> = paths
            .into_iter()
            .map(|p| {
                let validator = validator.clone();
                let path_str = p.to_string_lossy().to_string();
                thread::spawn(move || {
                    let result = validator.validate_import_path(&p);
                    (path_str, result)
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join()).collect();

        let mut errors = Vec::new();
        for result in results {
            match result {
                Ok((_path, Ok(_))) => {}
                Ok((path, Err(e))) => {
                    errors.push(format!("Validation failed for {}: {:?}", path, e));
                }
                Err(_) => {
                    errors.push("Thread panicked".to_string());
                }
            }
        }

        assert!(
            errors.is_empty(),
            "Found {} validation errors: {:?}",
            errors.len(),
            errors
        );
    }

    #[test]
    fn test_malicious_inputs() {
        let (config, allowed_paths) = default_config();
        let validator = PathValidator::new(config, allowed_paths);

        let malicious_inputs = [
            "../../../../etc/passwd",
            r"\\server\share\sensitive.txt",
            "/dev/null",
            "file:///etc/passwd",
            "./.git/config",
        ];

        for input in &malicious_inputs {
            let path = Path::new(input);
            assert!(
                validator.validate_import_path(path).is_err(),
                "Malicious input should be rejected: {}",
                input
            );
        }
    }
}
