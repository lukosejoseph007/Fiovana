// src-tauri/src/app_config/types.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

// Import workspace types for recent workspaces functionality
use crate::workspace::types::RecentWorkspace;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl FromStr for Environment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Self::Development),
            "staging" | "stage" => Ok(Self::Staging),
            "prod" | "production" => Ok(Self::Production),
            _ => Err(()),
        }
    }
}

impl Environment {
    /// Like FromStr, but falls back to Development instead of Err
    #[allow(dead_code)]
    pub fn from_str_fallback(s: &str) -> Self {
        Self::from_str(s).unwrap_or(Self::Development)
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Self::Development)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub debug: bool,
    pub max_file_handles: usize,
    pub request_timeout_ms: u64,
    pub max_concurrent_operations: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            name: "Proxemic".to_string(),
            version: "0.1.0".to_string(),
            environment: Environment::Development,
            debug: true,
            max_file_handles: 100,
            request_timeout_ms: 30000, // 30 seconds
            max_concurrent_operations: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allowed_extensions: Vec<String>,
    pub max_path_length: usize,
    pub max_file_size: u64,
    pub allowed_mime_types: Vec<String>,
    pub prohibited_filename_chars: Vec<char>,
    pub enable_magic_number_validation: bool,
    pub enable_path_traversal_protection: bool,
    pub enable_file_quarantine: bool,
    pub quarantine_directory: PathBuf,
    pub audit_enabled: bool,
    pub rate_limit_enabled: bool,
    pub max_requests_per_minute: u32,

    // Additional production security settings
    pub require_file_signature_verification: bool,
    pub max_concurrent_file_operations: usize,
    pub enable_file_content_scanning: bool,
    pub quarantine_suspicious_files: bool,
    pub log_all_file_access: bool,
    pub enforce_strict_mime_validation: bool,
    pub block_executable_extensions: bool,
    pub max_filename_length: usize,
    pub enable_sandboxed_file_processing: bool,
}

impl SecurityConfig {
    /// Create production-hardened security configuration
    pub fn production_hardened() -> Self {
        Self {
            // Core file restrictions - very restrictive for production
            allowed_extensions: vec![
                ".pdf".to_string(),
                ".txt".to_string(),
                ".md".to_string(),
                ".docx".to_string(),
            ],
            max_path_length: 200,            // Reduced from 260 for extra safety
            max_file_size: 25 * 1024 * 1024, // 25MB - much more restrictive
            allowed_mime_types: vec![
                "application/pdf".to_string(),
                "text/plain".to_string(),
                "text/markdown".to_string(),
                "text/csv".to_string(),
                "application/json".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
                "application/octet-stream".to_string(), // fallback
            ],

            // Filename restrictions
            prohibited_filename_chars: "<>:\"/\\|?*\0${}[]()&;~`'\"".chars().collect(),
            max_filename_length: 100, // Restrictive filename length

            // Security features - ALL enabled in production
            enable_magic_number_validation: true,
            enable_path_traversal_protection: true,
            enable_file_quarantine: true,
            require_file_signature_verification: true,
            enable_file_content_scanning: true,
            quarantine_suspicious_files: true,
            log_all_file_access: true,
            enforce_strict_mime_validation: true,
            block_executable_extensions: true,
            enable_sandboxed_file_processing: true,

            // Audit and monitoring
            audit_enabled: true,

            // Rate limiting - very restrictive
            rate_limit_enabled: true,
            max_requests_per_minute: 30, // Much more restrictive than development
            max_concurrent_file_operations: 3, // Very conservative

            // Quarantine directory with secure permissions
            quarantine_directory: Self::get_secure_quarantine_path(),
        }
    }

    /// Create development-friendly configuration
    pub fn development_defaults() -> Self {
        Self {
            allowed_extensions: vec![
                ".docx".to_string(),
                ".pdf".to_string(),
                ".md".to_string(),
                ".txt".to_string(),
                ".csv".to_string(), // Additional formats for dev
                ".json".to_string(),
            ],
            max_path_length: 260,
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_mime_types: vec![
                "application/pdf".to_string(),
                "text/plain".to_string(),
                "text/markdown".to_string(),
                "text/csv".to_string(),
                "application/json".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
                "application/octet-stream".to_string(), // fallback
            ],
            prohibited_filename_chars: "<>:\"/\\|?*\0".chars().collect(),
            max_filename_length: 255,

            // Basic security enabled but not as strict
            enable_magic_number_validation: true,
            enable_path_traversal_protection: true,
            enable_file_quarantine: false, // Disabled for easier development
            require_file_signature_verification: false,
            enable_file_content_scanning: false,
            quarantine_suspicious_files: false,
            log_all_file_access: false,
            enforce_strict_mime_validation: false,
            block_executable_extensions: true, // Still important in dev
            enable_sandboxed_file_processing: false,

            audit_enabled: false, // Can be noisy during development
            rate_limit_enabled: true,
            max_requests_per_minute: 200, // More permissive for development
            max_concurrent_file_operations: 10,

            quarantine_directory: std::env::temp_dir().join("proxemic_quarantine"),
        }
    }

    /// Get secure quarantine directory path for production
    fn get_secure_quarantine_path() -> PathBuf {
        // Try to use system-appropriate secure temp location
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            // Linux with XDG runtime directory
            PathBuf::from(runtime_dir).join("proxemic_quarantine")
        } else if let Ok(temp_dir) = std::env::var("PROXEMIC_QUARANTINE_DIR") {
            // Explicit override from environment
            PathBuf::from(temp_dir)
        } else {
            // Fallback to system temp with restricted permissions
            std::env::temp_dir().join("proxemic_secure_quarantine")
        }
    }

    /// Apply environment variable overrides for security settings
    pub fn apply_env_overrides(&mut self) -> Result<(), String> {
        // File size limit override
        if let Ok(max_size_str) = std::env::var("PROXEMIC_MAX_FILE_SIZE") {
            match max_size_str.parse::<u64>() {
                Ok(size) => {
                    if size <= 500 * 1024 * 1024 {
                        // Cap at 500MB
                        self.max_file_size = size;
                    } else {
                        return Err(
                            "PROXEMIC_MAX_FILE_SIZE exceeds maximum allowed (500MB)".to_string()
                        );
                    }
                }
                Err(_) => return Err("Invalid PROXEMIC_MAX_FILE_SIZE value".to_string()),
            }
        }

        // Rate limit override
        if let Ok(rate_str) = std::env::var("PROXEMIC_MAX_REQUESTS_PER_MINUTE") {
            match rate_str.parse::<u32>() {
                Ok(rate) => {
                    if rate <= 1000 {
                        // Reasonable upper bound
                        self.max_requests_per_minute = rate;
                    } else {
                        return Err(
                            "PROXEMIC_MAX_REQUESTS_PER_MINUTE exceeds maximum (1000)".to_string()
                        );
                    }
                }
                Err(_) => return Err("Invalid PROXEMIC_MAX_REQUESTS_PER_MINUTE value".to_string()),
            }
        }

        // Quarantine directory override
        if let Ok(quarantine_dir) = std::env::var("PROXEMIC_QUARANTINE_DIR") {
            let path = PathBuf::from(quarantine_dir);
            if path.is_absolute() {
                self.quarantine_directory = path;
            } else {
                return Err("PROXEMIC_QUARANTINE_DIR must be an absolute path".to_string());
            }
        }

        // Security feature toggles (with validation for production)
        if let Ok(audit_str) = std::env::var("PROXEMIC_AUDIT_ENABLED") {
            self.audit_enabled = audit_str.to_lowercase() == "true";
        }

        if let Ok(rate_limit_str) = std::env::var("PROXEMIC_RATE_LIMIT_ENABLED") {
            self.rate_limit_enabled = rate_limit_str.to_lowercase() == "true";
        }

        if let Ok(magic_str) = std::env::var("PROXEMIC_MAGIC_NUMBER_VALIDATION") {
            self.enable_magic_number_validation = magic_str.to_lowercase() == "true";
        }

        if let Ok(quarantine_str) = std::env::var("PROXEMIC_ENABLE_QUARANTINE") {
            self.enable_file_quarantine = quarantine_str.to_lowercase() == "true";
        }

        Ok(())
    }

    /// Validate security configuration for the given environment
    pub fn validate_for_environment(&self, env: &Environment) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Basic validation
        if self.allowed_extensions.is_empty() {
            errors.push("At least one file extension must be allowed".to_string());
        }

        if self.max_file_size == 0 {
            errors.push("Maximum file size must be greater than 0".to_string());
        }

        if self.max_path_length == 0 || self.max_path_length > 32767 {
            errors.push("Path length must be between 1 and 32767 characters".to_string());
        }

        if self.max_filename_length == 0 || self.max_filename_length > 255 {
            errors.push("Filename length must be between 1 and 255 characters".to_string());
        }

        // Production-specific validation
        if env.is_production() {
            if !self.audit_enabled {
                errors.push("Audit logging must be enabled in production".to_string());
            }

            if !self.rate_limit_enabled {
                errors.push("Rate limiting must be enabled in production".to_string());
            }

            if !self.enable_magic_number_validation {
                errors.push("Magic number validation must be enabled in production".to_string());
            }

            if !self.enable_path_traversal_protection {
                errors.push("Path traversal protection must be enabled in production".to_string());
            }

            if self.max_file_size > 100 * 1024 * 1024 {
                errors.push("Maximum file size should not exceed 100MB in production".to_string());
            }

            if self.max_requests_per_minute > 100 {
                errors.push(
                    "Rate limit should be conservative in production (≤100 requests/minute)"
                        .to_string(),
                );
            }

            // Check for suspicious configurations
            let dangerous_extensions = [".exe", ".bat", ".sh", ".ps1", ".scr", ".com", ".cmd"];
            for ext in &self.allowed_extensions {
                if dangerous_extensions
                    .iter()
                    .any(|&dangerous| ext == dangerous)
                {
                    errors.push(format!(
                        "Executable extension {} is not allowed in production",
                        ext
                    ));
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        // Default to development-friendly settings
        Self::development_defaults()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_enabled: bool,
    pub file_path: Option<PathBuf>,
    pub console_enabled: bool,
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub structured_logging: bool,

    // New fields for log management
    pub aggregation_enabled: bool,
    pub aggregation_endpoints: Vec<String>,
    pub aggregation_protocol: String, // "http", "https", "tcp", "udp"
    pub aggregation_timeout_ms: u64,
    pub aggregation_batch_size: usize,

    // Integrity verification
    pub integrity_checks_enabled: bool,
    pub checksum_algorithm: String, // "sha256", "sha512"
    pub signing_enabled: bool,
    pub signing_key: Option<String>,

    // Security features
    pub encryption_enabled: bool,
    pub encryption_key: Option<String>,
    pub compress_logs: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: "pretty".to_string(),
            file_enabled: false,
            file_path: None,
            console_enabled: true,
            max_file_size_mb: 10,
            max_files: 5,
            structured_logging: false,

            // Default values for new log management fields
            aggregation_enabled: false,
            aggregation_endpoints: Vec::new(),
            aggregation_protocol: "https".to_string(),
            aggregation_timeout_ms: 5000,
            aggregation_batch_size: 100,

            integrity_checks_enabled: false,
            checksum_algorithm: "sha256".to_string(),
            signing_enabled: false,
            signing_key: None,

            encryption_enabled: false,
            encryption_key: None,
            compress_logs: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_ms: u64,
    pub enable_migrations: bool,
    pub backup_enabled: bool,
    pub backup_interval_hours: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite:./Proxemic.db".to_string(),
            max_connections: 10,
            connection_timeout_ms: 30000,
            enable_migrations: true,
            backup_enabled: false,
            backup_interval_hours: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub openrouter_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub max_embedding_batch_size: usize,
    pub vector_index_path: PathBuf,
    pub enable_cloud_sync: bool,
    pub enable_collaboration: bool,
    pub request_timeout_ms: u64,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            openrouter_api_key: None,
            anthropic_api_key: None,
            max_embedding_batch_size: 100,
            vector_index_path: PathBuf::from("./vector_index"),
            enable_cloud_sync: false,
            enable_collaboration: false,
            request_timeout_ms: 60000, // 60 seconds for AI requests
        }
    }
}

/// Workspace manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceManagerConfig {
    pub recent_workspaces: Option<Vec<RecentWorkspace>>,
    pub max_recent: usize,
    pub auto_cleanup_days: u32,
}

impl Default for WorkspaceManagerConfig {
    fn default() -> Self {
        Self {
            recent_workspaces: Some(Vec::new()),
            max_recent: 20,
            auto_cleanup_days: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxemicConfig {
    pub app: AppConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
    pub ai: AIConfig,
    pub workspace: WorkspaceManagerConfig,

    // Additional runtime configuration
    #[serde(skip)]
    pub config_file_path: Option<PathBuf>,
    #[serde(skip)]
    pub loaded_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProxemicConfig {
    /// Create environment-appropriate default configuration
    pub fn for_environment(env: &Environment) -> Self {
        let mut config = Self {
            app: AppConfig {
                environment: env.clone(),
                debug: env.is_development(),
                ..Default::default()
            },
            security: match env {
                Environment::Production => SecurityConfig::production_hardened(),
                _ => SecurityConfig::development_defaults(),
            },
            logging: LoggingConfig {
                level: match env {
                    Environment::Production => "warn".to_string(),
                    Environment::Staging => "info".to_string(),
                    Environment::Development => "debug".to_string(),
                },
                structured_logging: env.is_production(),
                file_enabled: env.is_production(),
                // Production-specific log management settings
                aggregation_enabled: env.is_production(),
                integrity_checks_enabled: env.is_production(),
                compress_logs: env.is_production(),
                ..Default::default()
            },
            database: DatabaseConfig {
                max_connections: match env {
                    Environment::Production => 5,
                    Environment::Staging => 8,
                    Environment::Development => 10,
                },
                backup_enabled: env.is_production(),
                ..Default::default()
            },
            ai: AIConfig {
                max_embedding_batch_size: match env {
                    Environment::Production => 25,
                    Environment::Staging => 50,
                    Environment::Development => 100,
                },
                enable_cloud_sync: false, // Disabled by default for security
                enable_collaboration: false,
                ..Default::default()
            },
            workspace: WorkspaceManagerConfig::default(),
            config_file_path: None,
            loaded_at: None,
        };

        // Apply production hardening
        if env.is_production() {
            config.apply_production_hardening();
        }

        config
    }

    /// Apply production-safe hardening to configuration
    pub fn apply_production_hardening(&mut self) {
        if self.app.environment.is_production() {
            // Disable debug mode in production
            self.app.debug = false;

            // Apply production-hardened security
            self.security = SecurityConfig::production_hardened();

            // Enable file logging in production
            self.logging.file_enabled = true;
            self.logging.structured_logging = true;
            if self.logging.level == "debug" || self.logging.level == "trace" {
                self.logging.level = "warn".to_string();
            }

            // Reduce connection limits for stability
            self.database.max_connections = std::cmp::min(self.database.max_connections, 5);
            self.database.backup_enabled = true;

            // Disable experimental features
            self.ai.enable_cloud_sync = false;
            self.ai.enable_collaboration = false;
            self.ai.max_embedding_batch_size = std::cmp::min(self.ai.max_embedding_batch_size, 25);
        }
    }

    /// Apply environment variable overrides with validation
    pub fn apply_environment_overrides(&mut self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Apply security overrides
        if let Err(e) = self.security.apply_env_overrides() {
            errors.push(format!("Security config error: {}", e));
        }

        // Debug override - fix the logic here
        if let Ok(debug_str) = std::env::var("PROXEMIC_DEBUG") {
            let debug_enabled = debug_str.to_lowercase() == "true";

            // Only validate in production if trying to ENABLE debug
            if self.app.environment.is_production() && debug_enabled {
                errors.push("Debug mode cannot be enabled in production environment".to_string());
            } else {
                // Allow disabling debug in any environment
                self.app.debug = debug_enabled;
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Comprehensive configuration validation
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // App validation
        if self.app.name.is_empty() {
            errors.push("Application name cannot be empty".to_string());
        }

        if self.app.max_file_handles == 0 {
            errors.push("Maximum file handles must be greater than 0".to_string());
        }

        // Security validation
        if let Err(security_errors) = self
            .security
            .validate_for_environment(&self.app.environment)
        {
            errors.extend(security_errors);
        }

        // Logging validation
        let valid_log_levels = ["error", "warn", "info", "debug", "trace"];
        if !valid_log_levels.contains(&self.logging.level.to_lowercase().as_str()) {
            errors.push(format!(
                "Log level must be one of: {}",
                valid_log_levels.join(", ")
            ));
        }

        // Database validation
        if self.database.url.is_empty() {
            errors.push("Database URL cannot be empty".to_string());
        }

        if self.database.max_connections == 0 {
            errors.push("Maximum database connections must be greater than 0".to_string());
        }

        // Production-specific validations
        if self.app.environment.is_production() {
            if self.app.debug {
                errors.push("Debug mode must be disabled in production".to_string());
            }

            if !self.security.audit_enabled {
                errors.push("Audit logging must be enabled in production".to_string());
            }

            if !self.logging.structured_logging {
                errors.push("Structured logging must be enabled in production".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get environment-specific configuration file names
    pub fn config_file_names(env: &Environment) -> Vec<String> {
        match env {
            Environment::Development => vec![
                "proxemic.dev.json".to_string(),
                "config.dev.json".to_string(),
                "proxemic.json".to_string(),
            ],
            Environment::Staging => vec![
                "proxemic.staging.json".to_string(),
                "config.staging.json".to_string(),
                "proxemic.json".to_string(),
            ],
            Environment::Production => vec![
                "proxemic.prod.json".to_string(),
                "proxemic.production.json".to_string(),
                "config.prod.json".to_string(),
                "config.production.json".to_string(),
            ],
        }
    }
}
