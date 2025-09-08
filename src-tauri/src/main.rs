// src-tauri/src/main.rs

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use tauri::Manager;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

mod commands;
mod filesystem;

use commands::{get_file_info_secure, import_file, validate_file_for_import};
use filesystem::{init_security_subsystem, PathValidator, SecurityConfig};

fn main() {
    // Initialize audit logging first
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(true)
        .init();

    tauri::Builder::default()
        // Tauri logging plugin
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .target(Target::new(TargetKind::LogDir { file_name: None }))
                .target(Target::new(TargetKind::Stdout))
                .target(Target::new(TargetKind::Webview))
                .rotation_strategy(RotationStrategy::KeepSome(10))
                .build(),
        )
        // Register Tauri commands
        .invoke_handler(tauri::generate_handler![
            validate_file_for_import,
            get_file_info_secure,
            import_file
        ])
        // Setup security subsystem and app state
        .setup(|app| {
            // Load security configuration (use default or app-specific)
            let config = SecurityConfig::default();

            // Initialize the filesystem security subsystem
            let validator: PathValidator =
                init_security_subsystem(config).expect("Failed to initialize security subsystem");

            // Manage validator globally using Arc
            app.manage(Arc::new(validator));

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
