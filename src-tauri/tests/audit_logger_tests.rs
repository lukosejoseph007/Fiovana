use proxemic::filesystem::security::audit_logger::SecurityAuditor;
use proxemic::filesystem::security::audit_logger::SecurityLevel;
use uuid::Uuid;

#[test]
fn test_security_level_parsing() {
    assert_eq!(
        SecurityAuditor::parse_security_level("LOW"),
        SecurityLevel::Low
    );
    assert_eq!(
        SecurityAuditor::parse_security_level("MEDIUM"),
        SecurityLevel::Medium
    );
    assert_eq!(
        SecurityAuditor::parse_security_level("HIGH"),
        SecurityLevel::High
    );
    assert_eq!(
        SecurityAuditor::parse_security_level("CRITICAL"),
        SecurityLevel::Critical
    );
    // Test default for unknown levels
    assert_eq!(
        SecurityAuditor::parse_security_level("UNKNOWN"),
        SecurityLevel::Medium
    );
}

#[test]
fn test_correlation_id_generation() {
    let id1 = SecurityAuditor::new_correlation_id();
    let id2 = SecurityAuditor::new_correlation_id();

    // IDs should be unique
    assert_ne!(id1, id2);
    // IDs should be valid UUIDs (check by parsing)
    assert!(Uuid::parse_str(&id1.to_string()).is_ok());
    assert!(Uuid::parse_str(&id2.to_string()).is_ok());
}

#[test]
fn test_file_access_logging() {
    use proxemic::filesystem::errors::SecurityError;
    use std::path::Path;

    let path = Path::new("/test/path");
    let operation = "read";
    let result: Result<std::path::PathBuf, SecurityError> = Err(SecurityError::AccessDenied {
        path: "/test/path".to_string(),
    });

    // Test that the function compiles and runs without panicking
    SecurityAuditor::log_file_access_attempt(
        path,
        operation,
        &result,
        "HIGH",
        Some(Uuid::new_v4()),
    );
}

#[test]
fn test_security_violation_logging() {
    use std::path::Path;

    let path = Path::new("/test/path");

    // Test that the function compiles and runs without panicking
    SecurityAuditor::log_security_violation(
        "TEST_VIOLATION",
        "Test violation details",
        Some(path),
        Some("test_operation"),
        "CRITICAL",
        Some(Uuid::new_v4()),
    );
}

#[test]
fn test_configuration_change_logging() {
    // Test that the function compiles and runs without panicking
    SecurityAuditor::log_configuration_change(
        "test_config",
        "old_value",
        "new_value",
        "MEDIUM",
        Some(Uuid::new_v4()),
    );
}

#[test]
fn test_environment_override_logging() {
    // Test that the function compiles and runs without panicking
    SecurityAuditor::log_environment_override(
        "TEST_VAR",
        "test_value",
        "LOW",
        Some(Uuid::new_v4()),
    );
}

#[test]
fn test_schema_validation_failure_logging() {
    use serde_json::json;

    let errors = vec!["Error 1".to_string(), "Error 2".to_string()];
    let config = json!({"test": "value"});

    // Test that the function compiles and runs without panicking
    SecurityAuditor::log_schema_validation_failure(&errors, &config, "HIGH", Some(Uuid::new_v4()));
}

#[test]
fn test_permission_escalation_logging() {
    use std::path::Path;

    let path = Path::new("/test/path");

    // Test that the function compiles and runs without panicking
    SecurityAuditor::log_permission_escalation_attempt(
        "test_operation",
        path,
        "CRITICAL",
        Some(Uuid::new_v4()),
    );
}

#[test]
fn test_resource_exhaustion_logging() {
    // Test that the function compiles and runs without panicking
    SecurityAuditor::log_resource_exhaustion("memory", 1024, 512, "HIGH", Some(Uuid::new_v4()));
}
