// src-tauri/src/commands.rs

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::path_validator::PathValidator;
use std::path::PathBuf;

#[tauri::command]
pub async fn import_file(path: PathBuf) -> Result<PathBuf, SecurityError> {
    let validator = PathValidator::new(SecurityConfig::default());
    validator.validate_import_path(&path)
}
