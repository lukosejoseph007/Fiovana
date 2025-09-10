// src-tauri/src/filesystem/security/mod.rs
// Security module definition with production-safe architecture

pub mod access_control;
pub mod audit_logger;
pub mod config_validator;
pub mod deployment_checker;
pub mod env_validator;
pub mod file_guard;
pub mod json_schema_validator;
pub mod log_rotation;
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
pub use deployment_checker::DeploymentChecker;
pub use security_config::{SecurityConfig, SecurityConfigError, SecurityLevel};
pub use startup_validator::StartupValidationResult;

/// Initialize the security system and perform startup validation
/// This should be called early in the application lifecycle
pub fn initialize_security_system() -> Result<StartupValidationResult, SecurityConfigError> {
    use startup_validator::StartupValidator;

    let validator = StartupValidator::new();
    validator.validate_startup_configuration()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_system_initialization() {
        let result = initialize_security_system();
        assert!(result.is_ok());
    }
}
