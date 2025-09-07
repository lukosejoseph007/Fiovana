// src-tauri/src/commands.rs

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::path_validator::PathValidator;
use std::path::PathBuf;

#[tauri::command]
pub async fn import_file(path: PathBuf) -> Result<PathBuf, SecurityError> {
    let config = SecurityConfig::default();
    let allowed_paths = vec![
        dirs::desktop_dir().unwrap(),
        dirs::document_dir().unwrap(),
        dirs::download_dir().unwrap(),
    ];

    let validator = PathValidator::new(config, allowed_paths);
    validator.validate_import_path(&path)
}
