// src-tauri/src/filesystem/security/security_config.rs
// Enhanced security configuration with production-hardened defaults and environment overrides
// Compatible with existing filesystem operations

use crate::app_config::types::SecurityConfig as AppSecurityConfig;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Security configuration validation errors
#[derive(Debug, thiserror::Error)]
pub enum SecurityConfigError {
    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },
    #[error("Environment variable error: {var} - {error}")]
    EnvVarError { var: String, error: String },
    #[error("Schema validation failed: {errors:?}")]
    SchemaValidation { errors: Vec<String> },
}

/// Production security levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    Development,
    Production,
    HighSecurity,
}

/// Enhanced SecurityConfig with validation and environment overrides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub allowed_extensions: HashSet<String>,
    pub max_path_length: usize,
    pub max_file_size: u64,
    pub allowed_mime_types: HashSet<String>,
    pub allowed_workspace_paths: Vec<PathBuf>,
    pub temp_directory: PathBuf,
    pub prohibited_filename_chars: HashSet<char>,
    pub enable_magic_number_validation: bool,
    pub magic_number_map: HashMap<String, Vec<Vec<u8>>>,
    pub security_level: SecurityLevel,
    pub enforce_workspace_boundaries: bool,
    pub max_concurrent_operations: u32,
    pub audit_logging_enabled: bool,
}

impl SecurityConfig {
    /// Create SecurityConfig with production-hardened defaults
    pub fn production_defaults() -> Self {
        Self {
            allowed_extensions: Self::production_allowed_extensions(),
            max_path_length: 260, // Windows MAX_PATH limit for safety
            max_file_size: 100 * 1024 * 1024, // 100MB limit for production
            allowed_mime_types: Self::production_allowed_mime_types(),
            allowed_workspace_paths: Self::get_restricted_workspace_paths(),
            temp_directory: std::env::temp_dir(),
            prohibited_filename_chars: Self::strict_prohibited_chars(),
            enable_magic_number_validation: true, // Always enabled in production
            magic_number_map: Self::create_magic_number_map(),
            security_level: SecurityLevel::Production,
            enforce_workspace_boundaries: true, // Strict enforcement
            max_concurrent_operations: 10,      // Prevent resource exhaustion
            audit_logging_enabled: true,        // Always audit in production
        }
    }

    /// Create SecurityConfig with high-security defaults
    pub fn high_security_defaults() -> Self {
        let mut config = Self::production_defaults();
        config.security_level = SecurityLevel::HighSecurity;
        config.max_file_size = 50 * 1024 * 1024; // 50MB for high security
        config.max_path_length = 200; // Even stricter path limits
        config.max_concurrent_operations = 5; // Very limited concurrency
        config.allowed_extensions = Self::high_security_allowed_extensions();
        config
    }

    /// Create SecurityConfig from AppSecurityConfig with environment overrides
    pub fn from_app_config_with_env(
        app_config: &AppSecurityConfig,
    ) -> Result<Self, SecurityConfigError> {
        let mut config = Self::from_app_config(app_config);

        // Apply environment variable overrides
        config.apply_environment_overrides()?;

        // Validate the final configuration
        config.validate()?;

        Ok(config)
    }

    /// Legacy compatibility method - maintains original behavior
    pub fn from_app_config(app_config: &AppSecurityConfig) -> Self {
        let allowed_extensions: HashSet<String> =
            app_config.allowed_extensions.iter().cloned().collect();
        let allowed_mime_types: HashSet<String> =
            app_config.allowed_mime_types.iter().cloned().collect();
        let prohibited_chars: HashSet<char> = app_config
            .prohibited_filename_chars
            .iter()
            .cloned()
            .collect();

        Self {
            allowed_extensions,
            max_path_length: app_config.max_path_length,
            max_file_size: app_config.max_file_size,
            allowed_mime_types,
            allowed_workspace_paths: Self::get_default_workspace_paths(),
            temp_directory: std::env::temp_dir(),
            prohibited_filename_chars: prohibited_chars,
            enable_magic_number_validation: app_config.enable_magic_number_validation,
            magic_number_map: Self::create_magic_number_map(),
            security_level: SecurityLevel::Development, // Default to development for legacy compatibility
            enforce_workspace_boundaries: true,
            max_concurrent_operations: 50, // More permissive for development
            audit_logging_enabled: false,  // Disabled by default in development
        }
    }

    /// Apply environment variable overrides with fixed error types
    pub fn apply_environment_overrides(&mut self) -> Result<(), SecurityConfigError> {
        // Security level override
        if let Ok(level_str) = std::env::var("PROXEMIC_SECURITY_LEVEL") {
            match level_str.to_lowercase().as_str() {
                "development" => self.security_level = SecurityLevel::Development,
                "production" => {
                    self.security_level = SecurityLevel::Production;
                    self.apply_production_overrides();
                }
                "high_security" => {
                    self.security_level = SecurityLevel::HighSecurity;
                    self.apply_high_security_overrides();
                }
                _ => {
                    return Err(SecurityConfigError::EnvVarError {
                        var: "PROXEMIC_SECURITY_LEVEL".to_string(),
                        error: format!("Invalid security level: {}", level_str),
                    })
                }
            }
        }

        // File size override (with validation)
        if let Ok(size_str) = std::env::var("PROXEMIC_MAX_FILE_SIZE") {
            match size_str.parse::<u64>() {
                Ok(size) => {
                    if size > 1024 * 1024 * 1024 {
                        // 1GB hard limit
                        return Err(SecurityConfigError::EnvVarError {
                            var: "PROXEMIC_MAX_FILE_SIZE".to_string(),
                            error: "File size limit cannot exceed 1GB".to_string(),
                        });
                    }
                    self.max_file_size = size;
                }
                Err(_) => {
                    return Err(SecurityConfigError::EnvVarError {
                        var: "PROXEMIC_MAX_FILE_SIZE".to_string(),
                        error: "Invalid numeric value".to_string(),
                    })
                }
            }
        }

        // Path length override (with validation)
        if let Ok(path_len_str) = std::env::var("PROXEMIC_MAX_PATH_LENGTH") {
            match path_len_str.parse::<usize>() {
                Ok(length) => {
                    if length < 50 || length > 4096 {
                        // Reasonable bounds
                        return Err(SecurityConfigError::EnvVarError {
                            var: "PROXEMIC_MAX_PATH_LENGTH".to_string(),
                            error: "Path length must be between 50 and 4096 characters".to_string(),
                        });
                    }
                    self.max_path_length = length;
                }
                Err(_) => {
                    return Err(SecurityConfigError::EnvVarError {
                        var: "PROXEMIC_MAX_PATH_LENGTH".to_string(),
                        error: "Invalid numeric value".to_string(),
                    })
                }
            }
        }

        // Magic number validation override
        if let Ok(magic_str) = std::env::var("PROXEMIC_ENABLE_MAGIC_VALIDATION") {
            match magic_str.to_lowercase().as_str() {
                "true" | "1" | "yes" => self.enable_magic_number_validation = true,
                "false" | "0" | "no" => {
                    // Warn about disabling in production
                    if matches!(
                        self.security_level,
                        SecurityLevel::Production | SecurityLevel::HighSecurity
                    ) {
                        eprintln!(
                            "WARNING: Magic number validation disabled in production environment"
                        );
                    }
                    self.enable_magic_number_validation = false;
                }
                _ => {
                    return Err(SecurityConfigError::EnvVarError {
                        var: "PROXEMIC_ENABLE_MAGIC_VALIDATION".to_string(),
                        error: "Must be true/false, 1/0, or yes/no".to_string(),
                    })
                }
            }
        }

        // Concurrent operations override
        if let Ok(ops_str) = std::env::var("PROXEMIC_MAX_CONCURRENT_OPERATIONS") {
            match ops_str.parse::<u32>() {
                Ok(ops) => {
                    if ops > 1000 {
                        return Err(SecurityConfigError::EnvVarError {
                            var: "PROXEMIC_MAX_CONCURRENT_OPERATIONS".to_string(),
                            error: "Cannot exceed 1000 concurrent operations".to_string(),
                        });
                    }
                    self.max_concurrent_operations = ops;
                }
                Err(_) => {
                    return Err(SecurityConfigError::EnvVarError {
                        var: "PROXEMIC_MAX_CONCURRENT_OPERATIONS".to_string(),
                        error: "Invalid numeric value".to_string(),
                    })
                }
            }
        }

        // Workspace boundaries override
        if let Ok(boundaries_str) = std::env::var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES") {
            match boundaries_str.to_lowercase().as_str() {
                "true" | "1" | "yes" => self.enforce_workspace_boundaries = true,
                "false" | "0" | "no" => {
                    if matches!(
                        self.security_level,
                        SecurityLevel::Production | SecurityLevel::HighSecurity
                    ) {
                        eprintln!(
                            "WARNING: Workspace boundaries disabled in production environment"
                        );
                    }
                    self.enforce_workspace_boundaries = false;
                }
                _ => {
                    return Err(SecurityConfigError::EnvVarError {
                        var: "PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES".to_string(),
                        error: "Must be true/false, 1/0, or yes/no".to_string(),
                    })
                }
            }
        }

        // Audit logging override
        if let Ok(audit_str) = std::env::var("PROXEMIC_AUDIT_LOGGING_ENABLED") {
            match audit_str.to_lowercase().as_str() {
                "true" | "1" | "yes" => self.audit_logging_enabled = true,
                "false" | "0" | "no" => self.audit_logging_enabled = false,
                _ => {
                    return Err(SecurityConfigError::EnvVarError {
                        var: "PROXEMIC_AUDIT_LOGGING_ENABLED".to_string(),
                        error: "Must be true/false, 1/0, or yes/no".to_string(),
                    })
                }
            }
        }

        Ok(())
    }

    /// Apply production-specific security overrides
    fn apply_production_overrides(&mut self) {
        self.enable_magic_number_validation = true;
        self.enforce_workspace_boundaries = true;
        self.audit_logging_enabled = true;
        self.max_concurrent_operations = self.max_concurrent_operations.min(20);

        // Ensure production file size limits
        if self.max_file_size > 200 * 1024 * 1024 {
            // 200MB max for production
            self.max_file_size = 100 * 1024 * 1024; // Default to 100MB
        }
    }

    /// Apply high-security overrides
    fn apply_high_security_overrides(&mut self) {
        self.apply_production_overrides();
        self.max_file_size = self.max_file_size.min(50 * 1024 * 1024); // 50MB max
        self.max_path_length = self.max_path_length.min(200);
        self.max_concurrent_operations = 5;

        // Restrict to very limited file types in high security
        self.allowed_extensions = Self::high_security_allowed_extensions();
    }

    /// Validate configuration for security compliance
    pub fn validate(&self) -> Result<(), SecurityConfigError> {
        let mut errors = Vec::new();

        // Validate file size limits
        if self.max_file_size == 0 {
            errors.push("max_file_size cannot be zero".to_string());
        }
        if self.max_file_size > 2 * 1024 * 1024 * 1024 {
            // 2GB hard limit
            errors.push("max_file_size cannot exceed 2GB".to_string());
        }

        // Validate path length
        if self.max_path_length < 10 {
            errors.push("max_path_length too small (minimum 10)".to_string());
        }
        if self.max_path_length > 8192 {
            errors.push("max_path_length too large (maximum 8192)".to_string());
        }

        // Validate allowed extensions
        if self.allowed_extensions.is_empty() {
            errors.push("allowed_extensions cannot be empty".to_string());
        }

        // Production-specific validations
        if matches!(
            self.security_level,
            SecurityLevel::Production | SecurityLevel::HighSecurity
        ) {
            if !self.enable_magic_number_validation {
                errors.push("magic_number_validation must be enabled in production".to_string());
            }
            if !self.enforce_workspace_boundaries {
                errors.push("workspace_boundaries must be enforced in production".to_string());
            }
            if self.max_concurrent_operations > 100 {
                errors.push("max_concurrent_operations too high for production".to_string());
            }
        }

        // High-security specific validations
        if matches!(self.security_level, SecurityLevel::HighSecurity) {
            if self.max_file_size > 100 * 1024 * 1024 {
                errors.push("max_file_size too large for high security mode".to_string());
            }
            if self.max_concurrent_operations > 10 {
                errors
                    .push("max_concurrent_operations too high for high security mode".to_string());
            }
        }

        if !errors.is_empty() {
            return Err(SecurityConfigError::SchemaValidation { errors });
        }

        Ok(())
    }

    /// Production-hardened allowed extensions (minimal set)
    fn production_allowed_extensions() -> HashSet<String> {
        [
            ".txt", ".md", ".pdf", ".csv", ".json", ".docx", ".xlsx",
            ".pptx", // Common office formats
            ".png", ".jpg", ".jpeg", // Common image formats
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// High-security allowed extensions (very minimal set)
    fn high_security_allowed_extensions() -> HashSet<String> {
        [".txt", ".md", ".csv", ".json"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// Production-hardened MIME types
    fn production_allowed_mime_types() -> HashSet<String> {
        [
            "text/plain",
            "text/markdown",
            "application/pdf",
            "text/csv",
            "application/json",
            "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "image/png",
            "image/jpeg",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Strict prohibited characters for production
    fn strict_prohibited_chars() -> HashSet<char> {
        [
            '<', '>', '"', ':', '/', '\\', '|', '?', '*', '\0', '\x01', '\x02', '\x03', '\x04',
        ]
        .iter()
        .cloned()
        .collect()
    }

    /// Restricted workspace paths for production (only user directories)
    fn get_restricted_workspace_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join("Documents"));
            paths.push(home_dir.join("Downloads"));
            // Note: Desktop removed for production security
        }
        paths.push(std::env::temp_dir());
        paths
            .into_iter()
            .filter(|p| p.exists() && p.is_dir())
            .collect()
    }

    /// Get default workspace paths (legacy compatibility - maintains original behavior)
    fn get_default_workspace_paths() -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(home_dir) = dirs::home_dir() {
            paths.push(home_dir.join("Desktop"));
            paths.push(home_dir.join("Documents"));
            paths.push(home_dir.join("Downloads"));

            #[cfg(target_os = "windows")]
            {
                paths.push(home_dir.join("OneDrive"));
                paths.push(home_dir.join("OneDrive - Personal"));
            }

            #[cfg(target_os = "macos")]
            {
                paths.push(home_dir.join("iCloud Drive"));
            }
        }
        paths.push(std::env::temp_dir());
        paths
            .into_iter()
            .filter(|p| p.exists() && p.is_dir())
            .collect()
    }

    /// Create the magic number mapping for file type detection (enhanced from original)
    fn create_magic_number_map() -> HashMap<String, Vec<Vec<u8>>> {
        let mut magic_numbers = HashMap::new();

        // PDF files
        magic_numbers.insert("pdf".to_string(), vec![vec![0x25, 0x50, 0x44, 0x46]]);

        // Microsoft Office files (ZIP-based)
        magic_numbers.insert(
            "docx".to_string(),
            vec![
                vec![0x50, 0x4B, 0x03, 0x04],
                vec![0x50, 0x4B, 0x05, 0x06],
                vec![0x50, 0x4B, 0x07, 0x08],
            ],
        );

        // Text files (no reliable magic number)
        magic_numbers.insert("txt".to_string(), vec![]);
        magic_numbers.insert("md".to_string(), vec![]);
        magic_numbers.insert("csv".to_string(), vec![]);
        magic_numbers.insert("json".to_string(), vec![]);

        // PNG
        magic_numbers.insert(
            "png".to_string(),
            vec![vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]],
        );

        // JPEG (enhanced with more signatures)
        magic_numbers.insert("jpg".to_string(), vec![vec![0xFF, 0xD8, 0xFF]]);
        magic_numbers.insert("jpeg".to_string(), vec![vec![0xFF, 0xD8, 0xFF]]);

        magic_numbers
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        // Determine security level from environment or default to development
        let security_level = std::env::var("PROXEMIC_SECURITY_LEVEL")
            .unwrap_or_else(|_| "development".to_string())
            .to_lowercase();

        match security_level.as_str() {
            "production" => Self::production_defaults(),
            "high_security" => Self::high_security_defaults(),
            _ => {
                // Default to development mode using AppSecurityConfig for compatibility
                let app_config = AppSecurityConfig::default();
                Self::from_app_config(&app_config)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_production_defaults() {
        let config = SecurityConfig::production_defaults();
        assert_eq!(config.security_level, SecurityLevel::Production);
        assert!(config.enable_magic_number_validation);
        assert!(config.enforce_workspace_boundaries);
        assert!(config.audit_logging_enabled);
        assert_eq!(config.max_file_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_high_security_defaults() {
        let config = SecurityConfig::high_security_defaults();
        assert_eq!(config.security_level, SecurityLevel::HighSecurity);
        assert_eq!(config.max_file_size, 50 * 1024 * 1024);
        assert_eq!(config.max_concurrent_operations, 5);
    }

    #[test]
    fn test_legacy_compatibility() {
        let app_config = AppSecurityConfig::default();
        let config = SecurityConfig::from_app_config(&app_config);
        assert_eq!(config.security_level, SecurityLevel::Development);
        assert_eq!(config.max_concurrent_operations, 50);
        assert!(!config.audit_logging_enabled);
    }

    #[test]
    fn test_config_validation() {
        let mut config = SecurityConfig::production_defaults();
        assert!(config.validate().is_ok());

        // Test invalid configuration
        config.max_file_size = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_environment_overrides() {
        std::env::set_var("PROXEMIC_MAX_FILE_SIZE", "1048576"); // 1MB
        std::env::set_var("PROXEMIC_SECURITY_LEVEL", "production");

        let app_config = AppSecurityConfig::default();
        let config = SecurityConfig::from_app_config_with_env(&app_config);

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.max_file_size, 1048576);
        assert_eq!(config.security_level, SecurityLevel::Production);

        // Clean up
        std::env::remove_var("PROXEMIC_MAX_FILE_SIZE");
        std::env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }

    #[test]
    fn test_magic_number_mapping() {
        let config = SecurityConfig::default();
        assert!(config.magic_number_map.contains_key("pdf"));
        assert!(config.magic_number_map.contains_key("png"));
        assert!(config.magic_number_map.contains_key("jpg"));

        // Text files should have empty magic number arrays
        assert_eq!(config.magic_number_map.get("txt").unwrap().len(), 0);
        assert_eq!(config.magic_number_map.get("md").unwrap().len(), 0);
    }
}
