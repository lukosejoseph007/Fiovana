// src-tauri/src/filesystem/errors.rs
// Enhanced security error types merged with existing implementation

use std::fmt;
use thiserror::Error;

/// Comprehensive security error types for the Proxemic filesystem security system
#[derive(Error, Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub enum SecurityError {
    // === Original errors (maintained for compatibility) ===
    #[error("Path traversal attempt detected: {path}")]
    PathTraversal { path: String },

    #[error("Invalid file extension: {extension}")]
    InvalidExtension { extension: String },

    #[error("Path too long: {length} exceeds maximum {max}")]
    PathTooLong { length: usize, max: usize },

    #[error("Filename contains prohibited characters: {filename}")]
    ProhibitedCharacters { filename: String },

    #[error("Access denied to path: {path}")]
    AccessDenied { path: String },

    #[error("File size {size} exceeds maximum allowed {max}")]
    FileTooLarge { size: u64, max: u64 },

    #[error("Path is outside the allowed workspace: {path}")]
    PathOutsideWorkspace { path: String },

    #[error("I/O error occurred: {0}")]
    IoError(String),

    #[error("File validation failed")]
    FileValidationFailed,

    #[error("Invalid file type: {0}")]
    FileTypeViolation(String),

    #[error("MIME type violation: {0}")]
    MimeTypeViolation(String),

    #[error("Magic number mismatch: {0}")]
    MagicNumberMismatch(String),

    // === Enhanced security errors ===
    #[error("Configuration validation failed: {0}")]
    ConfigValidationFailed(String),

    #[error("Operation limit exceeded: {current}/{max} concurrent operations")]
    OperationLimitExceeded { current: u32, max: u32 },

    #[error("Suspicious content detected in {path}: {reason}")]
    SuspiciousContent { path: String, reason: String },

    #[error("Security policy violation: {message}")]
    SecurityPolicyViolation { message: String },

    #[error("Rate limit exceeded for {operation}: {current}/{max} per minute")]
    RateLimitExceeded {
        operation: String,
        current: u32,
        max: u32,
    },

    #[error("Environment configuration error: {variable} - {error}")]
    EnvironmentConfigError { variable: String, error: String },

    #[error("Content validation failed: {reason}")]
    ContentValidationFailed { reason: String },

    #[error("Embedded executable detected in document: {path}")]
    EmbeddedExecutable { path: String },

    #[error("Audit logging failed: {reason}")]
    AuditLoggingFailed { reason: String },

    #[error("Integrity check failed: {path}")]
    IntegrityCheckFailed { path: String },

    #[error("File quarantined due to security threat: {path}")]
    FileQuarantined { path: String },

    #[error("Operation timeout: {operation} exceeded {timeout_ms}ms")]
    OperationTimeout { operation: String, timeout_ms: u64 },

    #[error("Security service unavailable: {service}")]
    SecurityServiceUnavailable { service: String },

    // === Fallback errors ===
    #[error("Security error: {message}")]
    Generic { message: String },
}

impl SecurityError {
    /// Get error code (maintained for existing compatibility)
    pub fn code(&self) -> &'static str {
        match self {
            SecurityError::PathTraversal { .. } => "SEC_PATH_TRAVERSAL",
            SecurityError::InvalidExtension { .. } => "SEC_INVALID_EXTENSION",
            SecurityError::PathTooLong { .. } => "SEC_PATH_TOO_LONG",
            SecurityError::ProhibitedCharacters { .. } => "SEC_PROHIBITED_CHARS",
            SecurityError::AccessDenied { .. } => "SEC_ACCESS_DENIED",
            SecurityError::FileTooLarge { .. } => "SEC_FILE_TOO_LARGE",
            SecurityError::PathOutsideWorkspace { .. } => "SEC_PATH_OUTSIDE_WORKSPACE",
            SecurityError::IoError(_) => "SEC_IO_ERROR",
            SecurityError::FileValidationFailed => "SEC_FILE_VALIDATION_FAILED",
            SecurityError::MimeTypeViolation(_) => "SEC_MIME_VIOLATION",
            SecurityError::MagicNumberMismatch(_) => "SEC_MAGIC_NUMBER_MISMATCH",
            SecurityError::FileTypeViolation(_) => "SEC_FILE_TYPE_VIOLATION",
            SecurityError::ConfigValidationFailed(_) => "SEC_CONFIG_VALIDATION_FAILED",
            SecurityError::OperationLimitExceeded { .. } => "SEC_OPERATION_LIMIT_EXCEEDED",
            SecurityError::SuspiciousContent { .. } => "SEC_SUSPICIOUS_CONTENT",
            SecurityError::SecurityPolicyViolation { .. } => "SEC_POLICY_VIOLATION",
            SecurityError::RateLimitExceeded { .. } => "SEC_RATE_LIMIT_EXCEEDED",
            SecurityError::EnvironmentConfigError { .. } => "SEC_ENV_CONFIG_ERROR",
            SecurityError::ContentValidationFailed { .. } => "SEC_CONTENT_VALIDATION_FAILED",
            SecurityError::EmbeddedExecutable { .. } => "SEC_EMBEDDED_EXECUTABLE",
            SecurityError::AuditLoggingFailed { .. } => "SEC_AUDIT_LOGGING_FAILED",
            SecurityError::IntegrityCheckFailed { .. } => "SEC_INTEGRITY_CHECK_FAILED",
            SecurityError::FileQuarantined { .. } => "SEC_FILE_QUARANTINED",
            SecurityError::OperationTimeout { .. } => "SEC_OPERATION_TIMEOUT",
            SecurityError::SecurityServiceUnavailable { .. } => "SEC_SERVICE_UNAVAILABLE",
            SecurityError::Generic { .. } => "SEC_GENERIC",
        }
    }

    /// Get the severity level of the security error
    pub fn severity(&self) -> SecurityErrorSeverity {
        match self {
            // Critical - System-threatening errors
            SecurityError::EmbeddedExecutable { .. } | SecurityError::SuspiciousContent { .. } => {
                SecurityErrorSeverity::Critical
            }

            // High - Security policy violations and threats
            SecurityError::PathTraversal { .. }
            | SecurityError::SecurityPolicyViolation { .. }
            | SecurityError::AccessDenied { .. }
            | SecurityError::IntegrityCheckFailed { .. }
            | SecurityError::FileQuarantined { .. } => SecurityErrorSeverity::High,

            // Medium - Validation and configuration errors
            SecurityError::InvalidExtension { .. }
            | SecurityError::MimeTypeViolation(_)
            | SecurityError::ConfigValidationFailed(_)
            | SecurityError::OperationLimitExceeded { .. }
            | SecurityError::RateLimitExceeded { .. }
            | SecurityError::ContentValidationFailed { .. } => SecurityErrorSeverity::Medium,

            // Low - Minor violations and warnings
            SecurityError::PathTooLong { .. }
            | SecurityError::FileTooLarge { .. }
            | SecurityError::EnvironmentConfigError { .. } => SecurityErrorSeverity::Low,

            // Info - Informational errors
            SecurityError::FileValidationFailed | SecurityError::OperationTimeout { .. } => {
                SecurityErrorSeverity::Info
            }

            // Default to Medium for unknown errors
            _ => SecurityErrorSeverity::Medium,
        }
    }

    /// Get the error category for logging and metrics
    pub fn category(&self) -> SecurityErrorCategory {
        match self {
            SecurityError::InvalidExtension { .. }
            | SecurityError::PathTooLong { .. }
            | SecurityError::FileTooLarge { .. }
            | SecurityError::ProhibitedCharacters { .. }
            | SecurityError::PathTraversal { .. }
            | SecurityError::PathOutsideWorkspace { .. }
            | SecurityError::FileValidationFailed => SecurityErrorCategory::Validation,

            SecurityError::MimeTypeViolation(_)
            | SecurityError::MagicNumberMismatch(_)
            | SecurityError::FileTypeViolation(_) => SecurityErrorCategory::ContentValidation,

            SecurityError::SuspiciousContent { .. } | SecurityError::EmbeddedExecutable { .. } => {
                SecurityErrorCategory::ThreatDetection
            }

            SecurityError::ConfigValidationFailed(_)
            | SecurityError::EnvironmentConfigError { .. } => SecurityErrorCategory::Configuration,

            SecurityError::OperationLimitExceeded { .. }
            | SecurityError::RateLimitExceeded { .. } => SecurityErrorCategory::ResourceManagement,

            _ => SecurityErrorCategory::General,
        }
    }

    /// Check if the error should trigger an immediate security alert
    pub fn should_alert(&self) -> bool {
        matches!(
            self.severity(),
            SecurityErrorSeverity::Critical | SecurityErrorSeverity::High
        )
    }

    /// Get recommended remediation action
    pub fn remediation(&self) -> &'static str {
        match self {
            SecurityError::SuspiciousContent { .. } => {
                "Quarantine file immediately and scan system"
            }
            SecurityError::PathTraversal { .. } => "Block access and review file path validation",
            SecurityError::EmbeddedExecutable { .. } => {
                "Quarantine document and notify security team"
            }
            SecurityError::IntegrityCheckFailed { .. } => {
                "Verify file authenticity and re-download if necessary"
            }
            SecurityError::OperationLimitExceeded { .. } => {
                "Implement rate limiting or increase resource limits"
            }
            SecurityError::ConfigValidationFailed(_) => "Review and correct configuration settings",
            _ => "Review security logs and take appropriate action",
        }
    }
}

/// Security error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub enum SecurityErrorSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl fmt::Display for SecurityErrorSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityErrorSeverity::Info => write!(f, "INFO"),
            SecurityErrorSeverity::Low => write!(f, "LOW"),
            SecurityErrorSeverity::Medium => write!(f, "MEDIUM"),
            SecurityErrorSeverity::High => write!(f, "HIGH"),
            SecurityErrorSeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Security error categories for classification and metrics
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SecurityErrorCategory {
    Validation,
    ContentValidation,
    ThreatDetection,
    Configuration,
    ResourceManagement,
    Authentication,
    Authorization,
    General,
}

impl fmt::Display for SecurityErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityErrorCategory::Validation => write!(f, "VALIDATION"),
            SecurityErrorCategory::ContentValidation => write!(f, "CONTENT_VALIDATION"),
            SecurityErrorCategory::ThreatDetection => write!(f, "THREAT_DETECTION"),
            SecurityErrorCategory::Configuration => write!(f, "CONFIGURATION"),
            SecurityErrorCategory::ResourceManagement => write!(f, "RESOURCE_MANAGEMENT"),
            SecurityErrorCategory::Authentication => write!(f, "AUTHENTICATION"),
            SecurityErrorCategory::Authorization => write!(f, "AUTHORIZATION"),
            SecurityErrorCategory::General => write!(f, "GENERAL"),
        }
    }
}

// === Original ValidationError (maintained for compatibility) ===
#[allow(dead_code)]
#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum ValidationError {
    #[error("File type validation failed: {reason}")]
    FileType { reason: String },

    #[error("Magic number mismatch for file type: {expected} vs {actual}")]
    MagicNumber { expected: String, actual: String },

    #[error("File corruption detected: {details}")]
    Corruption { details: String },

    #[error("File size {size} exceeds maximum allowed {max}")]
    FileSize { size: u64, max: u64 },

    #[error("MIME type violation: {mime}")]
    MimeType { mime: String },
}

impl ValidationError {
    pub fn code(&self) -> &'static str {
        match self {
            ValidationError::FileType { .. } => "VAL_FILE_TYPE",
            ValidationError::MagicNumber { .. } => "VAL_MAGIC_MISMATCH",
            ValidationError::Corruption { .. } => "VAL_CORRUPTION",
            ValidationError::FileSize { .. } => "VAL_FILE_SIZE",
            ValidationError::MimeType { .. } => "VAL_MIME_TYPE",
        }
    }
}

impl From<std::io::Error> for SecurityError {
    fn from(err: std::io::Error) -> SecurityError {
        SecurityError::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_error_display() {
        let err = SecurityError::PathTraversal {
            path: "/etc/passwd".into(),
        };
        assert_eq!(
            format!("{}", err),
            "Path traversal attempt detected: /etc/passwd"
        );
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::MagicNumber {
            expected: "PDF".into(),
            actual: "TXT".into(),
        };
        assert_eq!(
            format!("{}", err),
            "Magic number mismatch for file type: PDF vs TXT"
        );
    }

    #[test]
    fn test_security_error_code() {
        let err = SecurityError::AccessDenied {
            path: "C:/restricted.txt".into(),
        };
        assert_eq!(err.code(), "SEC_ACCESS_DENIED");
    }

    #[test]
    fn test_error_severity_classification() {
        let malicious_error = SecurityError::SuspiciousContent {
            path: "/test/file".to_string(),
            reason: "Test malware".to_string(),
        };
        assert_eq!(malicious_error.severity(), SecurityErrorSeverity::Critical);
        assert!(malicious_error.should_alert());

        let validation_error = SecurityError::InvalidExtension {
            extension: ".exe".to_string(),
        };
        assert_eq!(validation_error.severity(), SecurityErrorSeverity::Medium);
    }

    #[test]
    fn test_error_categorization() {
        let threat_error = SecurityError::SuspiciousContent {
            path: "/test/file".to_string(),
            reason: "Test reason".to_string(),
        };
        assert_eq!(
            threat_error.category(),
            SecurityErrorCategory::ThreatDetection
        );

        let config_error = SecurityError::ConfigValidationFailed("Test".to_string());
        assert_eq!(
            config_error.category(),
            SecurityErrorCategory::Configuration
        );
    }
}
