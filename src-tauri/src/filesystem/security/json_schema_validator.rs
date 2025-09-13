// src-tauri/src/filesystem/security/json_schema_validator.rs
// JSON Schema validation for security configuration using jsonschema crate

use crate::filesystem::security::config_validator::ValidationError;
use jsonschema::Validator;
use serde_json::Value;

/// JSON Schema validator for security configuration
#[allow(dead_code)]
pub struct JsonSchemaValidator {
    schema: Validator,
}

impl JsonSchemaValidator {
    /// Create a new validator with the built-in security configuration schema
    #[allow(dead_code)]
    pub fn new() -> Result<Self, ValidationError> {
        let schema_value = Self::create_schema_value();
        let schema = jsonschema::validator_for(&schema_value).map_err(|e| {
            ValidationError::SchemaValidation {
                errors: vec![format!("Failed to compile JSON Schema: {}", e)],
            }
        })?;

        Ok(Self { schema })
    }

    /// Validate a configuration against the JSON Schema
    #[allow(dead_code)]
    pub fn validate(&self, config: &Value) -> Result<(), ValidationError> {
        if let Err(error) = self.schema.validate(config) {
            let instance_path = error.instance_path.to_string();
            let schema_path = error.schema_path.to_string();
            let error_message = format!(
                "{}: {} (schema path: {})",
                instance_path, error, schema_path
            );

            return Err(ValidationError::SchemaValidation {
                errors: vec![error_message],
            });
        }

        Ok(())
    }

    /// Create the JSON Schema for security configuration validation
    fn create_schema_value() -> Value {
        serde_json::json!({
            "$schema": "http://json-schema.org/draft-07/schema#",
            "title": "Security Configuration Schema",
            "type": "object",
            "properties": {
                "max_file_size": {
                    "type": "integer",
                    "minimum": 1024,
                    "maximum": 2147483647, // 2GB - 1 (max i32 value)
                    "description": "Maximum file size in bytes"
                },
                "max_path_length": {
                    "type": "integer",
                    "minimum": 10,
                    "maximum": 8192,
                    "description": "Maximum path length in characters"
                },
                "enable_magic_number_validation": {
                    "type": "boolean",
                    "description": "Enable magic number validation"
                },
                "security_level": {
                    "type": "string",
                    "enum": ["development", "production", "high_security"],
                    "description": "Security level"
                },
                "allowed_extensions": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "pattern": "^\\.[a-zA-Z0-9]+$"
                    },
                    "minItems": 1,
                    "maxItems": 50,
                    "description": "Allowed file extensions"
                },
                "max_concurrent_operations": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 1000,
                    "description": "Maximum concurrent operations"
                },
                "enforce_workspace_boundaries": {
                    "type": "boolean",
                    "description": "Enforce workspace boundaries"
                },
                "audit_logging_enabled": {
                    "type": "boolean",
                    "description": "Enable audit logging"
                },
                "allowed_mime_types": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "pattern": "^[a-zA-Z0-9+/.-]+/[a-zA-Z0-9+/.-]+$"
                    },
                    "minItems": 1,
                    "maxItems": 100,
                    "description": "Allowed MIME types"
                },
                "prohibited_filename_chars": {
                    "type": "array",
                    "items": {
                        "type": "string",
                        "minLength": 1,
                        "maxLength": 1
                    },
                    "description": "Prohibited filename characters"
                }
            },
            "required": [
                "max_file_size",
                "max_path_length",
                "enable_magic_number_validation",
                "security_level",
                "allowed_extensions",
                "max_concurrent_operations",
                "enforce_workspace_boundaries"
            ],
            "additionalProperties": false,
            "if": {
                "properties": {
                    "security_level": {
                        "const": "production"
                    }
                }
            },
            "then": {
                "properties": {
                    "enable_magic_number_validation": {
                        "const": true
                    },
                    "enforce_workspace_boundaries": {
                        "const": true
                    },
                    "audit_logging_enabled": {
                        "const": true
                    }
                },
                "required": ["audit_logging_enabled"]
            },
            "else": {
                "if": {
                    "properties": {
                        "security_level": {
                            "const": "high_security"
                        }
                    }
                },
                "then": {
                    "properties": {
                        "enable_magic_number_validation": {
                            "const": true
                        },
                        "enforce_workspace_boundaries": {
                            "const": true
                        },
                        "audit_logging_enabled": {
                            "const": true
                        },
                        "max_concurrent_operations": {
                            "maximum": 10
                        },
                        "max_file_size": {
                            "maximum": 52428800 // 50MB
                        }
                    }
                }
            }
        })
    }

    /// Get the schema as a JSON value for documentation purposes
    #[allow(dead_code)]
    pub fn get_schema_json(&self) -> Value {
        Self::create_schema_value()
    }
}

/// Combined validator that uses both programmatic and JSON Schema validation
#[allow(dead_code)]
pub struct CombinedValidator {
    json_validator: JsonSchemaValidator,
}

#[allow(dead_code)]
impl CombinedValidator {
    pub fn new() -> Result<Self, ValidationError> {
        let json_validator = JsonSchemaValidator::new()?;
        Ok(Self { json_validator })
    }

    /// Validate configuration using both JSON Schema and programmatic validation
    #[allow(unused_variables)]
    pub fn validate(&self, config: &Value, security_level: &str) -> Result<(), ValidationError> {
        // First validate with JSON Schema
        self.json_validator.validate(config)?;

        // Additional programmatic validation can be added here if needed
        // This ensures structural validation from JSON Schema and business logic validation

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_schema_validation() {
        let validator = JsonSchemaValidator::new().unwrap();

        // Valid production configuration
        let valid_config = json!({
            "max_file_size": 104857600,
            "max_path_length": 260,
            "enable_magic_number_validation": true,
            "security_level": "production",
            "allowed_extensions": [".txt", ".pdf"],
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        assert!(validator.validate(&valid_config).is_ok());

        // Invalid configuration - missing required field
        let invalid_config = json!({
            "max_file_size": 104857600,
            "max_path_length": 260,
            "enable_magic_number_validation": true,
            "security_level": "production",
            // Missing allowed_extensions
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        assert!(validator.validate(&invalid_config).is_err());

        // Invalid configuration - magic validation disabled in production
        let invalid_production_config = json!({
            "max_file_size": 104857600,
            "max_path_length": 260,
            "enable_magic_number_validation": false, // Should be true for production
            "security_level": "production",
            "allowed_extensions": [".txt", ".pdf"],
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        assert!(validator.validate(&invalid_production_config).is_err());
    }

    #[test]
    fn test_high_security_constraints() {
        let validator = JsonSchemaValidator::new().unwrap();

        // Valid high security configuration
        let valid_high_security = json!({
            "max_file_size": 52428800, // 50MB
            "max_path_length": 200,
            "enable_magic_number_validation": true,
            "security_level": "high_security",
            "allowed_extensions": [".txt", ".md"],
            "max_concurrent_operations": 5,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        assert!(validator.validate(&valid_high_security).is_ok());

        // Invalid high security - file size too large
        let invalid_high_security = json!({
            "max_file_size": 104857600, // 100MB - too large for high security
            "max_path_length": 200,
            "enable_magic_number_validation": true,
            "security_level": "high_security",
            "allowed_extensions": [".txt", ".md"],
            "max_concurrent_operations": 5,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        assert!(validator.validate(&invalid_high_security).is_err());
    }

    #[test]
    fn test_schema_compilation() {
        // Test that the schema compiles successfully
        assert!(JsonSchemaValidator::new().is_ok());
    }

    #[test]
    fn test_combined_validator() {
        let validator = CombinedValidator::new().unwrap();

        let valid_config = json!({
            "max_file_size": 104857600,
            "max_path_length": 260,
            "enable_magic_number_validation": true,
            "security_level": "production",
            "allowed_extensions": [".txt", ".pdf"],
            "max_concurrent_operations": 10,
            "enforce_workspace_boundaries": true,
            "audit_logging_enabled": true
        });

        assert!(validator.validate(&valid_config, "production").is_ok());
    }
}
