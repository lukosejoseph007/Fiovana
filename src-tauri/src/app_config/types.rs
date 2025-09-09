// src-tauri/src/app_config/types.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

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
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_extensions: vec![
                ".docx".to_string(),
                ".pdf".to_string(),
                ".md".to_string(),
                ".txt".to_string(),
            ],
            max_path_length: 260,
            max_file_size: 100 * 1024 * 1024, // 100MB
            allowed_mime_types: vec![
                "application/pdf".to_string(),
                "text/plain".to_string(),
                "text/markdown".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
            ],
            prohibited_filename_chars: "<>:\"/\\|?*\0".chars().collect(),
            enable_magic_number_validation: true,
            enable_path_traversal_protection: true,
            enable_file_quarantine: true,
            quarantine_directory: std::env::temp_dir().join("proxemic_quarantine"),
            audit_enabled: true,
            rate_limit_enabled: true,
            max_requests_per_minute: 100,
        }
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProxemicConfig {
    pub app: AppConfig,
    pub security: SecurityConfig,
    pub logging: LoggingConfig,
    pub database: DatabaseConfig,
    pub ai: AIConfig,

    // Additional runtime configuration
    #[serde(skip)]
    pub config_file_path: Option<PathBuf>,
    #[serde(skip)]
    pub loaded_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl ProxemicConfig {
    /// Apply production-safe hardening to configuration
    pub fn apply_production_hardening(&mut self) {
        if self.app.environment.is_production() {
            // Disable debug mode in production
            self.app.debug = false;

            // Enable all security features
            self.security.enable_magic_number_validation = true;
            self.security.enable_path_traversal_protection = true;
            self.security.enable_file_quarantine = true;
            self.security.audit_enabled = true;
            self.security.rate_limit_enabled = true;

            // Reduce file size limits for production
            self.security.max_file_size = 50 * 1024 * 1024; // 50MB in production

            // Enable file logging in production
            self.logging.file_enabled = true;
            self.logging.structured_logging = true;
            self.logging.level = "warn".to_string();

            // Reduce connection limits for stability
            self.database.max_connections = 5;

            // Disable experimental features
            self.ai.enable_cloud_sync = false;
            self.ai.enable_collaboration = false;
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
