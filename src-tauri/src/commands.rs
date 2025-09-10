// src-tauri/src/commands.rs

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Performance metrics for validation commands
#[derive(Debug, Default)]
pub struct ValidationMetrics {
    pub total_validations: AtomicU64,
    pub successful_validations: AtomicU64,
    pub failed_validations: AtomicU64,
    pub total_validation_time_ns: AtomicU64,
}

impl ValidationMetrics {
    /// Record a successful validation with timing
    pub fn record_success(&self, duration: std::time::Duration) {
        self.total_validations.fetch_add(1, Ordering::Relaxed);
        self.successful_validations.fetch_add(1, Ordering::Relaxed);
        self.total_validation_time_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Record a failed validation with timing
    pub fn record_failure(&self, duration: std::time::Duration) {
        self.total_validations.fetch_add(1, Ordering::Relaxed);
        self.failed_validations.fetch_add(1, Ordering::Relaxed);
        self.total_validation_time_ns
            .fetch_add(duration.as_nanos() as u64, Ordering::Relaxed);
    }

    /// Get the average validation time in nanoseconds
    #[allow(dead_code)]
    pub fn average_validation_time_ns(&self) -> u64 {
        let total_validations = self.total_validations.load(Ordering::Relaxed);
        if total_validations == 0 {
            return 0;
        }
        self.total_validation_time_ns.load(Ordering::Relaxed) / total_validations
    }

    /// Get the validation failure rate as a percentage (0-100)
    #[allow(dead_code)]
    pub fn failure_rate(&self) -> f64 {
        let total_validations = self.total_validations.load(Ordering::Relaxed);
        if total_validations == 0 {
            return 0.0;
        }
        let failed_validations = self.failed_validations.load(Ordering::Relaxed);
        (failed_validations as f64 / total_validations as f64) * 100.0
    }

    /// Reset all metrics to zero
    #[allow(dead_code)]
    pub fn reset(&self) {
        self.total_validations.store(0, Ordering::Relaxed);
        self.successful_validations.store(0, Ordering::Relaxed);
        self.failed_validations.store(0, Ordering::Relaxed);
        self.total_validation_time_ns.store(0, Ordering::Relaxed);
    }
}

// Global validation metrics instance
static VALIDATION_METRICS: Lazy<ValidationMetrics> = Lazy::new(ValidationMetrics::default);

/// Get a reference to the global validation metrics
#[allow(dead_code)]
pub fn get_validation_metrics() -> &'static ValidationMetrics {
    &VALIDATION_METRICS
}

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, serde::Deserialize)]
pub enum CommandError {
    SecurityError {
        message: String,
        code: String,
        severity: String,
    },
    IoError(String),
    Custom(String),
}

impl From<SecurityError> for CommandError {
    fn from(err: SecurityError) -> CommandError {
        CommandError::SecurityError {
            message: err.to_string(),
            code: err.code().to_string(),
            severity: err.severity().to_string(),
        }
    }
}

impl From<std::io::Error> for CommandError {
    fn from(err: std::io::Error) -> CommandError {
        CommandError::IoError(err.to_string())
    }
}

impl From<String> for CommandError {
    fn from(err: String) -> CommandError {
        CommandError::Custom(err)
    }
}

#[derive(Serialize)]
pub struct FileInfo {
    size: u64,
    modified: Option<std::time::SystemTime>,
    is_file: bool,
    is_dir: bool,
}

/// Helper function to create a properly configured validator for production use
fn create_default_validator() -> PathValidator {
    let mut config = SecurityConfig::default();
    // Ensure common extensions are allowed
    config.allowed_extensions.insert(".txt".to_string());

    let mut allowed_paths = vec![
        dirs::desktop_dir().unwrap_or_default(),
        dirs::document_dir().unwrap_or_default(),
        dirs::download_dir().unwrap_or_default(),
    ];

    // Also allow temp directory for temporary operations
    allowed_paths.push(std::env::temp_dir());

    PathValidator::new(config, allowed_paths)
}

// ---------------- Standard Tauri Commands ----------------

#[tauri::command]
pub async fn greet(name: String) -> Result<String, String> {
    Ok(format!("Hello, {}!", name))
}

#[tauri::command]
pub async fn validate_file_for_import(path: String) -> Result<String, CommandError> {
    let start_time = Instant::now();
    let validator = create_default_validator();
    let result = validator.validate_import_path(Path::new(&path));
    SecurityAuditor::log_file_access_attempt(
        Path::new(&path),
        "validate_import",
        &result,
        "development",
        None,
    );

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    result
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn get_file_info_secure(path: String) -> Result<FileInfo, CommandError> {
    let start_time = Instant::now();
    let validator = create_default_validator();
    let validated_path = validator
        .validate_import_path(Path::new(&path))
        .map_err(CommandError::from)?;
    let metadata = fs::metadata(&validated_path).map_err(CommandError::from)?;

    let duration = start_time.elapsed();
    VALIDATION_METRICS.record_success(duration);

    Ok(FileInfo {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
    })
}

#[tauri::command]
pub async fn import_file(path: PathBuf) -> Result<PathBuf, CommandError> {
    let start_time = Instant::now();
    let validator = create_default_validator();
    let result = validator.validate_import_path(&path);

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    result.map_err(CommandError::from)
}

// ---------------- Testable Variants with Custom Validator ----------------

#[allow(dead_code)]
pub async fn validate_file_for_import_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<PathBuf, SecurityError> {
    let result = validator.validate_import_path(path);
    SecurityAuditor::log_file_access_attempt(path, "validate_import", &result, "development", None);
    result
}

#[allow(dead_code)]
pub async fn get_file_info_secure_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<FileInfo, SecurityError> {
    let validated_path = validator.validate_import_path(path)?;
    let metadata =
        fs::metadata(&validated_path).map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        })?;
    Ok(FileInfo {
        size: metadata.len(),
        modified: metadata.modified().ok(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
    })
}

#[allow(dead_code)]
pub async fn import_file_with_validator(
    path: &Path,
    validator: &PathValidator,
) -> Result<PathBuf, SecurityError> {
    validator.validate_import_path(path)
}
