// src-tauri/src/filesystem/security/mod.rs
// Security module definition with production-safe architecture

pub mod access_control;
pub mod audit_logger;
pub mod config_validator;
pub mod deployment_checker;
pub mod env_validator;
pub mod file_guard;
pub mod magic_number_validator;
pub mod path_validator;
pub mod permissions;
pub mod permissions_escalation;
pub mod scope;
pub mod scope_restrictions;
pub mod scope_validator;
pub mod security_config;
pub mod startup_validator;

// Re-export commonly used types for convenience
pub use config_validator::{ConfigSchemaValidator, ValidationError as ConfigValidationError};
pub use deployment_checker::DeploymentChecker;
pub use path_validator::PathValidator;
pub use security_config::{SecurityConfig, SecurityConfigError, SecurityLevel};
pub use startup_validator::{initialize_security_system, StartupValidationResult};

// Security constants
pub const DEFAULT_MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
pub const PRODUCTION_MAX_FILE_SIZE: u64 = 100 * 1024 * 1024; // 100MB
pub const HIGH_SECURITY_MAX_FILE_SIZE: u64 = 50 * 1024 * 1024; // 50MB
pub const DEFAULT_MAX_PATH_LENGTH: usize = 260; // Windows MAX_PATH
pub const DEFAULT_MAX_CONCURRENT_OPS: u32 = 10;

/// Production security validation helper
pub fn validate_production_config(config: &SecurityConfig) -> Result<(), SecurityConfigError> {
    config.validate()
}

/// Environment-aware security configuration loader
pub fn load_security_config() -> Result<SecurityConfig, SecurityConfigError> {
    let mut config = SecurityConfig::default();
    config.apply_environment_overrides()?;
    config.validate()?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_security_config() {
        let config = load_security_config();
        assert!(config.is_ok());
    }

    #[test]
    fn test_production_constants() {
        assert_eq!(PRODUCTION_MAX_FILE_SIZE, 100 * 1024 * 1024);
        assert_eq!(HIGH_SECURITY_MAX_FILE_SIZE, 50 * 1024 * 1024);
        assert_eq!(DEFAULT_MAX_PATH_LENGTH, 260);
    }

    #[test]
    fn test_security_system_initialization() {
        let result = initialize_security_system();
        assert!(result.is_ok());
    }
}
