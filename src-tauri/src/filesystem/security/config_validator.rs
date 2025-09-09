// src-tauri/src/filesystem/security/config_validator.rs
// Schema validation for security configuration

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Field validation failed: {field} - {message}")]
    FieldValidation { field: String, message: String },
    #[error("Schema validation failed: {errors:?}")]
    SchemaValidation { errors: Vec<String> },
    #[error("Security policy violation: {message}")]
    SecurityViolation { message: String },
}

/// Configuration schema definition
#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigSchema {
    pub version: String,
    pub fields: Vec<FieldSchema>,
    pub security_constraints: SecurityConstraints,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldSchema {
    pub name: String,
    pub field_type: FieldType,
    pub required: bool,
    pub constraints: Option<FieldConstraints>,
    pub security_level_requirements: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    Array,
    Object,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FieldConstraints {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub min_value: Option<u64>,
    pub max_value: Option<u64>,
    pub allowed_values: Option<Vec<String>>,
    pub pattern: Option<String>,
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityConstraints {
    pub production_required_fields: Vec<String>,
    pub high_security_required_fields: Vec<String>,
    pub forbidden_combinations: Vec<ForbiddenCombination>,
    pub mandatory_overrides: Vec<MandatoryOverride>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForbiddenCombination {
    pub condition: String,
    pub forbidden_field: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MandatoryOverride {
    pub security_level: String,
    pub field: String,
    pub required_value: serde_json::Value,
}

/// Configuration schema validator
pub struct ConfigSchemaValidator {
    schema: ConfigSchema,
}

impl ConfigSchemaValidator {
    pub fn new() -> Self {
        Self {
            schema: Self::create_default_schema(),
        }
    }

    pub fn from_schema(schema: ConfigSchema) -> Self {
        Self { schema }
    }

    /// Validate a configuration against the schema
    pub fn validate_config(
        &self,
        config: &serde_json::Value,
        security_level: &str,
    ) -> Result<(), ValidationError> {
        let mut errors = Vec::new();

        // Validate each field according to schema
        for field_schema in &self.schema.fields {
            if let Err(e) = self.validate_field(config, field_schema, security_level) {
                match e {
                    ValidationError::FieldValidation { field, message } => {
                        errors.push(format!("{}: {}", field, message));
                    }
                    ValidationError::SchemaValidation {
                        errors: field_errors,
                    } => {
                        errors.extend(field_errors);
                    }
                    _ => errors.push(e.to_string()),
                }
            }
        }

        // Validate security constraints
        if let Err(e) = self.validate_security_constraints(config, security_level) {
            match e {
                ValidationError::SecurityViolation { message } => {
                    errors.push(format!("Security violation: {}", message));
                }
                ValidationError::SchemaValidation {
                    errors: security_errors,
                } => {
                    errors.extend(security_errors);
                }
                _ => errors.push(e.to_string()),
            }
        }

        if !errors.is_empty() {
            return Err(ValidationError::SchemaValidation { errors });
        }

        Ok(())
    }

    fn validate_field(
        &self,
        config: &serde_json::Value,
        field_schema: &FieldSchema,
        security_level: &str,
    ) -> Result<(), ValidationError> {
        let field_value = config.get(&field_schema.name);

        // Check if required field is present
        if field_schema.required && field_value.is_none() {
            return Err(ValidationError::FieldValidation {
                field: field_schema.name.clone(),
                message: "Required field is missing".to_string(),
            });
        }

        // Check security level requirements
        if let Some(ref required_levels) = field_schema.security_level_requirements {
            if required_levels.contains(&security_level.to_string()) && field_value.is_none() {
                return Err(ValidationError::FieldValidation {
                    field: field_schema.name.clone(),
                    message: format!("Field required for security level: {}", security_level),
                });
            }
        }

        if let Some(value) = field_value {
            self.validate_field_value(value, field_schema)?;
        }

        Ok(())
    }

    fn validate_field_value(
        &self,
        value: &serde_json::Value,
        field_schema: &FieldSchema,
    ) -> Result<(), ValidationError> {
        // Type validation
        match (&field_schema.field_type, value) {
            (FieldType::String, serde_json::Value::String(s)) => {
                if let Some(ref constraints) = field_schema.constraints {
                    self.validate_string_constraints(s, constraints, &field_schema.name)?;
                }
            }
            (FieldType::Number, serde_json::Value::Number(n)) => {
                if let Some(ref constraints) = field_schema.constraints {
                    if let Some(n_u64) = n.as_u64() {
                        self.validate_number_constraints(n_u64, constraints, &field_schema.name)?;
                    }
                }
            }
            (FieldType::Boolean, serde_json::Value::Bool(_)) => {
                // Boolean values are inherently valid
            }
            (FieldType::Array, serde_json::Value::Array(arr)) => {
                if let Some(ref constraints) = field_schema.constraints {
                    if let Some(max_len) = constraints.max_length {
                        if arr.len() > max_len {
                            return Err(ValidationError::FieldValidation {
                                field: field_schema.name.clone(),
                                message: format!("Array too long: {} > {}", arr.len(), max_len),
                            });
                        }
                    }
                }
            }
            _ => {
                return Err(ValidationError::FieldValidation {
                    field: field_schema.name.clone(),
                    message: format!("Type mismatch: expected {:?}", field_schema.field_type),
                });
            }
        }

        Ok(())
    }

    fn validate_string_constraints(
        &self,
        value: &str,
        constraints: &FieldConstraints,
        field_name: &str,
    ) -> Result<(), ValidationError> {
        if let Some(min_len) = constraints.min_length {
            if value.len() < min_len {
                return Err(ValidationError::FieldValidation {
                    field: field_name.to_string(),
                    message: format!("String too short: {} < {}", value.len(), min_len),
                });
            }
        }

        if let Some(max_len) = constraints.max_length {
            if value.len() > max_len {
                return Err(ValidationError::FieldValidation {
                    field: field_name.to_string(),
                    message: format!("String too long: {} > {}", value.len(), max_len),
                });
            }
        }

        if let Some(ref allowed_values) = constraints.allowed_values {
            if !allowed_values.contains(&value.to_string()) {
                return Err(ValidationError::FieldValidation {
                    field: field_name.to_string(),
                    message: format!("Invalid value: '{}' not in allowed values", value),
                });
            }
        }

        Ok(())
    }

    fn validate_number_constraints(
        &self,
        value: u64,
        constraints: &FieldConstraints,
        field_name: &str,
    ) -> Result<(), ValidationError> {
        if let Some(min_val) = constraints.min_value {
            if value < min_val {
                return Err(ValidationError::FieldValidation {
                    field: field_name.to_string(),
                    message: format!("Value too small: {} < {}", value, min_val),
                });
            }
        }

        if let Some(max_val) = constraints.max_value {
            if value > max_val {
                return Err(ValidationError::FieldValidation {
                    field: field_name.to_string(),
                    message: format!("Value too large: {} > {}", value, max_val),
                });
            }
        }

        Ok(())
    }

    fn validate_security_constraints(
        &self,
        config: &serde_json::Value,
        security_level: &str,
    ) -> Result<(), ValidationError> {
        let constraints = &self.schema.security_constraints;

        // Check required fields for security level
        let required_fields = match security_level {
            "production" => &constraints.production_required_fields,
            "high_security" => &constraints.high_security_required_fields,
            _ => &Vec::new(),
        };

        for required_field in required_fields {
            if config.get(required_field).is_none() {
                return Err(ValidationError::SecurityViolation {
                    message: format!(
                        "Required field '{}' missing for security level '{}'",
                        required_field, security_level
                    ),
                });
            }
        }

        // Check forbidden combinations
        for forbidden in &constraints.forbidden_combinations {
            if self.evaluate_condition(config, &forbidden.condition) {
                if config.get(&forbidden.forbidden_field).is_some() {
                    return Err(ValidationError::SecurityViolation {
                        message: forbidden.message.clone(),
                    });
                }
            }
        }

        // Check mandatory overrides
        for override_rule in &constraints.mandatory_overrides {
            if override_rule.security_level == security_level {
                if let Some(actual_value) = config.get(&override_rule.field) {
                    if actual_value != &override_rule.required_value {
                        return Err(ValidationError::SecurityViolation {
                            message: format!(
                                "Field '{}' must be set to {:?} for security level '{}'",
                                override_rule.field, override_rule.required_value, security_level
                            ),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn evaluate_condition(&self, config: &serde_json::Value, condition: &str) -> bool {
        // Simple condition evaluation (can be extended with a proper parser)
        // Format: "field_name=value" or "field_name!=value"
        if let Some((field, expected_value)) = condition.split_once('=') {
            if let Some(field_value) = config.get(field.trim()) {
                let expected = expected_value.trim().trim_matches('"');
                match field_value {
                    serde_json::Value::String(s) => s == expected,
                    serde_json::Value::Bool(b) => {
                        expected.parse::<bool>().map_or(false, |exp| *b == exp)
                    }
                    serde_json::Value::Number(n) => expected
                        .parse::<f64>()
                        .map_or(false, |exp| n.as_f64() == Some(exp)),
                    _ => false,
                }
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Create the default security configuration schema
    fn create_default_schema() -> ConfigSchema {
        ConfigSchema {
            version: "1.0.0".to_string(),
            fields: vec![
                FieldSchema {
                    name: "max_file_size".to_string(),
                    field_type: FieldType::Number,
                    required: true,
                    constraints: Some(FieldConstraints {
                        min_value: Some(1024),                   // 1KB minimum
                        max_value: Some(2 * 1024 * 1024 * 1024), // 2GB maximum
                        ..Default::default()
                    }),
                    security_level_requirements: Some(vec![
                        "production".to_string(),
                        "high_security".to_string(),
                    ]),
                },
                FieldSchema {
                    name: "max_path_length".to_string(),
                    field_type: FieldType::Number,
                    required: true,
                    constraints: Some(FieldConstraints {
                        min_value: Some(10),
                        max_value: Some(8192),
                        ..Default::default()
                    }),
                    security_level_requirements: None,
                },
                FieldSchema {
                    name: "enable_magic_number_validation".to_string(),
                    field_type: FieldType::Boolean,
                    required: true,
                    constraints: None,
                    security_level_requirements: Some(vec![
                        "production".to_string(),
                        "high_security".to_string(),
                    ]),
                },
                FieldSchema {
                    name: "security_level".to_string(),
                    field_type: FieldType::String,
                    required: true,
                    constraints: Some(FieldConstraints {
                        allowed_values: Some(vec![
                            "development".to_string(),
                            "production".to_string(),
                            "high_security".to_string(),
                        ]),
                        ..Default::default()
                    }),
                    security_level_requirements: None,
                },
                FieldSchema {
                    name: "allowed_extensions".to_string(),
                    field_type: FieldType::Array,
                    required: true,
                    constraints: Some(FieldConstraints {
                        min_length: Some(1),  // At least one extension must be allowed
                        max_length: Some(50), // Prevent excessive permissiveness
                        ..Default::default()
                    }),
                    security_level_requirements: None,
                },
                FieldSchema {
                    name: "max_concurrent_operations".to_string(),
                    field_type: FieldType::Number,
                    required: true,
                    constraints: Some(FieldConstraints {
                        min_value: Some(1),
                        max_value: Some(1000),
                        ..Default::default()
                    }),
                    security_level_requirements: None,
                },
                FieldSchema {
                    name: "enforce_workspace_boundaries".to_string(),
                    field_type: FieldType::Boolean,
                    required: true,
                    constraints: None,
                    security_level_requirements: Some(vec![
                        "production".to_string(),
                        "high_security".to_string(),
                    ]),
                },
                FieldSchema {
                    name: "audit_logging_enabled".to_string(),
                    field_type: FieldType::Boolean,
                    required: false,
                    constraints: None,
                    security_level_requirements: Some(vec![
                        "production".to_string(),
                        "high_security".to_string(),
                    ]),
                },
            ],
            security_constraints: SecurityConstraints {
                production_required_fields: vec![
                    "enable_magic_number_validation".to_string(),
                    "enforce_workspace_boundaries".to_string(),
                    "audit_logging_enabled".to_string(),
                ],
                high_security_required_fields: vec![
                    "enable_magic_number_validation".to_string(),
                    "enforce_workspace_boundaries".to_string(),
                    "audit_logging_enabled".to_string(),
                ],
                forbidden_combinations: vec![
                    ForbiddenCombination {
                        condition: "security_level=production".to_string(),
                        forbidden_field: "enable_magic_number_validation".to_string(),
                        message: "Magic number validation cannot be disabled in production"
                            .to_string(),
                    },
                    ForbiddenCombination {
                        condition: "security_level=high_security".to_string(),
                        forbidden_field: "enforce_workspace_boundaries".to_string(),
                        message: "Workspace boundaries cannot be disabled in high security mode"
                            .to_string(),
                    },
                ],
                mandatory_overrides: vec![
                    MandatoryOverride {
                        security_level: "production".to_string(),
                        field: "enable_magic_number_validation".to_string(),
                        required_value: serde_json::Value::Bool(true),
                    },
                    MandatoryOverride {
                        security_level: "production".to_string(),
                        field: "enforce_workspace_boundaries".to_string(),
                        required_value: serde_json::Value::Bool(true),
                    },
                    MandatoryOverride {
                        security_level: "high_security".to_string(),
                        field: "enable_magic_number_validation".to_string(),
                        required_value: serde_json::Value::Bool(true),
                    },
                    MandatoryOverride {
                        security_level: "high_security".to_string(),
                        field: "enforce_workspace_boundaries".to_string(),
                        required_value: serde_json::Value::Bool(true),
                    },
                    MandatoryOverride {
                        security_level: "high_security".to_string(),
                        field: "max_concurrent_operations".to_string(),
                        required_value: serde_json::Value::Number(serde_json::Number::from(10)),
                    },
                ],
            },
        }
    }
}

impl Default for FieldConstraints {
    fn default() -> Self {
        Self {
            min_length: None,
            max_length: None,
            min_value: None,
            max_value: None,
            allowed_values: None,
            pattern: None,
            format: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_valid_production_config() {
        let validator = ConfigSchemaValidator::new();
        let config = json!({
            "max_file_size": 104857600, // 100MB
            "max_path_length": 260,
            "enable_magic_number_validation": true,
            "security_level": "production",
            "allowed_extensions": [".txt", ".pdf"],
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        assert!(validator.validate_config(&config, "production").is_ok());
    }

    #[test]
    fn test_invalid_production_config_missing_required() {
        let validator = ConfigSchemaValidator::new();
        let config = json!({
            "max_file_size": 104857600,
            "max_path_length": 260,
            "security_level": "production",
            "allowed_extensions": [".txt", ".pdf"],
            "max_concurrent_operations": 10,
            // Missing required fields for production
        });

        assert!(validator.validate_config(&config, "production").is_err());
    }

    #[test]
    fn test_invalid_file_size_too_large() {
        let validator = ConfigSchemaValidator::new();
        let config = json!({
            "max_file_size": 5000000000u64, // 5GB - too large
            "max_path_length": 260,
            "enable_magic_number_validation": true,
            "security_level": "production",
            "allowed_extensions": [".txt"],
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        let result = validator.validate_config(&config, "production");
        assert!(result.is_err());
        if let Err(ValidationError::SchemaValidation { errors }) = result {
            assert!(errors.iter().any(|e| e.contains("Value too large")));
        }
    }

    #[test]
    fn test_invalid_security_level() {
        let validator = ConfigSchemaValidator::new();
        let config = json!({
            "max_file_size": 104857600,
            "max_path_length": 260,
            "enable_magic_number_validation": true,
            "security_level": "invalid_level", // Invalid security level
            "allowed_extensions": [".txt"],
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        let result = validator.validate_config(&config, "production");
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_allowed_extensions() {
        let validator = ConfigSchemaValidator::new();
        let config = json!({
            "max_file_size": 104857600,
            "max_path_length": 260,
            "enable_magic_number_validation": true,
            "security_level": "production",
            "allowed_extensions": [], // Empty array - should fail
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        let result = validator.validate_config(&config, "production");
        assert!(result.is_err());
    }

    #[test]
    fn test_mandatory_override_violation() {
        let validator = ConfigSchemaValidator::new();
        let config = json!({
            "max_file_size": 104857600,
            "max_path_length": 260,
            "enable_magic_number_validation": false, // Should be forced to true in production
            "security_level": "production",
            "allowed_extensions": [".txt"],
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        let result = validator.validate_config(&config, "production");
        assert!(result.is_err());
        if let Err(ValidationError::SchemaValidation { errors }) = result {
            assert!(errors.iter().any(|e| e.contains("must be set to")));
        }
    }
}
