// Integration test for the audit logger system
#[cfg(test)]
mod tests {
    use proxemic::filesystem::security::audit_logger;
    use proxemic::filesystem::SecurityError;
    use std::path::Path;

    #[test]
    fn test_log_rotation_initialization() {
        // Test that log rotation can be initialized
        let result = audit_logger::SecurityAuditor::init_log_rotation(None);
        assert!(result.is_ok(), "Log rotation initialization should succeed");

        // Test that it's marked as initialized
        assert!(
            audit_logger::SecurityAuditor::is_log_rotation_initialized(),
            "Log rotation should be marked as initialized"
        );
    }

    #[test]
    fn test_file_access_logging() {
        // Test logging a file access attempt
        let test_path = Path::new("/test/file.txt");
        let result: Result<std::path::PathBuf, SecurityError> = Err(SecurityError::AccessDenied {
            path: "test permission".to_string(),
        });

        // This should not panic and should log successfully
        audit_logger::SecurityAuditor::log_file_access_attempt(
            test_path, "read", &result, "HIGH", None,
        );
    }

    #[test]
    fn test_security_level_parsing() {
        // Test security level parsing
        assert_eq!(
            audit_logger::SecurityAuditor::parse_security_level("LOW"),
            audit_logger::SecurityLevel::Low
        );

        assert_eq!(
            audit_logger::SecurityAuditor::parse_security_level("MEDIUM"),
            audit_logger::SecurityLevel::Medium
        );

        assert_eq!(
            audit_logger::SecurityAuditor::parse_security_level("HIGH"),
            audit_logger::SecurityLevel::High
        );

        assert_eq!(
            audit_logger::SecurityAuditor::parse_security_level("CRITICAL"),
            audit_logger::SecurityLevel::Critical
        );

        // Test default case
        assert_eq!(
            audit_logger::SecurityAuditor::parse_security_level("UNKNOWN"),
            audit_logger::SecurityLevel::Medium
        );
    }
}
