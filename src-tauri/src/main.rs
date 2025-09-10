// src-tauri/src/main.rs
// Prevent console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::Manager;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ai;
mod app_config;
mod commands;
mod db;
mod document;
mod filesystem;
mod vector;

use app_config::ConfigManager;
use filesystem::security::{
    audit_logger, initialize_security_system, SecurityLevel, StartupValidationResult,
};

// Enhanced application state to hold both configuration and security information
pub struct AppState {
    pub config_manager: Arc<ConfigManager>,
    pub security_state: SecurityState,
}

// Security state to store security information
#[derive(Clone)]
pub struct SecurityState {
    pub validation_result: StartupValidationResult,
}

#[tokio::main]
async fn main() {
    // Initialize AI system first
    ai::init();

    // Initialize early logging for security system
    init_early_logging();

    // CRITICAL: Initialize security system before anything else
    let security_result = match initialize_security_system() {
        Ok(result) => {
            if !result.success {
                eprintln!("CRITICAL: Security system initialization failed!");
                eprintln!("Application startup blocked for security reasons.");

                for error in &result.errors {
                    eprintln!("Security Error: {}", error);
                }

                // Exit immediately on security failure
                std::process::exit(1);
            }
            result
        }
        Err(e) => {
            eprintln!("FATAL: Failed to initialize security system: {}", e);
            std::process::exit(1);
        }
    };

    // Log security status
    info!("Security system initialized successfully");
    info!("Security Level: {:?}", security_result.security_level);
    info!("Production Ready: {}", security_result.production_ready);

    // Log any warnings
    if !security_result.warnings.is_empty() {
        tracing::warn!(
            "Security warnings detected ({} total):",
            security_result.warnings.len()
        );
        for warning in &security_result.warnings {
            tracing::warn!("  - {}", warning);
        }
    }

    // Production-specific startup behavior
    if matches!(
        security_result.security_level,
        SecurityLevel::Production | SecurityLevel::HighSecurity
    ) {
        if !security_result.production_ready {
            eprintln!("ERROR: Application not ready for production deployment!");
            eprintln!("Run deployment readiness check and fix all issues before proceeding.");
            std::process::exit(1);
        }
        info!("Production mode verified - starting application");
    }

    // Initialize configuration system after security validation
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

    // Initialize comprehensive logging based on configuration and security level
    if let Err(e) = initialize_comprehensive_logging(&config_manager, &security_result).await {
        eprintln!("Failed to initialize comprehensive logging: {}", e);
        std::process::exit(1);
    }

    info!("Starting Proxemic application...");
    info!("Environment: {:?}", config_manager.environment());

    // Create enhanced application state with both config and security
    let app_state = AppState {
        config_manager: Arc::clone(&config_manager),
        security_state: SecurityState {
            validation_result: security_result.clone(),
        },
    };

    // Build and run the Tauri application
    tauri::Builder::default()
        .manage(app_state)
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // Existing commands
            commands::greet,
            commands::validate_file_for_import,
            commands::import_file,
            commands::get_file_info_secure,
            // Security status commands
            get_security_status,
            get_deployment_report,
        ])
        .setup(move |app| {
            info!("Application setup complete");

            // Log configuration status
            let state = app.state::<AppState>();
            if let Some(config_path) = state.config_manager.config_file_path() {
                info!("Using configuration file: {}", config_path.display());
            } else {
                info!("Using default configuration");
            }

            // Additional security monitoring setup
            setup_security_monitoring(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Initialize early logging for security system startup
fn init_early_logging() {
    // Configure basic logging for security initialization
    let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        // Default log levels based on security level
        match std::env::var("PROXEMIC_SECURITY_LEVEL")
            .unwrap_or_default()
            .as_str()
        {
            "production" | "high_security" => "warn".to_string(),
            _ => "info".to_string(),
        }
    });

    std::env::set_var("RUST_LOG", &log_level);

    let subscriber = tracing_subscriber::registry().with(
        tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level)),
    );

    // Use structured logging in production
    if std::env::var("PROXEMIC_STRUCTURED_LOGGING").unwrap_or_default() == "true" {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        subscriber.with(tracing_subscriber::fmt::layer()).init();
    }

    info!("Early logging initialized with level: {}", log_level);
}

/// Initialize comprehensive logging based on both configuration and security requirements
async fn initialize_comprehensive_logging(
    config_manager: &ConfigManager,
    security_result: &StartupValidationResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_guard = config_manager.get_config();
    let config = config_guard
        .read()
        .map_err(|_| "Failed to read configuration")?;

    // Determine log level from config, but respect security requirements
    let mut log_level = match config.logging.level.to_lowercase().as_str() {
        "error" => tracing::Level::ERROR,
        "warn" => tracing::Level::WARN,
        "info" => tracing::Level::INFO,
        "debug" => tracing::Level::DEBUG,
        "trace" => tracing::Level::TRACE,
        _ => tracing::Level::INFO,
    };

    // Override log level for production security
    if matches!(
        security_result.security_level,
        SecurityLevel::Production | SecurityLevel::HighSecurity
    ) {
        // In production, enforce minimum WARN level unless explicitly configured otherwise
        if matches!(log_level, tracing::Level::DEBUG | tracing::Level::TRACE) {
            log_level = tracing::Level::WARN;
            tracing::warn!("Log level adjusted to WARN for production security");
        }
    }

    let subscriber_builder = tracing_subscriber::FmtSubscriber::builder().with_max_level(log_level);

    // Configure output based on settings and security level
    let use_structured = config.logging.structured_logging
        || matches!(
            security_result.security_level,
            SecurityLevel::Production | SecurityLevel::HighSecurity
        );

    if use_structured {
        // Use JSON formatting for structured logging
        let subscriber = subscriber_builder.json().finish();
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        // Use pretty formatting for development
        let subscriber = subscriber_builder.pretty().finish();
        tracing::subscriber::set_global_default(subscriber)?;
    }

    // File logging with security considerations
    if config.logging.file_enabled {
        if let Some(ref _file_path) = config.logging.file_path {
            // File logging implementation would go here
            // This requires additional dependencies like tracing-appender
            info!("File logging is configured but not yet implemented");

            // In production, file logging should include:
            // - Log rotation
            // - Secure file permissions
            // - Integrity verification
            // - Encryption for sensitive logs
        }
    }

    info!(
        "Comprehensive logging initialized with level: {}",
        config.logging.level
    );
    Ok(())
}

/// Set up security monitoring based on configuration and security level
fn setup_security_monitoring(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let state = app.state::<AppState>();
    let security_level = &state.security_state.validation_result.security_level;

    // Set up security monitoring based on security level
    let audit_enabled = std::env::var("PROXEMIC_AUDIT_LOGGING_ENABLED")
        .unwrap_or_default()
        .to_lowercase()
        == "true"
        || matches!(
            security_level,
            SecurityLevel::Production | SecurityLevel::HighSecurity
        );

    if audit_enabled {
        info!("Security audit logging enabled");

        // Initialize audit log rotation system
        if let Err(e) = audit_logger::SecurityAuditor::init_log_rotation(None) {
            tracing::error!("Failed to initialize audit log rotation: {}", e);
        } else {
            info!("Audit log rotation system initialized successfully");
        }

        // In a production system, you might set up:
        // - Security event aggregation
        // - Alert thresholds
        // - Monitoring dashboards
        // - Compliance logging
    }

    let performance_monitoring = std::env::var("PROXEMIC_PERFORMANCE_MONITORING")
        .unwrap_or_default()
        .to_lowercase()
        == "true"
        || matches!(security_level, SecurityLevel::Production);

    if performance_monitoring {
        info!("Performance monitoring enabled");
        // Set up performance monitoring
        // - Resource usage tracking
        // - Response time monitoring
        // - Memory leak detection
    }

    // Enhanced monitoring for high security environments
    if matches!(security_level, SecurityLevel::HighSecurity) {
        info!("High security monitoring enabled");
        // Additional security monitoring:
        // - File access monitoring
        // - Network activity logging
        // - Permission escalation detection
        // - Anomaly detection
    }

    Ok(())
}

// Security status command for frontend
#[tauri::command]
async fn get_security_status(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let result = &state.security_state.validation_result;

    Ok(serde_json::json!({
        "security_level": result.security_level,
        "production_ready": result.production_ready,
        "configuration_valid": result.config_valid,
        "environment_ready": result.environment_ready,
        "warnings_count": result.warnings.len(),
        "has_warnings": !result.warnings.is_empty(),
        "security_features_enabled": true,
        "config_management_active": true
    }))
}

// Deployment report command for admin interface
#[tauri::command]
async fn get_deployment_report(state: tauri::State<'_, AppState>) -> Result<String, String> {
    use filesystem::security::DeploymentChecker;

    let checker = DeploymentChecker::new();
    let mut report = checker
        .generate_deployment_report()
        .map_err(|e| e.to_string())?;

    // Enhance report with configuration information
    let config_guard = state.config_manager.get_config();
    if let Ok(config) = config_guard.read() {
        report.push_str("\n\n=== Configuration Status ===\n");
        report.push_str(&format!(
            "Environment: {:?}\n",
            state.config_manager.environment()
        ));
        report.push_str(&format!("Log Level: {}\n", config.logging.level));
        report.push_str(&format!(
            "Structured Logging: {}\n",
            config.logging.structured_logging
        ));
        report.push_str(&format!("File Logging: {}\n", config.logging.file_enabled));

        if let Some(config_path) = state.config_manager.config_file_path() {
            report.push_str(&format!("Config File: {}\n", config_path.display()));
        } else {
            report.push_str("Config File: Using defaults\n");
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_security_initialization() {
        // Set minimal valid environment for testing
        env::set_var("PROXEMIC_SECURITY_LEVEL", "development");

        let result = initialize_security_system();
        assert!(result.is_ok());

        let security_result = result.unwrap();
        assert!(security_result.success);

        env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }

    #[test]
    fn test_production_readiness_check() {
        // Test production configuration validation
        env::set_var("PROXEMIC_SECURITY_LEVEL", "production");
        env::set_var("PROXEMIC_ENABLE_MAGIC_VALIDATION", "true");
        env::set_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES", "true");
        env::set_var("PROXEMIC_AUDIT_LOGGING_ENABLED", "true");
        env::set_var(
            "PROXEMIC_ENCRYPTION_KEY",
            "test_key_32_characters_long_unique",
        );

        let result = initialize_security_system();
        assert!(result.is_ok());

        let security_result = result.unwrap();
        assert!(security_result.success);
        assert_eq!(security_result.security_level, SecurityLevel::Production);

        // Clean up
        env::remove_var("PROXEMIC_SECURITY_LEVEL");
        env::remove_var("PROXEMIC_ENABLE_MAGIC_VALIDATION");
        env::remove_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES");
        env::remove_var("PROXEMIC_AUDIT_LOGGING_ENABLED");
        env::remove_var("PROXEMIC_ENCRYPTION_KEY");
    }

    #[test]
    fn test_insecure_production_config_blocks_startup() {
        // Save original environment variables
        let original_security_level = env::var("PROXEMIC_SECURITY_LEVEL").ok();
        let original_encryption_key = env::var("PROXEMIC_ENCRYPTION_KEY").ok();
        let original_magic_validation = env::var("PROXEMIC_ENABLE_MAGIC_VALIDATION").ok();
        let original_workspace_boundaries = env::var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES").ok();
        let original_audit_logging = env::var("PROXEMIC_AUDIT_LOGGING_ENABLED").ok();

        // Test that insecure production config prevents startup
        env::set_var("PROXEMIC_SECURITY_LEVEL", "production");
        env::set_var(
            "PROXEMIC_ENCRYPTION_KEY",
            "your_secure_32_character_key_here_change_this",
        ); // Default key
        env::set_var("PROXEMIC_ENABLE_MAGIC_VALIDATION", "true");
        env::set_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES", "true");
        env::set_var("PROXEMIC_AUDIT_LOGGING_ENABLED", "true");

        let result = initialize_security_system();

        // Should fail due to default encryption key
        if let Ok(security_result) = result {
            assert!(!security_result.success);
            assert!(security_result
                .errors
                .iter()
                .any(|e| e.contains("Default encryption key")));
        }

        // Restore original environment variables
        if let Some(val) = original_security_level {
            env::set_var("PROXEMIC_SECURITY_LEVEL", val);
        } else {
            env::remove_var("PROXEMIC_SECURITY_LEVEL");
        }

        if let Some(val) = original_encryption_key {
            env::set_var("PROXEMIC_ENCRYPTION_KEY", val);
        } else {
            env::remove_var("PROXEMIC_ENCRYPTION_KEY");
        }

        if let Some(val) = original_magic_validation {
            env::set_var("PROXEMIC_ENABLE_MAGIC_VALIDATION", val);
        } else {
            env::remove_var("PROXEMIC_ENABLE_MAGIC_VALIDATION");
        }

        if let Some(val) = original_workspace_boundaries {
            env::set_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES", val);
        } else {
            env::remove_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES");
        }

        if let Some(val) = original_audit_logging {
            env::set_var("PROXEMIC_AUDIT_LOGGING_ENABLED", val);
        } else {
            env::remove_var("PROXEMIC_AUDIT_LOGGING_ENABLED");
        }
    }

    #[tokio::test]
    async fn test_config_integration() {
        // Test that configuration system works with security system
        env::set_var("PROXEMIC_SECURITY_LEVEL", "development");

        // This would require actual config files in a real test
        // For now, just verify the integration points exist
        // Integration test placeholder - no assertion needed

        env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }
}
