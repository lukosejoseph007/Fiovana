use crate::filesystem::errors::SecurityError;
use std::path::{Path, PathBuf};
use tracing::{error, info, warn};

pub struct SecurityAuditor;

impl SecurityAuditor {
    /// Logs a file access attempt, successful or denied.
    pub fn log_file_access_attempt(
        path: &Path,
        operation: &str,
        result: &Result<PathBuf, SecurityError>,
    ) {
        match result {
            Ok(validated_path) => info!(
                path = %path.display(),
                validated_path = %validated_path.display(),
                operation = operation,
                "File access granted"
            ),
            Err(e) => warn!(
                path = %path.display(),
                operation = operation,
                error = %e,
                error_code = e.code(),
                "File access denied"
            ),
        }
    }

    /// Logs a generic security violation.
    #[allow(dead_code)]
    pub fn log_security_violation(violation_type: &str, details: &str) {
        error!(
            violation_type = violation_type,
            details = details,
            "Security violation detected"
        );
    }
}
