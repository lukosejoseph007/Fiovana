// src-tauri/src/main.rs
// Prevent console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::Manager;
use tracing::info;

mod ai;
mod app_config;
mod commands;
mod db;
mod document;
mod filesystem;
mod vector;

use app_config::ConfigManager;

// Application state to hold the configuration manager
pub struct AppState {
    pub config_manager: Arc<ConfigManager>,
}

#[tokio::main]
async fn main() {
    ai::init();
    // Initialize configuration system first
    let config_manager = match app_config::init().await {
        Ok(manager) => {
            info!("Configuration system initialized successfully");
            Arc::new(manager)
        }
        Err(e) => {
            eprintln!("Failed to initialize configuration: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize logging based on configuration
    if let Err(e) = initialize_logging(&config_manager).await {
        eprintln!("Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("Starting Proxemic application...");
    info!("Environment: {:?}", config_manager.environment());

    // Create application state
    let app_state = AppState {
        config_manager: Arc::clone(&config_manager),
    };

    // Build and run the Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::greet,
            commands::validate_file_for_import,
            commands::import_file,
            commands::get_file_info_secure,
        ])
        .setup(|app| {
            info!("Application setup complete");

            // Log configuration status
            let state = app.state::<AppState>();
            if let Some(config_path) = state.config_manager.config_file_path() {
                info!("Using configuration file: {}", config_path.display());
            } else {
                info!("Using default configuration");
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Initialize logging based on configuration
async fn initialize_logging(
    config_manager: &ConfigManager,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_guard = config_manager.get_config();
    let config = config_guard
        .read()
        .map_err(|_| "Failed to read configuration")?;

    // Initialize tracing subscriber based on configuration
    let log_level = match config.logging.level.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };

    let subscriber_builder = tracing_subscriber::FmtSubscriber::builder().with_max_level(log_level);

    // Configure output based on settings
    if config.logging.structured_logging {
        // Use JSON formatting for structured logging
        let subscriber = subscriber_builder.json().finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        // Use pretty formatting for development
        let subscriber = subscriber_builder.pretty().finish();
        tracing::subscriber::set_global_default(subscriber)?;
    }

    // TODO: Add file logging support when file_enabled is true
    if config.logging.file_enabled {
        if let Some(ref _file_path) = config.logging.file_path {
            // File logging implementation would go here
            // This requires additional dependencies like tracing-appender
            info!("File logging is configured but not yet implemented");
        }
    }

    info!("Logging initialized with level: {}", config.logging.level);
    Ok(())
}
