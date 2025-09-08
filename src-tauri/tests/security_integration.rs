use std::path::{Path, PathBuf};
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

// ==================== HELPER FUNCTIONS ====================

/// Creates a test-friendly SecurityConfig with common extensions
fn create_test_security_config() -> SecurityConfig {
    let mut config = SecurityConfig::default();

    // Ensure common test extensions are allowed
    config.allowed_extensions.insert(".txt".to_string());
    config.allowed_extensions.insert(".docx".to_string());
    config.allowed_extensions.insert(".pdf".to_string());

    config
}

/// Creates a PathValidator for a specific temporary directory
fn create_validator_for_tempdir(temp_path: PathBuf) -> PathValidator {
    let config = create_test_security_config();
    let allowed_paths = vec![temp_path, std::env::temp_dir()];
    PathValidator::new(config, allowed_paths)
}

// ==================== TEST FUNCTIONS ====================

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

    println!("DEBUG: temp_dir path: {:?}", allowed_path);

    // Use our helper to create a properly configured validator
    let validator = create_validator_for_tempdir(allowed_path.clone());

    let valid_path = allowed_path.join("safe.txt");
    std::fs::write(&valid_path, b"safe content").unwrap();

    println!("DEBUG: valid_path: {:?}", valid_path);
    println!("DEBUG: valid_path exists: {}", valid_path.exists());
    println!("DEBUG: valid_path is_file: {}", valid_path.is_file());

    rt.block_on(async {
        let result = validate_file_for_import_with_validator(&valid_path, &validator).await;
        if let Err(ref e) = result {
            println!("DEBUG: Validation error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Expected valid path to be accepted. Error: {:?}",
            result.err()
        );

        let result =
            validate_file_for_import_with_validator(Path::new("../../../etc/passwd"), &validator)
                .await;
        assert!(result.is_err(), "Expected malicious path to be rejected");
    });
}

#[test]
fn validate_reserved_filenames() {
    let config = SecurityConfig::default();
    let allowed = vec![std::env::temp_dir()];
    let validator = PathValidator::new(config, allowed);

    // Windows reserved names (only run on Windows)
    #[cfg(target_os = "windows")]
    {
        let reserved = ["CON", "PRN", "NUL", "AUX", "COM1", "LPT1"];
        for name in reserved {
            let path = std::env::temp_dir().join(name);
            let result = validator.validate_import_path(&path);
            assert!(
                result.is_err(),
                "Reserved name `{}` should be rejected",
                name
            );
        }
    }

    // Unix-like hidden files
    #[cfg(unix)]
    {
        let hidden = [".git", ".env", ".ssh"];
        for name in hidden {
            let path = std::env::temp_dir().join(name);
            let result = validator.validate_import_path(&path);
            assert!(result.is_err(), "Hidden file `{}` should be rejected", name);
        }
    }
}

#[test]
fn test_get_file_info_secure() {
    let rt = Runtime::new().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let allowed_path = temp_dir.path().to_path_buf();

    println!("DEBUG: temp_dir path: {:?}", allowed_path);

    // Use our helper to create a properly configured validator
    let validator = create_validator_for_tempdir(allowed_path.clone());

    let valid_path = allowed_path.join("info.txt");
    std::fs::write(&valid_path, b"some content").unwrap();

    println!("DEBUG: valid_path: {:?}", valid_path);
    println!("DEBUG: valid_path exists: {}", valid_path.exists());

    rt.block_on(async {
        let result: Result<FileInfo, SecurityError> =
            get_file_info_secure_with_validator(&valid_path, &validator).await;
        if let Err(ref e) = result {
            println!("DEBUG: get_file_info error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Expected file info retrieval to succeed. Error: {:?}",
            result.err()
        );

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

    println!("DEBUG: temp_dir path: {:?}", allowed_path);

    // Use our helper to create a properly configured validator
    let validator = create_validator_for_tempdir(allowed_path.clone());

    let valid_path = allowed_path.join("import.txt");
    std::fs::write(&valid_path, b"importable").unwrap();

    println!("DEBUG: valid_path: {:?}", valid_path);
    println!("DEBUG: valid_path exists: {}", valid_path.exists());

    rt.block_on(async {
        let result = import_file_with_validator(&valid_path, &validator).await;
        if let Err(ref e) = result {
            println!("DEBUG: import error: {:?}", e);
        }
        assert!(
            result.is_ok(),
            "Expected valid import to succeed. Error: {:?}",
            result.err()
        );

        let result = import_file_with_validator(Path::new("../../../etc/passwd"), &validator).await;
        assert!(result.is_err(), "Expected malicious import to fail");
    });
}

#[test]
fn validate_tempdir_workflow() {
    use std::fs::File;
    use tempfile::tempdir;

    let dir = tempdir().expect("TempDir creation failed");
    let file_path = dir.path().join("workflow_test.txt");

    println!("DEBUG: tempdir path: {:?}", dir.path());
    println!("DEBUG: file_path: {:?}", file_path);

    // Create a dummy file
    File::create(&file_path).expect("Failed to create file");
    println!("DEBUG: file created, exists: {}", file_path.exists());

    // Create validator that includes the temp directory in allowed paths
    let validator = create_validator_for_tempdir(dir.path().to_path_buf());

    // Validate
    let result = validator.validate_import_path(&file_path);
    if let Err(ref e) = result {
        println!("DEBUG: validation error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "File inside TempDir should be valid. Error: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn validate_async_command_security() {
    use proxemic::filesystem::operations::validate_file_for_import;

    // Bad path traversal
    let bad = "../../etc/passwd";
    let result = validate_file_for_import(bad);
    assert!(
        result.is_err(),
        "Path traversal should be blocked in async command"
    );

    // Valid path - create file in temp directory
    let valid = std::env::temp_dir().join("safe_file.txt");
    println!("DEBUG: temp dir path: {:?}", std::env::temp_dir());
    println!("DEBUG: valid file path: {:?}", valid);

    std::fs::write(&valid, "dummy").unwrap();
    println!("DEBUG: file created, exists: {}", valid.exists());

    // Since operations.rs already includes temp_dir in allowed_paths, this should work
    let result = validate_file_for_import(valid.to_str().unwrap());
    if let Err(ref e) = result {
        println!("DEBUG: validation error: {:?}", e);
    }
    assert!(
        result.is_ok(),
        "Valid file in temp dir should be allowed. Error: {:?}",
        result.err()
    );
}
