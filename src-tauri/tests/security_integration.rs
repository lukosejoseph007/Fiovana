use proxemic::filesystem::security::config::SecurityConfig;
use proxemic::filesystem::security::path_validator::PathValidator;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn validate_import_rejects_exe() {
    let config = SecurityConfig {
        allowed_extensions: vec![".txt".to_string(), ".md".to_string(), ".pdf".to_string()]
            .into_iter()
            .collect::<HashSet<String>>(),
        prohibited_filename_chars: vec!['|', '<', '>', ':', '*', '?', '\\', '/']
            .into_iter()
            .collect::<HashSet<char>>(),
        max_path_length: 260,
        max_file_size: 1024 * 1024,
        allowed_workspace_paths: vec![],
        temp_directory: PathBuf::from(""),
        enable_magic_number_validation: false,
    };

    let temp_dir = TempDir::new().unwrap();
    let validator = PathValidator::new(config, vec![temp_dir.path().to_path_buf()]);
    let path = Path::new("malicious.exe");

    assert!(validator.validate_import_path(path).is_err());
}

#[test]
fn validate_import_accepts_pdf() {
    let config = SecurityConfig {
        allowed_extensions: vec![".txt".to_string(), ".md".to_string(), ".pdf".to_string()]
            .into_iter()
            .collect::<HashSet<String>>(),
        prohibited_filename_chars: vec!['|', '<', '>', ':', '*', '?', '\\', '/']
            .into_iter()
            .collect::<HashSet<char>>(),
        max_path_length: 260,
        max_file_size: 1024 * 1024,
        allowed_workspace_paths: vec![],
        temp_directory: PathBuf::from(""),
        enable_magic_number_validation: false,
    };

    let temp_dir = TempDir::new().unwrap();
    let validator = PathValidator::new(config, vec![temp_dir.path().to_path_buf()]);
    let path = temp_dir.path().join("document.pdf");

    assert!(
        validator.validate_import_path(&path).is_ok(),
        "PDF should be accepted by Tauri command layer. Error: {:?}",
        validator.validate_import_path(&path).err()
    );
}
