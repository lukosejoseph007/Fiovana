// src-tauri/src/app_config/mod.rs
use crate::app_config::encryption::ConfigEncryption;
use crate::app_config::errors::{ConfigError, ConfigResult};
use crate::app_config::types::{AppConfig, Environment, FiovanaConfig, SecurityConfig};
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::fs;

pub mod encryption;
pub mod errors;
pub mod types;

/// Configuration manager for the Fiovana application
/// Handles loading, validation, and management of application configuration
pub struct ConfigManager {
    config: Arc<RwLock<FiovanaConfig>>,
    #[allow(dead_code)]
    config_paths: Vec<PathBuf>,
    environment: Environment,
}

impl ConfigManager {
    /// Create a new ConfigManager and load configuration
    pub async fn new() -> ConfigResult<Self> {
        let environment = Self::detect_environment();
        let config_paths = Self::get_config_search_paths(&environment)?;

        // Start with environment-appropriate defaults
        let mut config = FiovanaConfig::for_environment(&environment);

        // Try to load configuration from files
        if let Ok(file_config) = Self::load_configuration(&config_paths, &environment).await {
            Self::merge_configurations(&mut config, file_config);
        }

        // Apply environment variable overrides with validation
        if let Err(override_errors) = config.apply_environment_overrides() {
            return Err(ConfigError::ValidationError {
                field: "environment_overrides".to_string(),
                message: format!(
                    "Environment override errors: {}",
                    override_errors.join(", ")
                ),
            });
        }

        // Comprehensive configuration validation
        if let Err(validation_errors) = config.validate() {
            return Err(ConfigError::ValidationError {
                field: "configuration".to_string(),
                message: format!(
                    "Configuration validation failed: {}",
                    validation_errors.join(", ")
                ),
            });
        }

        // Apply production hardening (redundant but ensures consistency)
        if environment.is_production() {
            config.apply_production_hardening();
        }

        // Decrypt sensitive fields
        ConfigEncryption::decrypt_sensitive_config(&mut config)?;

        // Update runtime metadata
        config.loaded_at = Some(chrono::Utc::now());

        let manager = Self {
            config: Arc::new(RwLock::new(config)),
            config_paths,
            environment,
        };

        // Log security configuration status
        manager.log_security_status();

        Ok(manager)
    }

    /// Log the current security configuration status
    fn log_security_status(&self) {
        if let Ok(config) = self.config.read() {
            let env_name = format!("{:?}", self.environment);

            if self.environment.is_production() {
                println!("🔒 Production security hardening applied:");
                println!(
                    "   - Max file size: {} MB",
                    config.security.max_file_size / (1024 * 1024)
                );
                println!(
                    "   - Rate limit: {} req/min",
                    config.security.max_requests_per_minute
                );
                println!("   - Audit enabled: {}", config.security.audit_enabled);
                println!(
                    "   - File quarantine: {}",
                    config.security.enable_file_quarantine
                );
                println!(
                    "   - Magic number validation: {}",
                    config.security.enable_magic_number_validation
                );
                println!(
                    "   - Strict MIME validation: {}",
                    config.security.enforce_strict_mime_validation
                );
            } else {
                println!("🔧 {} environment security configuration loaded", env_name);
            }
        }
    }

    /// Get a read-only reference to the current configuration
    pub fn get_config(&self) -> Arc<RwLock<FiovanaConfig>> {
        Arc::clone(&self.config)
    }

    /// Reload configuration from disk with full validation
    #[allow(dead_code)]
    pub async fn reload(&self) -> ConfigResult<()> {
        // Create new configuration with environment-appropriate defaults
        let mut new_config = FiovanaConfig::for_environment(&self.environment);

        // Try to load from files
        if let Ok(file_config) =
            Self::load_configuration(&self.config_paths, &self.environment).await
        {
            Self::merge_configurations(&mut new_config, file_config);
        }

        // Apply environment overrides with validation
        if let Err(override_errors) = new_config.apply_environment_overrides() {
            return Err(ConfigError::ValidationError {
                field: "environment_overrides".to_string(),
                message: format!(
                    "Environment override errors: {}",
                    override_errors.join(", ")
                ),
            });
        }

        // Validate the new configuration
        if let Err(validation_errors) = new_config.validate() {
            return Err(ConfigError::ValidationError {
                field: "configuration".to_string(),
                message: format!(
                    "Configuration validation failed: {}",
                    validation_errors.join(", ")
                ),
            });
        }

        // Apply production hardening if needed
        if self.environment.is_production() {
            new_config.apply_production_hardening();
        }

        ConfigEncryption::decrypt_sensitive_config(&mut new_config)?;
        new_config.loaded_at = Some(chrono::Utc::now());

        // Update the stored configuration
        let mut config_guard = self
            .config
            .write()
            .map_err(|_| ConfigError::ValidationError {
                field: "config_lock".to_string(),
                message: "Failed to acquire write lock for configuration".to_string(),
            })?;
        *config_guard = new_config;

        println!("Configuration reloaded and validated successfully");
        drop(config_guard);
        self.log_security_status();

        Ok(())
    }

    /// Save current configuration to disk (with sensitive values encrypted)
    #[allow(dead_code)]
    pub async fn save_configuration(&self, path: Option<PathBuf>) -> ConfigResult<()> {
        // Scope the lock so it's released ASAP
        let config_to_save = {
            let config_guard = self
                .config
                .read()
                .map_err(|_| ConfigError::ValidationError {
                    field: "config_lock".to_string(),
                    message: "Failed to acquire read lock for configuration".to_string(),
                })?;

            let mut cloned = config_guard.clone();
            ConfigEncryption::encrypt_sensitive_config(&mut cloned)?;
            cloned
        }; // <-- lock dropped here

        let save_path = path.unwrap_or_else(|| {
            self.config_paths.first().cloned().unwrap_or_else(|| {
                PathBuf::from(format!(
                    "fiovana.{}.json",
                    match self.environment {
                        Environment::Development => "dev",
                        Environment::Staging => "staging",
                        Environment::Production => "prod",
                    }
                ))
            })
        });

        // Create directory if it doesn't exist
        if let Some(parent) = save_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let json_content = serde_json::to_string_pretty(&config_to_save)?;
        fs::write(&save_path, json_content).await?;

        println!("Configuration saved to: {}", save_path.display());
        Ok(())
    }

    /// Detect the current environment from various sources with validation
    fn detect_environment() -> Environment {
        // Try environment variables in order of precedence
        if let Ok(env_str) = env::var("FIOVANA_ENV") {
            if let Ok(env) = Environment::from_str(&env_str) {
                println!("Environment detected from FIOVANA_ENV: {:?}", env);
                return env;
            } else {
                eprintln!(
                    "Warning: Invalid FIOVANA_ENV value '{}', falling back",
                    env_str
                );
            }
        }

        if let Ok(env_str) = env::var("RUST_ENV") {
            if let Ok(env) = Environment::from_str(&env_str) {
                println!("Environment detected from RUST_ENV: {:?}", env);
                return env;
            }
        }

        if let Ok(env_str) = env::var("NODE_ENV") {
            if let Ok(env) = Environment::from_str(&env_str) {
                println!("Environment detected from NODE_ENV: {:?}", env);
                return env;
            }
        }

        // Heuristic detection based on other environment variables
        if env::var("RUST_LOG").unwrap_or_default().contains("debug") {
            println!("Environment detected from RUST_LOG (debug): Development");
            return Environment::Development;
        }

        // Check for production indicators
        if env::var("PRODUCTION").is_ok()
            || env::var("PROD").is_ok()
            || env::var("NODE_ENV").unwrap_or_default() == "production"
        {
            println!("Environment detected from production indicators: Production");
            return Environment::Production;
        }

        // Default fallback
        println!("No environment specified, defaulting to Development");
        Environment::Development
    }

    /// Get configuration file search paths based on environment
    fn get_config_search_paths(environment: &Environment) -> ConfigResult<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Get application config directory
        let app_config_dir = Self::get_app_config_directory()?;

        // Add environment-specific paths in order of priority
        for filename in FiovanaConfig::config_file_names(environment) {
            // First check in app config directory
            paths.push(app_config_dir.join(&filename));
            // Then check current working directory
            paths.push(PathBuf::from(&filename));
            // Also check in src-tauri/src/app_config/ for bundled configs
            paths.push(PathBuf::from("src-tauri/src/app_config").join(&filename));
        }

        // Add generic fallback paths
        paths.push(app_config_dir.join("fiovana.json"));
        paths.push(PathBuf::from("fiovana.json"));

        Ok(paths)
    }

    /// Get the application's configuration directory
    fn get_app_config_directory() -> ConfigResult<PathBuf> {
        if let Ok(config_dir_override) = env::var("FIOVANA_CONFIG_DIR") {
            return Ok(PathBuf::from(config_dir_override));
        }

        if let Some(config_dir) = dirs::config_dir() {
            let app_config_dir = config_dir.join("fiovana");

            // Try to create the directory if it doesn't exist
            if !app_config_dir.exists() {
                if let Err(e) = std::fs::create_dir_all(&app_config_dir) {
                    eprintln!(
                        "Warning: Failed to create config directory {}: {}",
                        app_config_dir.display(),
                        e
                    );
                    // Fall back to current directory
                    return Ok(PathBuf::from("."));
                }
            }

            Ok(app_config_dir)
        } else {
            // Fallback to current directory
            Ok(PathBuf::from("."))
        }
    }

    /// Load configuration from available files
    async fn load_configuration(
        config_paths: &[PathBuf],
        environment: &Environment,
    ) -> ConfigResult<FiovanaConfig> {
        let mut config = FiovanaConfig::for_environment(environment);
        let mut found_config = false;

        // Try to load from each path in order
        for path in config_paths {
            if path.exists() {
                match Self::load_config_file(path).await {
                    Ok(file_config) => {
                        Self::merge_configurations(&mut config, file_config);
                        config.config_file_path = Some(path.clone());
                        found_config = true;
                        println!("Configuration loaded from: {}", path.display());
                        break;
                    }
                    Err(e) => {
                        eprintln!(
                            "Warning: Failed to load config from {}: {}",
                            path.display(),
                            e
                        );
                        continue;
                    }
                }
            }
        }

        if !found_config {
            println!(
                "No configuration file found, using defaults for {:?} environment",
                environment
            );
        }

        Ok(config)
    }

    /// Load a single configuration file
    async fn load_config_file(path: &Path) -> ConfigResult<FiovanaConfig> {
        let content = fs::read_to_string(path).await?;
        let config: FiovanaConfig =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError {
                message: format!("Failed to parse JSON from {}: {}", path.display(), e),
            })?;
        Ok(config)
    }

    /// Merge configuration from file into base configuration
    fn merge_configurations(base: &mut FiovanaConfig, file_config: FiovanaConfig) {
        // Only merge non-default values from file config

        // App config merging
        if file_config.app.name != AppConfig::default().name {
            base.app.name = file_config.app.name;
        }
        if file_config.app.version != AppConfig::default().version {
            base.app.version = file_config.app.version;
        }
        if file_config.app.debug != base.app.debug {
            base.app.debug = file_config.app.debug;
        }
        if file_config.app.max_file_handles != AppConfig::default().max_file_handles {
            base.app.max_file_handles = file_config.app.max_file_handles;
        }
        if file_config.app.request_timeout_ms != AppConfig::default().request_timeout_ms {
            base.app.request_timeout_ms = file_config.app.request_timeout_ms;
        }
        if file_config.app.max_concurrent_operations
            != AppConfig::default().max_concurrent_operations
        {
            base.app.max_concurrent_operations = file_config.app.max_concurrent_operations;
        }

        // Security config merging
        if !file_config.security.allowed_extensions.is_empty() {
            base.security.allowed_extensions = file_config.security.allowed_extensions;
        }
        if !file_config.security.allowed_mime_types.is_empty() {
            base.security.allowed_mime_types = file_config.security.allowed_mime_types;
        }
        if file_config.security.max_file_size != SecurityConfig::default().max_file_size {
            base.security.max_file_size = file_config.security.max_file_size;
        }
        if file_config.security.max_path_length != SecurityConfig::default().max_path_length {
            base.security.max_path_length = file_config.security.max_path_length;
        }
        if file_config.security.max_requests_per_minute
            != SecurityConfig::default().max_requests_per_minute
        {
            base.security.max_requests_per_minute = file_config.security.max_requests_per_minute;
        }

        // Merge boolean flags
        base.security.audit_enabled = file_config.security.audit_enabled;
        base.security.rate_limit_enabled = file_config.security.rate_limit_enabled;
        base.security.enable_magic_number_validation =
            file_config.security.enable_magic_number_validation;
        base.security.enable_path_traversal_protection =
            file_config.security.enable_path_traversal_protection;
        base.security.enable_file_quarantine = file_config.security.enable_file_quarantine;

        // Always merge these subsections completely
        base.logging = file_config.logging;
        base.database = file_config.database;
        base.ai = file_config.ai;
    }

    /// Public accessor methods
    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    #[allow(dead_code)]
    pub fn is_production(&self) -> bool {
        self.environment.is_production()
    }

    #[allow(dead_code)]
    pub fn is_development(&self) -> bool {
        self.environment.is_development()
    }

    pub fn config_file_path(&self) -> Option<PathBuf> {
        let config_guard = self.config.read().ok()?;
        config_guard.config_file_path.clone()
    }

    #[allow(dead_code)]
    pub fn loaded_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        let config_guard = self.config.read().ok()?;
        config_guard.loaded_at
    }

    /// Get current security settings (convenient accessor)
    pub fn get_security_config(&self) -> Option<SecurityConfig> {
        let config_guard = self.config.read().ok()?;
        Some(config_guard.security.clone())
    }

    /// Update a specific configuration section
    #[allow(dead_code)]
    pub fn update_security_config(&self, new_security_config: SecurityConfig) -> ConfigResult<()> {
        let mut config_guard = self
            .config
            .write()
            .map_err(|_| ConfigError::ValidationError {
                field: "config_lock".to_string(),
                message: "Failed to acquire write lock for configuration".to_string(),
            })?;

        // Validate the new security config
        if let Err(errors) = new_security_config.validate_for_environment(&self.environment) {
            return Err(ConfigError::ValidationError {
                field: "security".to_string(),
                message: format!("Security config validation failed: {}", errors.join(", ")),
            });
        }

        config_guard.security = new_security_config;
        Ok(())
    }

    /// Check if configuration is stale and needs reloading
    #[allow(dead_code)]
    pub fn is_stale(&self, max_age_hours: u64) -> bool {
        if let Some(loaded_at) = self.loaded_at() {
            let age = chrono::Utc::now().signed_duration_since(loaded_at);
            // Convert to total seconds for more precision with small time intervals
            let age_seconds = age.num_seconds();
            let max_age_seconds = max_age_hours as i64 * 3600;
            age_seconds >= max_age_seconds
        } else {
            true // No load time recorded, consider stale
        }
    }
}

/// Initialize the configuration system
pub async fn init() -> ConfigResult<ConfigManager> {
    println!("Initializing Fiovana configuration system...");

    let config_manager = ConfigManager::new().await?;

    let env = config_manager.environment();
    println!(
        "Configuration loaded successfully for {:?} environment",
        env
    );

    if let Some(config_path) = config_manager.config_file_path() {
        println!("Configuration file: {}", config_path.display());
    } else {
        println!("Using default configuration (no config file found)");
    }

    // Log some key configuration details
    if let Ok(config) = config_manager.config.read() {
        println!("App: {} v{}", config.app.name, config.app.version);
        println!("Debug mode: {}", config.app.debug);
        println!(
            "Max file size: {} MB",
            config.security.max_file_size / (1024 * 1024)
        );
        println!(
            "Allowed extensions: {:?}",
            config.security.allowed_extensions
        );
    }

    Ok(config_manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;

    #[tokio::test]
    #[serial]
    async fn test_config_manager_creation() {
        std::env::set_var("FIOVANA_ENV", "Development");

        let manager = ConfigManager::new().await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.environment().is_development());

        std::env::remove_var("FIOVANA_ENV");
    }

    #[tokio::test]
    #[serial]
    async fn test_production_environment_hardening() {
        std::env::set_var("FIOVANA_ENV", "Production");

        let manager = ConfigManager::new().await.unwrap();
        assert!(manager.is_production());

        if let Ok(config) = manager.config.read() {
            assert!(!config.app.debug); // Debug should be disabled in production
            assert!(config.security.audit_enabled); // Audit should be enabled
            assert!(config.security.rate_limit_enabled); // Rate limiting should be enabled
        }

        std::env::remove_var("FIOVANA_ENV");
    }

    #[tokio::test]
    #[serial]
    async fn test_config_file_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("fiovana.dev.json");

        let test_config = FiovanaConfig {
            app: AppConfig {
                name: "Test App".to_string(),
                debug: false,
                ..Default::default()
            },
            ..Default::default()
        };

        let json_content = serde_json::to_string_pretty(&test_config).unwrap();
        tokio::fs::write(&config_path, json_content).await.unwrap();

        let loaded_config = ConfigManager::load_config_file(&config_path).await.unwrap();
        assert_eq!(loaded_config.app.name, "Test App");
        assert!(!loaded_config.app.debug);
    }

    #[tokio::test]
    #[serial]
    async fn test_environment_detection() {
        std::env::set_var("FIOVANA_ENV", "Production");
        let env = ConfigManager::detect_environment();
        assert!(env.is_production());

        std::env::set_var("FIOVANA_ENV", "Development");
        let env = ConfigManager::detect_environment();
        assert!(env.is_development());

        std::env::remove_var("FIOVANA_ENV");

        // Test fallback
        let env = ConfigManager::detect_environment();
        assert!(env.is_development()); // Should default to development
    }

    #[tokio::test]
    #[serial]
    async fn test_environment_overrides() {
        // Clear all environment variables first
        std::env::remove_var("FIOVANA_ENV");
        std::env::remove_var("RUST_ENV");
        std::env::remove_var("NODE_ENV");
        std::env::remove_var("FIOVANA_DEBUG");
        std::env::remove_var("FIOVANA_MAX_FILE_SIZE");

        // Set specific test environment
        std::env::set_var("FIOVANA_ENV", "Development"); // Explicitly set development
        std::env::set_var("FIOVANA_DEBUG", "false");
        std::env::set_var("FIOVANA_MAX_FILE_SIZE", "1048576"); // 1MB

        let manager = ConfigManager::new().await.unwrap();

        if let Ok(config) = manager.config.read() {
            // The test should pass now - debug should be false due to env override
            assert!(
                !config.app.debug,
                "Debug should be false due to FIOVANA_DEBUG=false"
            );
            assert_eq!(config.security.max_file_size, 1048576);
        }

        // Clean up
        std::env::remove_var("FIOVANA_DEBUG");
        std::env::remove_var("FIOVANA_MAX_FILE_SIZE");
        std::env::remove_var("FIOVANA_ENV");
    }

    #[test]
    fn test_config_validation() {
        let mut config = FiovanaConfig::default();

        // Valid config should pass
        assert!(config.validate().is_ok());

        // Empty app name should fail
        config.app.name = String::new();
        assert!(config.validate().is_err());

        // Reset and test production validation
        config = FiovanaConfig::for_environment(&Environment::Production);
        config.app.debug = true; // This should fail in production
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_config_reload() {
        let manager = ConfigManager::new().await.unwrap();

        // Should be able to reload without errors
        let result = manager.reload().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_staleness() {
        let manager_result = std::thread::spawn(|| {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(async { ConfigManager::new().await })
        })
        .join()
        .unwrap();

        let manager = manager_result.unwrap();

        // Fresh config should not be stale
        assert!(!manager.is_stale(1)); // 1 hour threshold

        // Very short threshold should make it stale
        assert!(manager.is_stale(0)); // 0 hour threshold
    }
}
