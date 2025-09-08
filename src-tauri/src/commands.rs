// src-tauri/src/commands.rs

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::path_validator::PathValidator;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
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

#[tauri::command]
pub async fn validate_file_for_import(path: String) -> Result<String, CommandError> {
    let validator = PathValidator::new(SecurityConfig::default(), vec![]);

    let result = validator.validate_import_path(Path::new(&path));
    SecurityAuditor::log_file_access_attempt(Path::new(&path), "validate_import", &result);
    result
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn get_file_info_secure(path: String) -> Result<FileInfo, CommandError> {
    // Validate path first
    let validator = PathValidator::new(SecurityConfig::default(), vec![]);
    let validated_path = validator
        .validate_import_path(Path::new(&path))
        .map_err(CommandError::from)?;

    // Get file metadata
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
    let config = SecurityConfig::default();
    let allowed_paths = vec![
        dirs::desktop_dir().unwrap(),
        dirs::document_dir().unwrap(),
        dirs::download_dir().unwrap(),
    ];

    let validator = PathValidator::new(config, allowed_paths);
    validator
        .validate_import_path(&path)
        .map_err(CommandError::from)
}
