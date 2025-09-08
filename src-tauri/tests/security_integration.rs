use std::path::Path;
use tempfile::TempDir;

// Bring in your library types
use proxemic::commands::{
    get_file_info_secure_with_validator, import_file_with_validator,
    validate_file_for_import_with_validator, FileInfo,
};
use proxemic::filesystem::errors::SecurityError;
use proxemic::filesystem::security::config::SecurityConfig;
use proxemic::filesystem::security::path_validator::PathValidator;
use tokio::runtime::Runtime;

#[test]
fn validate_scope_restriction() {
    let temp_dir = TempDir::new().unwrap();
    let allowed_path = temp_dir.path().to_path_buf();

    let config = SecurityConfig {
        allowed_workspace_paths: vec![allowed_path.clone()],
        ..Default::default()
    };
    let validator = PathValidator::new(config, vec![allowed_path.clone()]);

    let valid_path = allowed_path.join("document.pdf");
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
        ..Default::default()
    };
    let validator = PathValidator::new(config, vec![allowed_path.clone()]);

    let traversal_path = allowed_path.join("../secret.txt");

    assert!(
        validator.validate_import_path(&traversal_path).is_err(),
        "Path traversal attempt should be rejected"
    );
}

#[test]
fn validate_tauri_command_security() {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let allowed_path = temp_dir.path().to_path_buf();

    let config = SecurityConfig {
        allowed_workspace_paths: vec![allowed_path.clone()],
        ..Default::default()
    };
    let validator = PathValidator::new(config, vec![allowed_path.clone()]);

    let valid_path = allowed_path.join("safe.txt");
    std::fs::write(&valid_path, b"safe content").unwrap();

    rt.block_on(async {
        let result = validate_file_for_import_with_validator(&valid_path, &validator).await;
        assert!(result.is_ok(), "Expected valid path to be accepted");

        let result =
            validate_file_for_import_with_validator(Path::new("../../../etc/passwd"), &validator)
                .await;
        assert!(result.is_err(), "Expected malicious path to be rejected");
    });
}

#[test]
fn test_get_file_info_secure() {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let allowed_path = temp_dir.path().to_path_buf();

    let config = SecurityConfig {
        allowed_workspace_paths: vec![allowed_path.clone()],
        ..Default::default()
    };
    let validator = PathValidator::new(config, vec![allowed_path.clone()]);

    let valid_path = allowed_path.join("info.txt");
    std::fs::write(&valid_path, b"some content").unwrap();

    rt.block_on(async {
        let result: Result<FileInfo, SecurityError> =
            get_file_info_secure_with_validator(&valid_path, &validator).await;
        assert!(result.is_ok(), "Expected file info retrieval to succeed");

        let result =
            get_file_info_secure_with_validator(Path::new("../../../etc/passwd"), &validator).await;
        assert!(
            result.is_err(),
            "Expected malicious file info request to be rejected"
        );
    });
}

#[test]
fn test_import_file_command() {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let allowed_path = temp_dir.path().to_path_buf();

    let config = SecurityConfig {
        allowed_workspace_paths: vec![allowed_path.clone()],
        ..Default::default()
    };
    let validator = PathValidator::new(config, vec![allowed_path.clone()]);

    let valid_path = allowed_path.join("import.txt");
    std::fs::write(&valid_path, b"importable").unwrap();

    rt.block_on(async {
        let result = import_file_with_validator(&valid_path, &validator).await;
        assert!(result.is_ok(), "Expected valid import to succeed");

        let result = import_file_with_validator(Path::new("../../../etc/passwd"), &validator).await;
        assert!(result.is_err(), "Expected malicious import to fail");
    });
}
