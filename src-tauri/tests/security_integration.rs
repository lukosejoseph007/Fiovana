use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Bring in your library types
use proxemic::filesystem::security::config::SecurityConfig;
use proxemic::filesystem::security::path_validator::PathValidator;

#[test]
fn validate_scope_restriction() {
    // Setup temporary workspace directory
    let temp_dir = TempDir::new().unwrap();
    let allowed_path = temp_dir.path().to_path_buf();

    // Configure security with scope restrictions
    let config = SecurityConfig {
        allowed_extensions: vec![".txt".to_string(), ".md".to_string(), ".pdf".to_string()]
            .into_iter()
            .collect(),
        prohibited_filename_chars: vec!['|', '<', '>', ':', '*', '?', '\\', '/']
            .into_iter()
            .collect(),
        max_path_length: 260,
        max_file_size: 1024 * 1024,
        allowed_workspace_paths: vec![allowed_path.clone()],
        temp_directory: PathBuf::from(""),
        enable_magic_number_validation: false,
    };

    let validator = PathValidator::new(config, vec![allowed_path.clone()]);

    // Valid path within allowed scope
    let valid_path = allowed_path.join("document.pdf");

    // Invalid path outside allowed scope
    let invalid_path = Path::new("C:/Windows/system32/calc.exe");

    assert!(
        validator.validate_import_path(&valid_path).is_ok(),
        "Path within allowed scope should be accepted"
    );
    assert!(
        validator.validate_import_path(invalid_path).is_err(),
        "Path outside allowed scope should be rejected"
    );
}

#[test]
fn validate_path_traversal_attempt() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_path = temp_dir.path().to_path_buf();

    let config = SecurityConfig {
        allowed_workspace_paths: vec![allowed_path.clone()],
        // Fill in defaults for the rest
        ..SecurityConfig::default()
    };

    let validator = PathValidator::new(config, vec![allowed_path.clone()]);

    // Attempt path traversal outside allowed scope
    let traversal_path = allowed_path.join("../secret.txt");

    assert!(
        validator.validate_import_path(&traversal_path).is_err(),
        "Path traversal attempt should be rejected"
    );
}
