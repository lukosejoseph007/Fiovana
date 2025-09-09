// src-tauri/src/filesystem/operations.rs
// Enhanced operations with production-safe security integration

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use std::path::Path;

// Legacy compatibility function - uses PathValidator directly
#[allow(dead_code)]
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    let validator = PathValidator::new(
        SecurityConfig::default(),
        vec![
            dirs::desktop_dir().unwrap_or_default(),
            dirs::document_dir().unwrap_or_default(),
            dirs::download_dir().unwrap_or_default(),
            std::env::temp_dir(),
        ],
    );

    validator
        .validate_import_path(Path::new(path))
        .map(|p| p.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_legacy_validate_function() {
        // Create a temporary file for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test that the legacy function works
        let result = validate_file_for_import(&test_file.to_string_lossy());
        assert!(result.is_ok());
    }

    #[test]
    fn test_production_security_level() {
        std::env::set_var("PROXEMIC_SECURITY_LEVEL", "production");

        // Test that environment variable affects default config
        let config = SecurityConfig::default();
        assert!(config.enable_magic_number_validation);
        assert!(config.enforce_workspace_boundaries);

        std::env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }
}
