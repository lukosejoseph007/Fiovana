// src-tauri/src/commands.rs

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::path_validator::PathValidator;

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, serde::Deserialize)]
pub enum CommandError {
    SecurityError(SecurityError),
    IoError(String),
    Custom(String),
}

impl From<SecurityError> for CommandError {
    fn from(err: SecurityError) -> CommandError {
        CommandError::SecurityError(err)
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> CommandError {
        CommandError::IoError(err.to_string())
    }
}

impl From<String> for CommandError {
    fn from(err: String) -> CommandError {
        CommandError::Custom(err)
    }
}

#[derive(Serialize)]
pub struct FileInfo {
    size: u64,
    modified: Option<std::time::SystemTime>,
    is_file: bool,
    is_dir: bool,
}

/// Helper function to create a properly configured validator for production use
fn create_default_validator() -> PathValidator {
    let mut config = SecurityConfig::default();
    // Ensure common extensions are allowed
    config.allowed_extensions.insert(".txt".to_string());

    let mut allowed_paths = vec![
        dirs::desktop_dir().unwrap_or_default(),
        dirs::document_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
    ];

    // Also allow temp directory for temporary operations
    allowed_paths.push(std::env::temp_dir());

    PathValidator::new(config, allowed_paths)
}

// ---------------- Standard Tauri Commands ----------------

#[tauri::command]
pub async fn validate_file_for_import(path: String) -> Result<String, CommandError> {
    let validator = create_default_validator();
    let result = validator.validate_import_path(Path::new(&path));
    SecurityAuditor::log_file_access_attempt(Path::new(&path), "validate_import", &result);
    result
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn get_file_info_secure(path: String) -> Result<FileInfo, CommandError> {
    let validator = create_default_validator();
    let validated_path = validator
        .validate_import_path(Path::new(&path))
        .map_err(CommandError::from)?;
    let metadata = fs::metadata(&validated_path).map_err(CommandError::from)?;
    Ok(FileInfo {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
    })
}

#[tauri::command]
pub async fn import_file(path: PathBuf) -> Result<PathBuf, CommandError> {
    let validator = create_default_validator();
    validator
        .validate_import_path(&path)
        .map_err(CommandError::from)
}

// ---------------- Testable Variants with Custom Validator ----------------

#[allow(dead_code)]
pub async fn validate_file_for_import_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<PathBuf, SecurityError> {
    let result = validator.validate_import_path(path);
    SecurityAuditor::log_file_access_attempt(path, "validate_import", &result);
    result
}

#[allow(dead_code)]
pub async fn get_file_info_secure_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<FileInfo, SecurityError> {
    let validated_path = validator.validate_import_path(path)?;
    let metadata =
        fs::metadata(&validated_path).map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        })?;
    Ok(FileInfo {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
    })
}

#[allow(dead_code)]
pub async fn import_file_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<PathBuf, SecurityError> {
    validator.validate_import_path(path)
}
