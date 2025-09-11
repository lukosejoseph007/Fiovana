// src-tauri/src/filesystem/operations.rs
// Enhanced operations with production-safe security integration and performance monitoring

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::backup_manager::BackupManager;
use crate::filesystem::security::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
use crate::filesystem::security::emergency_procedures::EmergencyManager;
use crate::filesystem::security::fallback_validator::{FallbackValidator, ValidationResult};
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::safe_mode::SafeModeManager;
use crate::filesystem::security::security_config::SecurityConfig;
use once_cell::sync::Lazy;
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
static FILE_OPERATION_METRICS: Lazy<FileOperationMetrics> =
    Lazy::new(FileOperationMetrics::default);

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

// Global circuit breaker manager for file operations
static CIRCUIT_BREAKER_MANAGER: Lazy<CircuitBreakerManager> = Lazy::new(CircuitBreakerManager::new);

// Global backup manager for configuration backups
static BACKUP_MANAGER: Lazy<BackupManager> =
    Lazy::new(|| BackupManager::new().expect("Failed to initialize backup manager"));

/// Backup security configurations after important operations
/// This is called automatically after validation operations
fn backup_security_configurations() {
    if let Ok(metadata) = BACKUP_MANAGER.create_manual_backup("security") {
        if metadata.success {
            log::info!("Security configuration backup completed successfully");
        } else {
            log::warn!(
                "Security configuration backup failed: {:?}",
                metadata.error_message
            );
        }
    } else {
        log::error!("Failed to initiate security configuration backup");
    }
}

// Enhanced validation with circuit breaker protection
#[allow(dead_code)]
pub fn validate_file_for_import(path: &str) -> Result<String, SecurityError> {
    // First check emergency restrictions
    let emergency_manager =
        EmergencyManager::new().map_err(|e| SecurityError::PathOutsideWorkspace {
            path: format!("Emergency system error: {}", e),
        })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(SecurityError::PathOutsideWorkspace {
            path: "Kill switch active - all operations disabled".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("validate") {
        return Err(SecurityError::PathOutsideWorkspace {
            path: "Operation blocked by emergency restrictions".to_string(),
        });
    }

    // Then check safe mode restrictions
    if !SafeModeManager::global()
        .is_file_allowed(Path::new(path))
        .map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        })?
    {
        return Err(SecurityError::PathOutsideWorkspace {
            path: "File blocked by safe mode restrictions".to_string(),
        });
    }

    let validator = PathValidator::new(
        SecurityConfig::default(),
        vec![
            dirs::desktop_dir().unwrap_or_default(),
            dirs::document_dir().unwrap_or_default(),
            dirs::download_dir().unwrap_or_default(),
            std::env::temp_dir(),
        ],
    );

    // Use circuit breaker for validation
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "file_validation",
        Some(CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: std::time::Duration::from_secs(60),
            success_threshold: 3,
        }),
    );

    let result = breaker.call(|| {
        validator
            .validate_import_path(Path::new(path))
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| anyhow::anyhow!("{}", e))
    });

    let final_result = result.map_err(|e| SecurityError::PathOutsideWorkspace {
        path: e.to_string(),
    });

    // Backup security configurations after successful validation
    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
}

// Enhanced validation with fallback mechanisms
#[allow(dead_code)]
pub fn validate_file_for_import_with_fallback(path: &str) -> Result<String, SecurityError> {
    // First check emergency restrictions
    let emergency_manager =
        EmergencyManager::new().map_err(|e| SecurityError::PathOutsideWorkspace {
            path: format!("Emergency system error: {}", e),
        })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(SecurityError::PathOutsideWorkspace {
            path: "Kill switch active - all operations disabled".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("validate") {
        return Err(SecurityError::PathOutsideWorkspace {
            path: "Operation blocked by emergency restrictions".to_string(),
        });
    }

    // Then check safe mode restrictions
    if !SafeModeManager::global()
        .is_file_allowed(Path::new(path))
        .map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        })?
    {
        return Err(SecurityError::PathOutsideWorkspace {
            path: "File blocked by safe mode restrictions".to_string(),
        });
    }

    let fallback_validator = FallbackValidator::new();
    let result = fallback_validator.validate_file(Path::new(path));

    let final_result = match result {
        Ok(ValidationResult::Approved(_)) | Ok(ValidationResult::Fallback(_)) => {
            // File is approved or validated via fallback
            Ok(path.to_string())
        }
        Ok(ValidationResult::Rejected(reason)) => {
            Err(SecurityError::PathOutsideWorkspace { path: reason })
        }
        Err(e) => Err(SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        }),
    };

    // Backup security configurations after successful validation
    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
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
