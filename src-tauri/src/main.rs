// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod filesystem;

use crate::filesystem::security::config::SecurityConfig;
use crate::filesystem::security::path_validator::PathValidator;
use tauri::Manager;

use crate::filesystem::security::permissions_escalation::PermissionEscalation;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            validate_file_for_import,
            get_file_info_secure,
            request_permission_escalation
        ])
        .setup(|app| {
            // Initialize security configuration
            let config = SecurityConfig::default();

            // Set up global security validator
            let validator = std::sync::Arc::new(PathValidator::new(config, Vec::new()));
            app.manage(validator);

            // Initialize audit logging
            tracing_subscriber::fmt()
                .with_max_level(tracing::Level::INFO)
                .with_target(true)
                .init();

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Secure command to validate file paths for import
#[tauri::command]
async fn validate_file_for_import(
    path: String,
    validator: tauri::State<'_, std::sync::Arc<PathValidator>>,
) -> Result<String, String> {
    use std::path::Path;

    match validator.as_ref().validate_import_path(Path::new(&path)) {
        Ok(canonical_path) => Ok(canonical_path.to_string_lossy().to_string()),
        Err(e) => {
            tracing::warn!(
                path = %path,
                error = %e,
                "File path validation failed"
            );
            Err(e.to_string())
        }
    }
}

// New command to request permission escalation
#[tauri::command]
async fn request_permission_escalation(
    _window: tauri::Window,
    _message: String,
) -> Result<bool, String> {
    // Simulate user approval; in a real app, you could get this from frontend
    let user_approved = true;

    // Create a permission escalation object
    let escalation = PermissionEscalation::from_user_input(user_approved);

    Ok(escalation.user_approved)
}

// Secure command to get file information
#[tauri::command]
async fn get_file_info_secure(
    path: String,
    validator: tauri::State<'_, std::sync::Arc<PathValidator>>,
) -> Result<FileInfo, String> {
    use std::path::Path;

    // Validate path first
    let validated_path = validator
        .as_ref()
        .validate_import_path(Path::new(&path))
        .map_err(|e| {
            tracing::warn!(
                path = %path,
                error = %e,
                "File info access denied"
            );
            e.to_string()
        })?;

    // Get file metadata
    match std::fs::metadata(&validated_path) {
        Ok(metadata) => {
            tracing::info!(
                path = %validated_path.display(),
                size = metadata.len(),
                "File info retrieved successfully"
            );

            Ok(FileInfo {
                size: metadata.len(),
                modified: metadata.modified().ok(),
                is_file: metadata.is_file(),
                is_dir: metadata.is_dir(),
            })
        }
        Err(e) => {
            tracing::error!(
                path = %validated_path.display(),
                error = %e,
                "Failed to read file metadata"
            );
            Err(format!("Failed to read file metadata: {}", e))
        }
    }
}

#[derive(serde::Serialize)]
struct FileInfo {
    size: u64,
    modified: Option<std::time::SystemTime>,
    is_file: bool,
    is_dir: bool,
}
