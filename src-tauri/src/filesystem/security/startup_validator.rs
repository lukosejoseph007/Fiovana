// src-tauri/src/filesystem/security/startup_validator.rs
// Application startup configuration validation

use crate::filesystem::security::{
    DeploymentChecker, EnvironmentValidator, SecurityConfig, SecurityConfigError, SecurityLevel,
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
                if level.to_lowercase().contains("trace") || level.to_lowercase().contains("debug")
                {
                    if matches!(
                        result.security_level,
                        SecurityLevel::Production | SecurityLevel::HighSecurity
                    ) {
                        result.warnings.push(
                            "Debug/trace logging enabled in production - may impact performance"
                                .to_string(),
                        );
                    }
                }
            }
            Err(_) => {
                result
                    .warnings
                    .push("RUST_LOG not configured - using default logging level".to_string());
            }
        }
    }

    /// Generate a startup validation report for logging/debugging
    pub fn generate_startup_report(&self) -> Result<String, SecurityConfigError> {
        let result = self.validate_startup_configuration()?;

        let mut report = String::new();
        report.push_str("═══════════════════════════════════════════════════════════════\n");
        report.push_str("                PROXEMIC STARTUP VALIDATION REPORT\n");
        report.push_str("═══════════════════════════════════════════════════════════════\n\n");

        // Status overview
        let status_icon = if result.success { "✅" } else { "❌" };
        report.push_str(&format!(
            "Startup Validation: {} {}\n",
            status_icon,
            if result.success { "PASSED" } else { "FAILED" }
        ));
        report.push_str(&format!("Security Level: {:?}\n", result.security_level));
        report.push_str(&format!(
            "Environment Ready: {}\n",
            if result.environment_ready {
                "✅"
            } else {
                "❌"
            }
        ));
        report.push_str(&format!(
            "Configuration Valid: {}\n",
            if result.config_valid { "✅" } else { "❌" }
        ));

        if matches!(
            result.security_level,
            SecurityLevel::Production | SecurityLevel::HighSecurity
        ) {
            report.push_str(&format!(
                "Production Ready: {}\n",
                if result.production_ready {
                    "✅"
                } else {
                    "❌"
                }
            ));
        }
        report.push('\n');

        // Errors
        if !result.errors.is_empty() {
            report.push_str("🔴 CRITICAL ERRORS:\n");
            for (i, error) in result.errors.iter().enumerate() {
                report.push_str(&format!("  {}. {}\n", i + 1, error));
            }
            report.push('\n');
        }

        // Warnings
        if !result.warnings.is_empty() {
            report.push_str("⚠️  WARNINGS:\n");
            for warning in &result.warnings {
                report.push_str(&format!("  • {}\n", warning));
            }
            report.push('\n');
        }

        // Recommendations based on result
        report.push_str("💡 STARTUP RECOMMENDATIONS:\n");
        if !result.success {
            report.push_str("  1. Address all critical errors before proceeding\n");
            report.push_str("  2. Review environment configuration\n");
            report.push_str("  3. Restart application after fixes\n");
        } else {
            report.push_str("  1. Application ready to start\n");
            if !result.warnings.is_empty() {
                report.push_str("  2. Consider addressing warnings for optimal operation\n");
            }
            if !result.production_ready
                && matches!(
                    result.security_level,
                    SecurityLevel::Production | SecurityLevel::HighSecurity
                )
            {
                report.push_str("  3. Complete production readiness checklist\n");
            }
        }

        report.push_str("\n═══════════════════════════════════════════════════════════════\n");

        Ok(report)
    }

    /// Check if it's safe to start the application
    pub fn is_safe_to_start(&self) -> Result<bool, SecurityConfigError> {
        let result = self.validate_startup_configuration()?;

        // Allow startup if basic validation passes, even with warnings
        // But block if there are critical security errors
        Ok(result.success && result.config_valid && result.environment_ready)
    }

    /// Get startup validation result for use by main application
    pub fn get_validation_result(&self) -> Result<StartupValidationResult, SecurityConfigError> {
        self.validate_startup_configuration()
    }
}

impl Default for StartupValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to run startup validation
/// Should be called early in main() or in Tauri's setup hook
pub fn validate_startup() -> Result<StartupValidationResult, SecurityConfigError> {
    let validator = StartupValidator::new();
    validator.validate_startup_configuration()
}

/// Convenience function to check if startup is safe
pub fn is_startup_safe() -> Result<bool, SecurityConfigError> {
    let validator = StartupValidator::new();
    validator.is_safe_to_start()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_startup_validator_creation() {
        let validator = StartupValidator::new();
        // Basic smoke test
        assert!(true);
    }

    #[test]
    fn test_development_mode_startup() {
        // Ensure we're in development mode
        env::set_var("PROXEMIC_SECURITY_LEVEL", "development");

        let validator = StartupValidator::new();
        let result = validator.validate_startup_configuration().unwrap();

        assert_eq!(result.security_level, SecurityLevel::Development);
        assert!(!result.production_ready); // Should not be production ready in dev mode

        env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }

    #[test]
    fn test_critical_security_check() {
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

        // Should fail due to default encryption key
        assert!(!result.success);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Default encryption key")));

        // Clean up
        env::remove_var("PROXEMIC_SECURITY_LEVEL");
        env::remove_var("PROXEMIC_ENCRYPTION_KEY");
        env::remove_var("PROXEMIC_ENABLE_MAGIC_VALIDATION");
        env::remove_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES");
        env::remove_var("PROXEMIC_AUDIT_LOGGING_ENABLED");
    }

    #[test]
    fn test_safe_startup_check() {
        // Set up a minimal valid configuration
        env::set_var("PROXEMIC_SECURITY_LEVEL", "development");

        let validator = StartupValidator::new();
        let is_safe = validator.is_safe_to_start().unwrap();

        // Development mode with minimal config should be safe
        assert!(is_safe);

        env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }
}
