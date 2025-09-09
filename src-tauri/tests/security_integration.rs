// src-tauri/tests/security_integration.rs
// Updated integration tests with flexible collection types (Vec) to avoid
// mismatched fixed-size array type errors.

use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Bring in your library types
use proxemic::commands::{
    get_file_info_secure_with_validator, import_file_with_validator,
    validate_file_for_import_with_validator, FileInfo,
};
use proxemic::filesystem::errors::SecurityError;
use proxemic::filesystem::security::config::SecurityConfig;
use proxemic::filesystem::security::magic_number_validator::MagicNumberValidator;
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

    // Allow octet-stream as fallback MIME type
    config
        .allowed_mime_types
        .insert("application/octet-stream".to_string());

    // If SecurityConfig has a magic_number_map, ensure it's present. If not,
    // tests that need magic numbers will construct their own config.
    // (We avoid assuming a specific field layout beyond what's present in your config.)

    config
}

#[test]
fn test_mixed_case_extension() {
    let config = SecurityConfig {
        allowed_extensions: [".pdf".into()].into_iter().collect(),
        allowed_mime_types: ["application/pdf".into()].into_iter().collect(),
        // Provide a magic number map-like field only if your SecurityConfig actually contains it.
        // This test uses MagicNumberValidator::new(&config) which expects fields used there.
        ..Default::default()
    };
    let validator = MagicNumberValidator::new(&config);

    // Create test PDF file with valid magic number and extension
    let temp_file = tempfile::Builder::new().suffix(".PDF").tempfile().unwrap();
    let path = temp_file.path();
    let mut file = std::fs::File::create(path).unwrap();
    // Write minimal valid PDF header
    file.write_all(b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n").unwrap(); // Valid PDF header

    // Test with lowercase ext argument to validator (the validator should be case-insensitive)
    assert!(validator.validate_file_type(path, "pdf").is_ok());

    // Test with mixed-case extension by renaming
    let mixed_case_path = path.with_extension("PdF");
    std::fs::rename(path, &mixed_case_path).unwrap();
    assert!(validator
        .validate_file_type(&mixed_case_path, "pdf")
        .is_ok());
}

#[test]
fn test_invalid_extensions() {
    let config = SecurityConfig::default();
    let validator = PathValidator::new(config, vec![std::env::temp_dir()]);

    type ErrorPredicate = fn(&SecurityError) -> bool;
    let test_cases: Vec<(&str, &[u8], ErrorPredicate)> = vec![
        ("virus.exe", b"malicious", |e| {
            matches!(e, SecurityError::InvalidExtension { .. })
        }),
        ("document.txt.exe", b"hidden", |e| {
            matches!(e, SecurityError::InvalidExtension { .. })
        }),
        ("IMAGE.JPEG.EXE", b"jpgEXE", |e| {
            matches!(e, SecurityError::InvalidExtension { .. })
        }),
        ("report.tar.gz.bad", b"badarchive", |e| {
            matches!(e, SecurityError::InvalidExtension { .. })
        }),
        ("double.extension.txt.doc", b"confusing", |e| {
            matches!(e, SecurityError::InvalidExtension { .. })
        }),
        ("file.EXE", b"executable", |e| {
            matches!(e, SecurityError::InvalidExtension { .. })
        }),
        ("script.bat.", b"batch", |e| {
            matches!(e, SecurityError::ProhibitedCharacters { .. })
        }), // Trailing dot
        ("config.bat..", b"double_dot", |e| {
            matches!(e, SecurityError::ProhibitedCharacters { .. })
        }), // Double trailing dots
        ("file..exe", b"multi_dot", |e| {
            matches!(e, SecurityError::InvalidExtension { .. })
        }),
        ("backup.tar.gz.exe", b"disguised", |e| {
            matches!(e, SecurityError::InvalidExtension { .. })
        }),
    ];

    for (case, content, expected_error) in test_cases {
        let path = std::env::temp_dir().join(case);
        std::fs::write(&path, content).unwrap();
        let result = validator.validate_import_path(&path);
        assert!(
            result.is_err(),
            "Invalid extension '{}' should be rejected",
            case
        );

        match result {
            Err(error) => {
                assert!(
                    expected_error(&error),
                    "Expected different error type for '{}'. Got: {:?}",
                    case,
                    error
                );
            }
            Ok(_) => panic!("Expected error for '{}'", case),
        }
    }
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
    let invalid_path = if cfg!(target_os = "windows") {
        Path::new("C:/Windows/system32/calc.exe")
    } else {
        Path::new("/usr/bin/passwd")
    };

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

    // Create a benign file and then try traversal relative to it
    let target = allowed_path.join("secret.txt");
    std::fs::write(&target, b"secret").unwrap();

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

    // Create config with MIME type restrictions
    let mut config = create_test_security_config();
    config.allowed_mime_types.insert("text/plain".to_string());
    config.max_file_size = 1024; // 1KB for testing

    // Use our helper to create a properly configured validator
    let validator = create_validator_for_tempdir(allowed_path.clone());

    let valid_path = allowed_path.join("safe.txt");
    std::fs::write(&valid_path, b"safe content").unwrap();

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
        let reserved = vec![
            "CON", "PRN", "NUL", "AUX", "COM1", "COM2", "LPT1", "LPT2", "CLOCK$",
        ];
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
        let hidden = vec![".git", ".env", ".ssh"];
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

    // Use our helper to create a properly configured validator
    let validator = create_validator_for_tempdir(allowed_path.clone());

    let valid_path = allowed_path.join("info.txt");
    std::fs::write(&valid_path, b"some content").unwrap();

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

    // Use our helper to create a properly configured validator
    let validator = create_validator_for_tempdir(allowed_path.clone());

    let valid_path = allowed_path.join("import.txt");
    std::fs::write(&valid_path, b"importable").unwrap();

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

    // Create a dummy file
    File::create(&file_path).expect("Failed to create file");

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
    std::fs::write(&valid, "dummy").unwrap();

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
