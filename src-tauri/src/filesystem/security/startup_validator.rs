// src-tauri/src/filesystem/security/startup_validator.rs
// Application startup configuration validation

use crate::filesystem::security::env_validator::EnvironmentValidator;
use crate::filesystem::security::{
    DeploymentChecker, SecurityConfig, SecurityConfigError, SecurityLevel,
};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info, warn};

/// Startup validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupValidationResult {
    pub success: bool,
    pub security_level: SecurityLevel,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub config_valid: bool,
    pub environment_ready: bool,
    pub production_ready: bool,
}

/// Startup configuration validator - runs before main application initialization
pub struct StartupValidator {
    env_validator: EnvironmentValidator,
    deployment_checker: DeploymentChecker,
}

impl StartupValidator {
    pub fn new() -> Self {
        Self {
            env_validator: EnvironmentValidator::new(),
            deployment_checker: DeploymentChecker::new(),
        }
    }

    /// Validate configuration at application startup
    /// This should be called early in the application lifecycle
    pub fn validate_startup_configuration(
        &self,
    ) -> Result<StartupValidationResult, SecurityConfigError> {
        info!("Starting Proxemic configuration validation...");

        let mut result = StartupValidationResult {
            success: false,
            security_level: SecurityLevel::Development,
            warnings: Vec::new(),
            errors: Vec::new(),
            config_valid: false,
            environment_ready: false,
            production_ready: false,
        };

        // 1. Validate environment variables
        match self.env_validator.validate_environment() {
            Ok(env_result) => {
                result.environment_ready = env_result.valid;
                result.security_level = env_result.security_level;
                result.errors.extend(env_result.errors);
                result.warnings.extend(env_result.warnings);

                if !result.environment_ready {
                    error!("Environment configuration validation failed");
                } else {
                    info!("Environment configuration validation passed");
                }
            }
            Err(e) => {
                error!("Failed to validate environment: {}", e);
                result
                    .errors
                    .push(format!("Environment validation error: {}", e));
            }
        }

        // 2. Validate security configuration
        match SecurityConfig::default().validate() {
            Ok(_) => {
                result.config_valid = true;
                info!("Security configuration validation passed");
            }
            Err(e) => {
                error!("Security configuration validation failed: {}", e);
                result.config_valid = false;
                result
                    .errors
                    .push(format!("Security configuration error: {}", e));
            }
        }

        // 3. Check deployment readiness if in production mode
        if matches!(
            result.security_level,
            SecurityLevel::Production | SecurityLevel::HighSecurity
        ) {
            match self.deployment_checker.assess_deployment_readiness() {
                Ok(assessment) => {
                    result.production_ready = assessment.ready_for_production;
                    result.errors.extend(assessment.critical_issues);
                    result.warnings.extend(assessment.warnings);

                    if result.production_ready {
                        info!("Production deployment readiness check passed");
                    } else {
                        warn!("Production deployment readiness check failed");
                    }
                }
                Err(e) => {
                    error!("Failed to assess deployment readiness: {}", e);
                    result
                        .errors
                        .push(format!("Deployment assessment error: {}", e));
                }
            }
        } else {
            // Development mode - skip production checks but note it
            result.production_ready = false;
            result
                .warnings
                .push("Running in development mode - not suitable for production".to_string());
        }

        // 4. Perform critical security checks
        self.perform_critical_security_checks(&mut result)?;

        // 5. Check for common misconfigurations
        self.check_common_misconfigurations(&mut result);

        // 6. Determine overall success
        result.success =
            result.errors.is_empty() && result.config_valid && result.environment_ready;

        // 7. Log final status
        if result.success {
            info!("✅ Proxemic startup configuration validation completed successfully");
            info!("Security Level: {:?}", result.security_level);
        } else {
            error!("❌ Proxemic startup configuration validation failed");
            for error in &result.errors {
                error!("  - {}", error);
            }
        }

        // 8. Log warnings
        for warning in &result.warnings {
            warn!("  ⚠️  {}", warning);
        }

        Ok(result)
    }

    /// Perform critical security checks that could prevent startup
    fn perform_critical_security_checks(
        &self,
        result: &mut StartupValidationResult,
    ) -> Result<(), SecurityConfigError> {
        // Check for critical security violations that should prevent startup
        if matches!(
            result.security_level,
            SecurityLevel::Production | SecurityLevel::HighSecurity
        ) {
            // 1. Check for default encryption key
            if let Ok(key) = env::var("PROXEMIC_ENCRYPTION_KEY") {
                if key == "your_secure_32_character_key_here_change_this" {
                    result.errors.push("CRITICAL: Default encryption key detected in production! Application startup blocked for security.".to_string());
                }
            }

            // 2. Check for disabled security features in production
            let critical_security_settings = [
                (
                    "PROXEMIC_ENABLE_MAGIC_VALIDATION",
                    "Magic number validation",
                ),
                (
                    "PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES",
                    "Workspace boundary enforcement",
                ),
                ("PROXEMIC_AUDIT_LOGGING_ENABLED", "Audit logging"),
            ];

            for (env_var, feature_name) in &critical_security_settings {
                if let Ok(value) = env::var(env_var) {
                    let is_enabled = matches!(value.to_lowercase().as_str(), "true" | "1" | "yes");
                    if !is_enabled {
                        result.errors.push(format!(
                            "CRITICAL: {} disabled in production mode - this is a security violation",
                            feature_name
                        ));
                    }
                }
            }

            // 3. Check for debug mode in production
            if let Ok(debug) = env::var("PROXEMIC_DEBUG") {
                if matches!(debug.to_lowercase().as_str(), "true" | "1" | "yes") {
                    result.errors.push(
                        "CRITICAL: Debug mode enabled in production - security risk".to_string(),
                    );
                }
            }
        }

        Ok(())
    }

    /// Check for common configuration mistakes
    fn check_common_misconfigurations(&self, result: &mut StartupValidationResult) {
        // Check for missing API keys
        let has_openrouter = env::var("OPENROUTER_API_KEY").is_ok();
        let has_anthropic = env::var("ANTHROPIC_API_KEY").is_ok();

        if !has_openrouter && !has_anthropic {
            result.warnings.push(
                "No AI service API keys configured - AI features will be unavailable".to_string(),
            );
        }

        // Check for overly permissive settings
        if let Ok(file_size_str) = env::var("PROXEMIC_MAX_FILE_SIZE") {
            if let Ok(file_size) = file_size_str.parse::<u64>() {
                if file_size > 1024 * 1024 * 1024 {
                    // 1GB
                    result.warnings.push(
                        "Very large file size limit configured - may impact performance"
                            .to_string(),
                    );
                }
            }
        }

        // Check for missing database configuration
        if env::var("DATABASE_URL").is_err() {
            result
                .warnings
                .push("Database URL not configured - using default SQLite database".to_string());
        }

        // Check log level configuration
        match env::var("RUST_LOG") {
            Ok(level) => {
                let is_debug_trace = level.to_lowercase().contains("trace")
                    || level.to_lowercase().contains("debug");
                let is_production_mode = matches!(
                    result.security_level,
                    SecurityLevel::Production | SecurityLevel::HighSecurity
                );

                if is_debug_trace && is_production_mode {
                    result.warnings.push(
                        "Debug/trace logging enabled in production - may impact performance"
                            .to_string(),
                    );
                }
            }
            Err(_) => {
                result
                    .warnings
                    .push("RUST_LOG not configured - using default logging level".to_string());
            }
        }
    }
    /// Legacy compatibility method for backward compatibility
    /// This bridges the old validate_startup_environment naming
    pub fn validate_startup_environment(
        &self,
    ) -> Result<StartupValidationResult, SecurityConfigError> {
        self.validate_startup_configuration()
    }
}

impl Default for StartupValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_startup_validator_creation() {
        let validator = StartupValidator::new();
        // Basic smoke test - ensure validator can be created
        assert!(matches!(
            validator.env_validator,
            EnvironmentValidator { .. }
        ));
    }

    #[test]
    fn test_development_mode_startup() {
        // Set up clean test environment - clear ALL environment variables that could affect detection
        let env_vars_to_clear = [
            "PROXEMIC_ENV",
            "RUST_ENV",
            "NODE_ENV",
            "PRODUCTION",
            "PROD",
            "PROXEMIC_SECURITY_LEVEL",
            "CI",
            "GITHUB_ACTIONS",
            "RUST_LOG",
        ];

        for var in &env_vars_to_clear {
            env::remove_var(var);
        }

        // Explicitly set development mode with all necessary variables
        env::set_var("PROXEMIC_ENV", "development");
        env::set_var("PROXEMIC_SECURITY_LEVEL", "development");
        env::set_var("RUST_ENV", "test"); // Explicitly set to test

        // Set non-debug log to avoid environment detection confusion
        env::set_var("RUST_LOG", "info");

        // Force debug mode to ensure development environment
        env::set_var("PROXEMIC_DEBUG", "true");

        let validator = StartupValidator::new();
        let result = validator.validate_startup_configuration().unwrap();

        // Debug information if assertion fails
        if result.security_level != SecurityLevel::Development {
            eprintln!("Expected Development, got {:?}", result.security_level);
            eprintln!("Environment variables:");
            for var in &env_vars_to_clear {
                eprintln!("  {}: {:?}", var, env::var(var));
            }
            eprintln!("Errors: {:?}", result.errors);
            eprintln!("Warnings: {:?}", result.warnings);
        }

        assert_eq!(result.security_level, SecurityLevel::Development);
        assert!(!result.production_ready); // Should not be production ready in dev mode

        // Clean up all variables we set
        for var in &env_vars_to_clear {
            env::remove_var(var);
        }
        env::remove_var("PROXEMIC_DEBUG");
    }

    #[test]
    fn test_critical_security_check() {
        // Clear ALL environment variables that could interfere with security level detection
        let env_vars_to_clear = [
            "PROXEMIC_ENV",
            "RUST_ENV",
            "NODE_ENV",
            "PRODUCTION",
            "PROD",
            "CI",
            "GITHUB_ACTIONS",
            "RUST_LOG",
            "PROXEMIC_DEBUG",
        ];

        for var in &env_vars_to_clear {
            env::remove_var(var);
        }

        // Set production mode with default encryption key (should fail)
        env::set_var("PROXEMIC_SECURITY_LEVEL", "production");
        env::set_var(
            "PROXEMIC_ENCRYPTION_KEY",
            "your_secure_32_character_key_here_change_this",
        );
        env::set_var("PROXEMIC_ENABLE_MAGIC_VALIDATION", "true");
        env::set_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES", "true");
        env::set_var("PROXEMIC_AUDIT_LOGGING_ENABLED", "true");

        let validator = StartupValidator::new();
        let result = validator.validate_startup_configuration().unwrap();

        // Debug output to understand what's happening
        println!("Security level: {:?}", result.security_level);
        println!("Success: {}", result.success);
        println!("Errors: {:?}", result.errors);
        println!("Warnings: {:?}", result.warnings);

        // Should fail due to default encryption key
        assert!(
            !result.success,
            "Expected validation to fail due to default encryption key"
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("encryption key") || e.contains("Encryption key")),
            "Expected error about encryption key, but got: {:?}",
            result.errors
        );

        // Clean up
        env::remove_var("PROXEMIC_SECURITY_LEVEL");
        env::remove_var("PROXEMIC_ENCRYPTION_KEY");
        env::remove_var("PROXEMIC_ENABLE_MAGIC_VALIDATION");
        env::remove_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES");
        env::remove_var("PROXEMIC_AUDIT_LOGGING_ENABLED");

        // Restore any cleared variables if needed for other tests
    }

    #[test]
    fn test_safe_startup_check() {
        // Set up a minimal valid configuration
        env::set_var("PROXEMIC_SECURITY_LEVEL", "development");

        let validator = StartupValidator::new();
        let result = validator.validate_startup_configuration().unwrap();

        // Development mode with minimal config should be safe
        assert!(result.success);
        assert!(result.config_valid);
        assert!(result.environment_ready);

        env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }
}
