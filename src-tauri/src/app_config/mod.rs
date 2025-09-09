// src-tauri/src/app_config/mod.rs
use crate::app_config::encryption::ConfigEncryption;
use crate::app_config::errors::{ConfigError, ConfigResult};
use crate::app_config::types::{AppConfig, Environment, ProxemicConfig, SecurityConfig};
use std::env;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use tokio::fs;

pub mod encryption;
pub mod errors;
pub mod types;

/// Configuration manager for the Proxemic application
/// Handles loading, validation, and management of application configuration
pub struct ConfigManager {
    config: Arc<RwLock<ProxemicConfig>>,
    config_paths: Vec<PathBuf>,
    environment: Environment,
}

impl ConfigManager {
    /// Create a new ConfigManager and load configuration
    pub async fn new() -> ConfigResult<Self> {
        let environment = Self::detect_environment();
        let config_paths = Self::get_config_search_paths(&environment)?;
        let mut config = Self::load_configuration(&config_paths, &environment).await?;

        // Apply environment-specific settings
        Self::apply_environment_overrides(&mut config, &environment)?;

        // Validate configuration
        Self::validate_configuration(&config)?;

        // Apply production hardening if needed
        if environment.is_production() {
            config.apply_production_hardening();
        }

        // Decrypt sensitive fields
        ConfigEncryption::decrypt_sensitive_config(&mut config)?;

        // Update runtime metadata
        config.loaded_at = Some(chrono::Utc::now());

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_paths,
            environment,
        })
    }

    /// Get a read-only reference to the current configuration
    pub fn get_config(&self) -> Arc<RwLock<ProxemicConfig>> {
        Arc::clone(&self.config)
    }

    /// Reload configuration from disk
    pub async fn reload(&self) -> ConfigResult<()> {
        let mut new_config =
            Self::load_configuration(&self.config_paths, &self.environment).await?;

        Self::apply_environment_overrides(&mut new_config, &self.environment)?;
        Self::validate_configuration(&new_config)?;

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

        Ok(())
    }

    /// Save current configuration to disk (with sensitive values encrypted)
    pub async fn save_configuration(&self, path: Option<PathBuf>) -> ConfigResult<()> {
        // Scope the lock so it’s released ASAP
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
                    "proxemic.{}.json",
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

        Ok(())
    }

    /// Detect the current environment from various sources
    fn detect_environment() -> Environment {
        if let Ok(env_str) = env::var("PROXEMIC_ENV") {
            if let Ok(env) = Environment::from_str(&env_str) {
                return env;
            }
        }
        if let Ok(env_str) = env::var("RUST_ENV") {
            if let Ok(env) = Environment::from_str(&env_str) {
                return env;
            }
        }
        if let Ok(env_str) = env::var("NODE_ENV") {
            if let Ok(env) = Environment::from_str(&env_str) {
                return env;
            }
        }

        if env::var("RUST_LOG").unwrap_or_default().contains("debug") {
            return Environment::Development;
        }

        Environment::Development
    }

    /// Get configuration file search paths based on environment
    fn get_config_search_paths(environment: &Environment) -> ConfigResult<Vec<PathBuf>> {
        let mut paths = Vec::new();

        // Get application config directory
        let app_config_dir = Self::get_app_config_directory()?;

        // Add environment-specific paths
        for filename in ProxemicConfig::config_file_names(environment) {
            paths.push(app_config_dir.join(&filename));
            paths.push(PathBuf::from(&filename)); // Current directory
        }

        // Add default fallback path
        paths.push(PathBuf::from("proxemic.json"));

        Ok(paths)
    }

    /// Get the application's configuration directory
    fn get_app_config_directory() -> ConfigResult<PathBuf> {
        if let Some(config_dir) = dirs::config_dir() {
            Ok(config_dir.join("proxemic"))
        } else {
            // Fallback to current directory
            Ok(PathBuf::from("."))
        }
    }

    /// Load configuration from available files
    async fn load_configuration(
        config_paths: &[PathBuf],
        environment: &Environment,
    ) -> ConfigResult<ProxemicConfig> {
        let mut config = ProxemicConfig::default();
        config.app.environment = environment.clone();

        let mut found_config = false;

        // Try to load from each path in order
        for path in config_paths {
            if path.exists() {
                match Self::load_config_file(path).await {
                    Ok(file_config) => {
                        Self::merge_configurations(&mut config, file_config);
                        config.config_file_path = Some(path.clone());
                        found_config = true;
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
    async fn load_config_file(path: &Path) -> ConfigResult<ProxemicConfig> {
        let content = fs::read_to_string(path).await?;
        let config: ProxemicConfig =
            serde_json::from_str(&content).map_err(|e| ConfigError::ParseError {
                message: format!("Failed to parse JSON from {}: {}", path.display(), e),
            })?;
        Ok(config)
    }

    /// Merge configuration from file into base configuration
    fn merge_configurations(base: &mut ProxemicConfig, file_config: ProxemicConfig) {
        if file_config.app.name != AppConfig::default().name {
            base.app.name = file_config.app.name;
        }
        if file_config.app.version != AppConfig::default().version {
            base.app.version = file_config.app.version;
        }
        if file_config.app.debug != AppConfig::default().debug {
            base.app.debug = file_config.app.debug;
        }

        if !file_config.security.allowed_extensions.is_empty() {
            base.security.allowed_extensions = file_config.security.allowed_extensions;
        }
        if file_config.security.max_file_size != SecurityConfig::default().max_file_size {
            base.security.max_file_size = file_config.security.max_file_size;
        }

        base.logging = file_config.logging;
        base.database = file_config.database;
        base.ai = file_config.ai;
    }

    /// Apply environment variable overrides
    fn apply_environment_overrides(
        config: &mut ProxemicConfig,
        environment: &Environment,
    ) -> ConfigResult<()> {
        if let Ok(log_level) = env::var("RUST_LOG") {
            config.logging.level = log_level;
        }

        if let Ok(debug_str) = env::var("PROXEMIC_DEBUG") {
            config.app.debug = debug_str.to_lowercase() == "true";
        }

        if let Ok(db_url) = env::var("DATABASE_URL") {
            config.database.url = db_url;
        }

        if let Ok(api_key) = env::var("OPENROUTER_API_KEY") {
            config.ai.openrouter_api_key = Some(api_key);
        }

        if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            config.ai.anthropic_api_key = Some(api_key);
        }

        if let Ok(max_size_str) = env::var("PROXEMIC_MAX_FILE_SIZE") {
            if let Ok(max_size) = max_size_str.parse::<u64>() {
                config.security.max_file_size = max_size;
            }
        }

        if let Ok(vector_path) = env::var("VECTOR_INDEX_PATH") {
            config.ai.vector_index_path = PathBuf::from(vector_path);
        }

        if let Ok(batch_size_str) = env::var("MAX_EMBEDDING_BATCH_SIZE") {
            if let Ok(batch_size) = batch_size_str.parse::<usize>() {
                config.ai.max_embedding_batch_size = batch_size;
            }
        }

        match environment {
            Environment::Production => {
                config.app.debug = false;
                config.security.audit_enabled = true;
                config.logging.structured_logging = true;
            }
            Environment::Development => {
                if env::var("PROXEMIC_DEBUG").is_err() {
                    config.app.debug = true;
                }
                config.logging.console_enabled = true;
            }
            Environment::Staging => {
                config.logging.structured_logging = true;
            }
        }

        Ok(())
    }

    /// Validate configuration
    fn validate_configuration(config: &ProxemicConfig) -> ConfigResult<()> {
        if config.app.name.is_empty() {
            return Err(ConfigError::ValidationError {
                field: "app.name".to_string(),
                message: "Application name cannot be empty".to_string(),
            });
        }

        if config.app.max_file_handles == 0 {
            return Err(ConfigError::ValidationError {
                field: "app.max_file_handles".to_string(),
                message: "Maximum file handles must be greater than 0".to_string(),
            });
        }

        if config.security.allowed_extensions.is_empty() {
            return Err(ConfigError::ValidationError {
                field: "security.allowed_extensions".to_string(),
                message: "At least one file extension must be allowed".to_string(),
            });
        }

        if config.security.max_file_size == 0 {
            return Err(ConfigError::ValidationError {
                field: "security.max_file_size".to_string(),
                message: "Maximum file size must be greater than 0".to_string(),
            });
        }

        if config.security.max_path_length == 0 || config.security.max_path_length > 32767 {
            return Err(ConfigError::ValidationError {
                field: "security.max_path_length".to_string(),
                message: "Path length must be between 1 and 32767 characters".to_string(),
            });
        }

        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&config.logging.level.to_lowercase().as_str()) {
            return Err(ConfigError::ValidationError {
                field: "logging.level".to_string(),
                message: format!("Log level must be one of: {}", valid_log_levels.join(", ")),
            });
        }

        if config.database.url.is_empty() {
            return Err(ConfigError::ValidationError {
                field: "database.url".to_string(),
                message: "Database URL cannot be empty".to_string(),
            });
        }

        if config.database.max_connections == 0 {
            return Err(ConfigError::ValidationError {
                field: "database.max_connections".to_string(),
                message: "Maximum database connections must be greater than 0".to_string(),
            });
        }

        if config.app.environment.is_production() {
            if config.app.debug {
                return Err(ConfigError::ValidationError {
                    field: "app.debug".to_string(),
                    message: "Debug mode must be disabled in production".to_string(),
                });
            }

            if !config.security.audit_enabled {
                return Err(ConfigError::ValidationError {
                    field: "security.audit_enabled".to_string(),
                    message: "Audit logging must be enabled in production".to_string(),
                });
            }

            if !config.logging.structured_logging {
                return Err(ConfigError::ValidationError {
                    field: "logging.structured_logging".to_string(),
                    message: "Structured logging must be enabled in production".to_string(),
                });
            }
        }

        Ok(())
    }

    pub fn environment(&self) -> &Environment {
        &self.environment
    }

    pub fn is_production(&self) -> bool {
        self.environment.is_production()
    }

    pub fn is_development(&self) -> bool {
        self.environment.is_development()
    }

    pub fn config_file_path(&self) -> Option<PathBuf> {
        let config_guard = self.config.read().ok()?;
        config_guard.config_file_path.clone()
    }

    pub fn loaded_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        let config_guard = self.config.read().ok()?;
        config_guard.loaded_at
    }
}

/// Initialize the configuration system
pub async fn init() -> ConfigResult<ConfigManager> {
    println!("Initializing Proxemic configuration system...");

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
        std::env::set_var("PROXEMIC_ENV", "Development"); // ✅ proper casing

        let manager = ConfigManager::new().await;
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.environment().is_development());

        std::env::remove_var("PROXEMIC_ENV");
    }

    #[tokio::test]
    #[serial]
    async fn test_config_file_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("proxemic.json");

        let test_config = ProxemicConfig {
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
        std::env::set_var("PROXEMIC_ENV", "Production");
        let env = ConfigManager::detect_environment();
        assert!(env.is_production());

        std::env::set_var("PROXEMIC_ENV", "Development");
        let env = ConfigManager::detect_environment();
        assert!(env.is_development());

        std::env::remove_var("PROXEMIC_ENV");
    }

    #[test]
    fn test_config_validation() {
        let mut config = ProxemicConfig::default();

        assert!(ConfigManager::validate_configuration(&config).is_ok());

        config.app.name = String::new();
        assert!(ConfigManager::validate_configuration(&config).is_err());

        config = ProxemicConfig::default();
        config.app.environment = Environment::Production;
        config.app.debug = true;
        assert!(ConfigManager::validate_configuration(&config).is_err());
    }
}
