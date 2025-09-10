// src-tauri/src/filesystem/operations.rs
// Enhanced operations with production-safe security integration and performance monitoring

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Performance metrics for file operations
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct FileOperationMetrics {
    pub total_operations: AtomicU64,
    pub successful_operations: AtomicU64,
    pub failed_operations: AtomicU64,
    pub total_duration_ns: AtomicU64,
}

impl FileOperationMetrics {
    /// Record a successful operation with timing
    #[allow(dead_code)]
    pub fn record_success(&self, duration: std::time::Duration) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.successful_operations.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Record a failed operation with timing
    #[allow(dead_code)]
    pub fn record_failure(&self, duration: std::time::Duration) {
        self.total_operations.fetch_add(1, Ordering::Relaxed);
        self.failed_operations.fetch_add(1, Ordering::Relaxed);
        self.total_duration_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Get the average operation duration in nanoseconds
    #[allow(dead_code)]
    pub fn average_duration_ns(&self) -> u64 {
        let total_ops = self.total_operations.load(Ordering::Relaxed);
        if total_ops == 0 {
            return 0;
        }
        self.total_duration_ns.load(Ordering::Relaxed) / total_ops
    }

    /// Get the error rate as a percentage (0-100)
    #[allow(dead_code)]
    pub fn error_rate(&self) -> f64 {
        let total_ops = self.total_operations.load(Ordering::Relaxed);
        if total_ops == 0 {
            return 0.0;
        }
        let failed_ops = self.failed_operations.load(Ordering::Relaxed);
        (failed_ops as f64 / total_ops as f64) * 100.0
    }

    /// Reset all metrics to zero
    #[allow(dead_code)]
    pub fn reset(&self) {
        self.total_operations.store(0, Ordering::Relaxed);
        self.successful_operations.store(0, Ordering::Relaxed);
        self.failed_operations.store(0, Ordering::Relaxed);
        self.total_duration_ns.store(0, Ordering::Relaxed);
    }
}

// Global metrics instance
#[allow(dead_code)]
static FILE_OPERATION_METRICS: once_cell::sync::Lazy<FileOperationMetrics> =
    once_cell::sync::Lazy::new(FileOperationMetrics::default);

/// Get a reference to the global file operation metrics
#[allow(dead_code)]
pub fn get_file_operation_metrics() -> &'static FileOperationMetrics {
    &FILE_OPERATION_METRICS
}

/// Helper function to validate file with performance tracking
#[allow(dead_code)]
pub fn validate_file_for_import_with_metrics(path: &str) -> Result<String, SecurityError> {
    let start_time = Instant::now();

    let result = validate_file_for_import(path);

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => FILE_OPERATION_METRICS.record_success(duration),
        Err(_) => FILE_OPERATION_METRICS.record_failure(duration),
    }

    result
}

// Legacy compatibility function - uses PathValidator directly
#[allow(dead_code)]
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    let validator = PathValidator::new(
        SecurityConfig::default(),
        vec![
            dirs::desktop_dir().unwrap_or_default(),
            dirs::document_dir().unwrap_or_default(),
            dirs::download_dir().unwrap_or_default(),
            std::env::temp_dir(),
        ],
    );

    validator
        .validate_import_path(Path::new(path))
        .map(|p| p.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_legacy_validate_function() {
        // Create a temporary file for testing
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").unwrap();

        // Test that the legacy function works
        let result = validate_file_for_import(&test_file.to_string_lossy());
        assert!(result.is_ok());
    }

    #[test]
    fn test_production_security_level() {
        std::env::set_var("PROXEMIC_SECURITY_LEVEL", "production");

        // Test that environment variable affects default config
        let config = SecurityConfig::default();
        assert!(config.enable_magic_number_validation);
        assert!(config.enforce_workspace_boundaries);

        std::env::remove_var("PROXEMIC_SECURITY_LEVEL");
    }
}
