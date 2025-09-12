// src-tauri/src/filesystem/security/env_validator.rs
// Production environment configuration validator

use crate::filesystem::security::security_config::{SecurityConfigError, SecurityLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;

/// Environment configuration validator for production deployments
#[derive(Debug, Clone)]
pub struct EnvironmentValidator {
    required_vars: HashMap<SecurityLevel, Vec<String>>,
    validation_rules: HashMap<String, ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub var_type: EnvVarType,
    pub min_value: Option<u64>,
    pub max_value: Option<u64>,
    pub allowed_values: Option<Vec<String>>,
    pub required_in_production: bool,
    pub security_critical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvVarType {
    String,
    Number,
    Boolean,
    Path,
    Enum,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub security_level: SecurityLevel,
    pub missing_critical_vars: Vec<String>,
}

impl EnvironmentValidator {
    pub fn new() -> Self {
        let mut validator = Self {
            required_vars: HashMap::new(),
            validation_rules: HashMap::new(),
        };

        validator.initialize_rules();
        validator
    }

    /// Initialize validation rules for all Proxemic environment variables
    fn initialize_rules(&mut self) {
        // Security Level
        self.add_rule(
            "PROXEMIC_SECURITY_LEVEL",
            ValidationRule {
                var_type: EnvVarType::Enum,
                min_value: None,
                max_value: None,
                allowed_values: Some(vec![
                    "development".to_string(),
                    "production".to_string(),
                    "high_security".to_string(),
                ]),
                required_in_production: true,
                security_critical: true,
            },
        );

        // File Security Limits
        self.add_rule(
            "PROXEMIC_MAX_FILE_SIZE",
            ValidationRule {
                var_type: EnvVarType::Number,
                min_value: Some(1024),                   // 1KB minimum
                max_value: Some(2 * 1024 * 1024 * 1024), // 2GB maximum
                allowed_values: None,
                required_in_production: false,
                security_critical: true,
            },
        );

        self.add_rule(
            "PROXEMIC_MAX_PATH_LENGTH",
            ValidationRule {
                var_type: EnvVarType::Number,
                min_value: Some(50),
                max_value: Some(4096),
                allowed_values: None,
                required_in_production: false,
                security_critical: false,
            },
        );

        self.add_rule(
            "PROXEMIC_MAX_CONCURRENT_OPERATIONS",
            ValidationRule {
                var_type: EnvVarType::Number,
                min_value: Some(1),
                max_value: Some(1000),
                allowed_values: None,
                required_in_production: false,
                security_critical: false,
            },
        );

        // Critical Security Features
        self.add_rule(
            "PROXEMIC_ENABLE_MAGIC_VALIDATION",
            ValidationRule {
                var_type: EnvVarType::Boolean,
                min_value: None,
                max_value: None,
                allowed_values: Some(vec![
                    "true".to_string(),
                    "false".to_string(),
                    "1".to_string(),
                    "0".to_string(),
                    "yes".to_string(),
                    "no".to_string(),
                ]),
                required_in_production: true,
                security_critical: true,
            },
        );

        self.add_rule(
            "PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES",
            ValidationRule {
                var_type: EnvVarType::Boolean,
                min_value: None,
                max_value: None,
                allowed_values: Some(vec![
                    "true".to_string(),
                    "false".to_string(),
                    "1".to_string(),
                    "0".to_string(),
                    "yes".to_string(),
                    "no".to_string(),
                ]),
                required_in_production: true,
                security_critical: true,
            },
        );

        self.add_rule(
            "PROXEMIC_AUDIT_LOGGING_ENABLED",
            ValidationRule {
                var_type: EnvVarType::Boolean,
                min_value: None,
                max_value: None,
                allowed_values: Some(vec![
                    "true".to_string(),
                    "false".to_string(),
                    "1".to_string(),
                    "0".to_string(),
                    "yes".to_string(),
                    "no".to_string(),
                ]),
                required_in_production: true,
                security_critical: true,
            },
        );

        // Encryption and API Keys (existence check only)
        self.add_rule(
            "PROXEMIC_ENCRYPTION_KEY",
            ValidationRule {
                var_type: EnvVarType::String,
                min_value: Some(32), // Minimum 32 characters for AES-256
                max_value: Some(256),
                allowed_values: None,
                required_in_production: true,
                security_critical: true,
            },
        );

        // Database Configuration
        self.add_rule(
            "DATABASE_URL",
            ValidationRule {
                var_type: EnvVarType::String,
                min_value: Some(10),
                max_value: Some(500),
                allowed_values: None,
                required_in_production: false,
                security_critical: false,
            },
        );

        // Initialize required variables by security level
        self.required_vars.insert(
            SecurityLevel::Production,
            vec![
                "PROXEMIC_SECURITY_LEVEL".to_string(),
                "PROXEMIC_ENABLE_MAGIC_VALIDATION".to_string(),
                "PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES".to_string(),
                "PROXEMIC_AUDIT_LOGGING_ENABLED".to_string(),
                "PROXEMIC_ENCRYPTION_KEY".to_string(),
            ],
        );

        self.required_vars.insert(
            SecurityLevel::HighSecurity,
            vec![
                "PROXEMIC_SECURITY_LEVEL".to_string(),
                "PROXEMIC_ENABLE_MAGIC_VALIDATION".to_string(),
                "PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES".to_string(),
                "PROXEMIC_AUDIT_LOGGING_ENABLED".to_string(),
                "PROXEMIC_ENCRYPTION_KEY".to_string(),
                "PROXEMIC_MAX_FILE_SIZE".to_string(),
                "PROXEMIC_MAX_CONCURRENT_OPERATIONS".to_string(),
            ],
        );

        self.required_vars
            .insert(SecurityLevel::Development, vec![]);
    }

    fn add_rule(&mut self, var_name: &str, rule: ValidationRule) {
        self.validation_rules.insert(var_name.to_string(), rule);
    }

    /// Validate current environment configuration
    pub fn validate_environment(&self) -> Result<ValidationResult, SecurityConfigError> {
        let mut result = ValidationResult {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            security_level: SecurityLevel::Development,
            missing_critical_vars: Vec::new(),
        };

        // Determine security level first
        result.security_level = self.determine_security_level()?;

        // Get required variables for this security level
        let empty_vec = vec![];
        let required_vars = self
            .required_vars
            .get(&result.security_level)
            .unwrap_or(&empty_vec);

        // Check required variables
        for var_name in required_vars {
            if env::var(var_name).is_err() {
                result.errors.push(format!(
                    "Required environment variable '{}' is missing for security level '{:?}'",
                    var_name, result.security_level
                ));
                result.missing_critical_vars.push(var_name.clone());
            }
        }

        // Validate all present environment variables
        for (var_name, rule) in &self.validation_rules {
            if let Ok(value) = env::var(var_name) {
                if let Err(error) = self.validate_variable(var_name, &value, rule) {
                    if rule.security_critical {
                        result.errors.push(error);
                    } else {
                        result.warnings.push(error);
                    }
                }
            }
        }

        // Production-specific validations
        if matches!(
            result.security_level,
            SecurityLevel::Production | SecurityLevel::HighSecurity
        ) {
            self.validate_production_requirements(&mut result)?;
        }

        result.valid = result.errors.is_empty();
        Ok(result)
    }

    fn determine_security_level(&self) -> Result<SecurityLevel, SecurityConfigError> {
        match env::var("PROXEMIC_SECURITY_LEVEL") {
            Ok(level_str) => match level_str.to_lowercase().as_str() {
                "development" => Ok(SecurityLevel::Development),
                "production" => Ok(SecurityLevel::Production),
                "high_security" => Ok(SecurityLevel::HighSecurity),
                _ => Err(SecurityConfigError::EnvVarError {
                    var: "PROXEMIC_SECURITY_LEVEL".to_string(),
                    error: format!("Invalid security level: {}", level_str),
                }),
            },
            Err(_) => Ok(SecurityLevel::Development), // Default to development
        }
    }

    fn validate_variable(
        &self,
        var_name: &str,
        value: &str,
        rule: &ValidationRule,
    ) -> Result<(), String> {
        match rule.var_type {
            EnvVarType::String => {
                if let Some(min_len) = rule.min_value {
                    if value.len() < min_len as usize {
                        return Err(format!(
                            "Variable '{}' too short: {} < {} characters",
                            var_name,
                            value.len(),
                            min_len
                        ));
                    }
                }
                if let Some(max_len) = rule.max_value {
                    if value.len() > max_len as usize {
                        return Err(format!(
                            "Variable '{}' too long: {} > {} characters",
                            var_name,
                            value.len(),
                            max_len
                        ));
                    }
                }
            }
            EnvVarType::Number => match value.parse::<u64>() {
                Ok(num_value) => {
                    if let Some(min_val) = rule.min_value {
                        if num_value < min_val {
                            return Err(format!(
                                "Variable '{}' too small: {} < {}",
                                var_name, num_value, min_val
                            ));
                        }
                    }
                    if let Some(max_val) = rule.max_value {
                        if num_value > max_val {
                            return Err(format!(
                                "Variable '{}' too large: {} > {}",
                                var_name, num_value, max_val
                            ));
                        }
                    }
                }
                Err(_) => {
                    return Err(format!(
                        "Variable '{}' is not a valid number: '{}'",
                        var_name, value
                    ));
                }
            },
            EnvVarType::Boolean => {
                let normalized = value.to_lowercase();
                if !matches!(
                    normalized.as_str(),
                    "true" | "false" | "1" | "0" | "yes" | "no"
                ) {
                    return Err(format!(
                        "Variable '{}' is not a valid boolean: '{}' (expected: true/false, 1/0, yes/no)",
                        var_name, value
                    ));
                }
            }
            EnvVarType::Enum => {
                if let Some(ref allowed_values) = rule.allowed_values {
                    if !allowed_values.contains(&value.to_lowercase()) {
                        return Err(format!(
                            "Variable '{}' has invalid value: '{}' (allowed: {:?})",
                            var_name, value, allowed_values
                        ));
                    }
                }
            }
            EnvVarType::Path => {
                // Basic path validation
                if value.is_empty() {
                    return Err(format!("Variable '{}' cannot be empty path", var_name));
                }
            }
        }

        Ok(())
    }

    fn validate_production_requirements(
        &self,
        result: &mut ValidationResult,
    ) -> Result<(), SecurityConfigError> {
        // Ensure critical security features are enabled in production
        let critical_boolean_vars = [
            ("PROXEMIC_ENABLE_MAGIC_VALIDATION", true),
            ("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES", true),
            ("PROXEMIC_AUDIT_LOGGING_ENABLED", true),
        ];

        for (var_name, expected_value) in &critical_boolean_vars {
            if let Ok(value) = env::var(var_name) {
                let is_enabled = matches!(value.to_lowercase().as_str(), "true" | "1" | "yes");
                if is_enabled != *expected_value {
                    result.errors.push(format!(
                        "Security violation: '{}' must be '{}' in production mode",
                        var_name,
                        if *expected_value { "true" } else { "false" }
                    ));
                }
            }
        }

        // Check for insecure development settings in production
        if let Ok(debug_value) = env::var("PROXEMIC_DEBUG") {
            if matches!(debug_value.to_lowercase().as_str(), "true" | "1" | "yes") {
                result.warnings.push(
                    "Warning: PROXEMIC_DEBUG is enabled in production environment".to_string(),
                );
            }
        }

        // Validate encryption key strength
        if let Ok(encryption_key) = env::var("PROXEMIC_ENCRYPTION_KEY") {
            if encryption_key == "your_secure_32_character_key_here_change_this" {
                result.errors.push(
                    "Security violation: Default encryption key detected in production. Change PROXEMIC_ENCRYPTION_KEY immediately!".to_string()
                );
            }
            if encryption_key.len() < 32 {
                result.errors.push(
                    "Security violation: Encryption key too short for AES-256 (minimum 32 characters)".to_string()
                );
            }
        }

        Ok(())
    }

    /// Generate a configuration validation report
    #[allow(dead_code)]
    pub fn generate_validation_report(&self) -> Result<String, SecurityConfigError> {
        let result = self.validate_environment()?;

        let mut report = String::new();
        report.push_str("=== Proxemic Environment Configuration Report ===\n\n");

        report.push_str(&format!("Security Level: {:?}\n", result.security_level));
        report.push_str(&format!("Configuration Valid: {}\n\n", result.valid));

        if !result.errors.is_empty() {
            report.push_str("ERRORS (Must Fix):\n");
            for error in &result.errors {
                report.push_str(&format!("  ❌ {}\n", error));
            }
            report.push('\n');
        }

        if !result.warnings.is_empty() {
            report.push_str("WARNINGS (Should Review):\n");
            for warning in &result.warnings {
                report.push_str(&format!("  ⚠️  {}\n", warning));
            }
            report.push('\n');
        }

        if !result.missing_critical_vars.is_empty() {
            report.push_str("MISSING CRITICAL VARIABLES:\n");
            for var in &result.missing_critical_vars {
                report.push_str(&format!("  🔴 {}\n", var));
            }
            report.push('\n');
        }

        // Show current configuration
        report.push_str("CURRENT ENVIRONMENT VARIABLES:\n");
        for var_name in self.validation_rules.keys() {
            match env::var(var_name) {
                Ok(value) => {
                    // Mask sensitive values
                    let display_value = if var_name.contains("KEY") || var_name.contains("SECRET") {
                        if value.len() > 8 {
                            format!("{}...{}", &value[..4], &value[value.len() - 4..])
                        } else {
                            "[HIDDEN]".to_string()
                        }
                    } else {
                        value
                    };
                    report.push_str(&format!("  ✅ {} = {}\n", var_name, display_value));
                }
                Err(_) => {
                    report.push_str(&format!("  ❌ {} = [NOT SET]\n", var_name));
                }
            }
        }

        Ok(report)
    }

    /// Check if environment is ready for production deployment
    #[allow(dead_code)]
    pub fn is_production_ready(&self) -> Result<bool, SecurityConfigError> {
        let result = self.validate_environment()?;
        Ok(result.valid
            && matches!(
                result.security_level,
                SecurityLevel::Production | SecurityLevel::HighSecurity
            )
            && result.missing_critical_vars.is_empty())
    }

    /// Get security recommendations based on current configuration
    #[allow(dead_code)]
    pub fn get_security_recommendations(&self) -> Result<Vec<String>, SecurityConfigError> {
        let result = self.validate_environment()?;
        let mut recommendations = Vec::new();

        match result.security_level {
            SecurityLevel::Development => {
                recommendations.push(
                    "Consider upgrading to 'production' security level for deployment".to_string(),
                );
                recommendations
                    .push("Enable audit logging for better security monitoring".to_string());
            }
            SecurityLevel::Production => {
                recommendations
                    .push("Configuration suitable for production deployment".to_string());
                if env::var("PROXEMIC_MAX_FILE_SIZE").is_err() {
                    recommendations.push(
                        "Consider setting PROXEMIC_MAX_FILE_SIZE to limit resource usage"
                            .to_string(),
                    );
                }
            }
            SecurityLevel::HighSecurity => {
                recommendations
                    .push("High security mode active - maximum protection enabled".to_string());
                recommendations
                    .push("Monitor logs closely for any security violations".to_string());
            }
        }

        // API key recommendations
        if env::var("OPENROUTER_API_KEY").is_err() && env::var("ANTHROPIC_API_KEY").is_err() {
            recommendations.push(
                "Set at least one AI service API key (OPENROUTER_API_KEY or ANTHROPIC_API_KEY)"
                    .to_string(),
            );
        }

        // Performance recommendations
        if let Ok(ops_str) = env::var("PROXEMIC_MAX_CONCURRENT_OPERATIONS") {
            if let Ok(ops) = ops_str.parse::<u32>() {
                if ops > 50
                    && matches!(
                        result.security_level,
                        SecurityLevel::Production | SecurityLevel::HighSecurity
                    )
                {
                    recommendations.push("Consider reducing PROXEMIC_MAX_CONCURRENT_OPERATIONS for better resource management".to_string());
                }
            }
        }

        Ok(recommendations)
    }
}

impl Default for EnvironmentValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use serial_test::serial;

    #[test]
    fn test_environment_validator_creation() {
        let validator = EnvironmentValidator::new();
        assert!(!validator.validation_rules.is_empty());
    }

    #[test]
    #[serial]
    fn test_security_level_validation() {
        let validator = EnvironmentValidator::new();

        // Test valid security level
        env::set_var("PROXEMIC_SECURITY_LEVEL", "production");
        let result = validator.determine_security_level();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SecurityLevel::Production);

        // Test invalid security level
        env::set_var("PROXEMIC_SECURITY_LEVEL", "invalid");
        let result = validator.determine_security_level();
        assert!(result.is_err());

        env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }

    #[test]
    fn test_number_validation() {
        let validator = EnvironmentValidator::new();
        let rule = ValidationRule {
            var_type: EnvVarType::Number,
            min_value: Some(10),
            max_value: Some(100),
            allowed_values: None,
            required_in_production: false,
            security_critical: false,
        };

        // Valid number
        assert!(validator.validate_variable("TEST_VAR", "50", &rule).is_ok());

        // Too small
        assert!(validator.validate_variable("TEST_VAR", "5", &rule).is_err());

        // Too large
        assert!(validator
            .validate_variable("TEST_VAR", "150", &rule)
            .is_err());

        // Not a number
        assert!(validator
            .validate_variable("TEST_VAR", "not_a_number", &rule)
            .is_err());
    }

    #[test]
    fn test_boolean_validation() {
        let validator = EnvironmentValidator::new();
        let rule = ValidationRule {
            var_type: EnvVarType::Boolean,
            min_value: None,
            max_value: None,
            allowed_values: Some(vec![
                "true".to_string(),
                "false".to_string(),
                "1".to_string(),
                "0".to_string(),
                "yes".to_string(),
                "no".to_string(),
            ]),
            required_in_production: false,
            security_critical: false,
        };

        // Valid booleans
        assert!(validator
            .validate_variable("TEST_VAR", "true", &rule)
            .is_ok());
        assert!(validator
            .validate_variable("TEST_VAR", "false", &rule)
            .is_ok());
        assert!(validator.validate_variable("TEST_VAR", "1", &rule).is_ok());
        assert!(validator.validate_variable("TEST_VAR", "0", &rule).is_ok());
        assert!(validator
            .validate_variable("TEST_VAR", "yes", &rule)
            .is_ok());
        assert!(validator.validate_variable("TEST_VAR", "no", &rule).is_ok());

        // Invalid boolean
        assert!(validator
            .validate_variable("TEST_VAR", "maybe", &rule)
            .is_err());
    }

    #[test]
    #[serial]
    fn test_production_validation() {
        let validator = EnvironmentValidator::new();

        // Set production environment
        env::set_var("PROXEMIC_SECURITY_LEVEL", "production");
        env::set_var("PROXEMIC_ENABLE_MAGIC_VALIDATION", "false"); // Should trigger error
        env::set_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES", "true");
        env::set_var("PROXEMIC_AUDIT_LOGGING_ENABLED", "true");
        env::set_var(
            "PROXEMIC_ENCRYPTION_KEY",
            "test_key_that_is_long_enough_32_chars",
        );

        let result = validator.validate_environment().unwrap();

        // Should have errors due to disabled magic validation
        assert!(!result.valid);
        assert!(!result.errors.is_empty());

        // Clean up
        env::remove_var("PROXEMIC_SECURITY_LEVEL");
        env::remove_var("PROXEMIC_ENABLE_MAGIC_VALIDATION");
        env::remove_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES");
        env::remove_var("PROXEMIC_AUDIT_LOGGING_ENABLED");
        env::remove_var("PROXEMIC_ENCRYPTION_KEY");
    }

    #[test]
    #[serial]
    fn test_encryption_key_validation() {
        let validator = EnvironmentValidator::new();

        env::set_var("PROXEMIC_SECURITY_LEVEL", "production");
        env::set_var(
            "PROXEMIC_ENCRYPTION_KEY",
            "your_secure_32_character_key_here_change_this",
        );
        env::set_var("PROXEMIC_ENABLE_MAGIC_VALIDATION", "true");
        env::set_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES", "true");
        env::set_var("PROXEMIC_AUDIT_LOGGING_ENABLED", "true");

        let result = validator.validate_environment().unwrap();

        // Should detect default encryption key
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.contains("Default encryption key detected")));

        // Clean up
        env::remove_var("PROXEMIC_SECURITY_LEVEL");
        env::remove_var("PROXEMIC_ENCRYPTION_KEY");
        env::remove_var("PROXEMIC_ENABLE_MAGIC_VALIDATION");
        env::remove_var("PROXEMIC_ENFORCE_WORKSPACE_BOUNDARIES");
        env::remove_var("PROXEMIC_AUDIT_LOGGING_ENABLED");
    }
}
