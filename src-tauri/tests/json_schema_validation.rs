// src-tauri/tests/json_schema_validation.rs
// Comprehensive tests for JSON Schema validation of security configuration

use fiovana::filesystem::security::json_schema_validator::JsonSchemaValidator;
use serde_json::json;

#[test]
fn test_comprehensive_json_schema_validation() {
    let validator = JsonSchemaValidator::new().unwrap();

    // Test valid development configuration
    let valid_dev_config = json!({
        "max_file_size": 52428800,
        "max_path_length": 260,
        "enable_magic_number_validation": false,
        "security_level": "development",
        "allowed_extensions": [".txt", ".pdf", ".docx"],
        "max_concurrent_operations": 50,
        "enforce_workspace_boundaries": false,
        "audit_logging_enabled": false
    });

    assert!(validator.validate(&valid_dev_config).is_ok());

    // Test valid production configuration
    let valid_prod_config = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&valid_prod_config).is_ok());

    // Test valid high security configuration
    let valid_high_security_config = json!({
        "max_file_size": 52428800,
        "max_path_length": 200,
        "enable_magic_number_validation": true,
        "security_level": "high_security",
        "allowed_extensions": [".txt", ".md"],
        "max_concurrent_operations": 5,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&valid_high_security_config).is_ok());
}

#[test]
fn test_invalid_configurations() {
    let validator = JsonSchemaValidator::new().unwrap();

    // Test missing required field
    let missing_required = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        // Missing allowed_extensions
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&missing_required).is_err());

    // Test invalid security level
    let invalid_security_level = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "invalid_level",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&invalid_security_level).is_err());

    // Test file size too small
    let file_size_too_small = json!({
        "max_file_size": 500,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&file_size_too_small).is_err());

    // Test file size too large
    let file_size_too_large = json!({
        "max_file_size": 5000000000u64,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&file_size_too_large).is_err());
}

#[test]
fn test_production_constraints() {
    let validator = JsonSchemaValidator::new().unwrap();

    // Test production with magic validation disabled (should fail)
    let invalid_production = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": false,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&invalid_production).is_err());

    // Test production with workspace boundaries disabled (should fail)
    let invalid_production_2 = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": false,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&invalid_production_2).is_err());

    // Test production with audit logging disabled (should fail)
    let invalid_production_3 = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": false
    });

    assert!(validator.validate(&invalid_production_3).is_err());
}

#[test]
fn test_high_security_constraints() {
    let validator = JsonSchemaValidator::new().unwrap();

    // Test high security with file size too large (should fail)
    let invalid_high_security = json!({
        "max_file_size": 104857600,
        "max_path_length": 200,
        "enable_magic_number_validation": true,
        "security_level": "high_security",
        "allowed_extensions": [".txt", ".md"],
        "max_concurrent_operations": 5,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&invalid_high_security).is_err());

    // Test high security with concurrent operations too high (should fail)
    let invalid_high_security_2 = json!({
        "max_file_size": 52428800,
        "max_path_length": 200,
        "enable_magic_number_validation": true,
        "security_level": "high_security",
        "allowed_extensions": [".txt", ".md"],
        "max_concurrent_operations": 20,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&invalid_high_security_2).is_err());
}

#[test]
fn test_array_validation() {
    let validator = JsonSchemaValidator::new().unwrap();

    // Test empty allowed_extensions (should fail)
    let empty_extensions = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&empty_extensions).is_err());

    // Test too many allowed_extensions (should fail)
    let many_extensions: Vec<String> = (1..=60).map(|i| format!(".ext{}", i)).collect();

    let too_many_extensions = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": many_extensions,
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator.validate(&too_many_extensions).is_err());
}

#[test]
fn test_additional_properties_rejection() {
    let validator = JsonSchemaValidator::new().unwrap();

    // Test configuration with additional properties (should fail)
    let with_extra_properties = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true,
        "extra_field": "should_cause_error"
    });

    assert!(validator.validate(&with_extra_properties).is_err());
}

#[test]
fn test_schema_compilation_and_reuse() {
    // Test that we can create multiple validators
    let validator1 = JsonSchemaValidator::new().unwrap();
    let validator2 = JsonSchemaValidator::new().unwrap();

    let config = json!({
        "max_file_size": 104857600,
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    assert!(validator1.validate(&config).is_ok());
    assert!(validator2.validate(&config).is_ok());
}

#[test]
fn test_error_messages_are_descriptive() {
    let validator = JsonSchemaValidator::new().unwrap();

    let invalid_config = json!({
        "max_file_size": 500, // Too small
        "max_path_length": 260,
        "enable_magic_number_validation": true,
        "security_level": "production",
        "allowed_extensions": [".txt", ".pdf"],
        "max_concurrent_operations": 10,
        "enforce_workspace_boundaries": true,
        "audit_logging_enabled": true
    });

    let result = validator.validate(&invalid_config);
    assert!(result.is_err());

    if let Err(err) = result {
        let error_str = err.to_string();
        assert!(error_str.contains("Value too small") || error_str.contains("minimum"));
    }
}
