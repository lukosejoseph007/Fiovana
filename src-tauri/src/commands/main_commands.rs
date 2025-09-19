// src-tauri/src/commands.rs

use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::backup_manager::BackupManager;
use crate::filesystem::security::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
use crate::filesystem::security::emergency_procedures::EmergencyManager;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::safe_mode::SafeModeManager;
use crate::filesystem::security::security_config::SecurityConfig;
use crate::document::{ContentHash, MetadataExtractor, FileProcessor, BatchHasher, DuplicateCheckResult,
    FileValidationResult, CorruptionCheckResult, EnhancedMetadata, ProgressManager, ImportProgress,
    ImportNotificationManager, ImportNotification, ImportError, ImportErrorInfo,
    ProgressPersistenceManager, PersistedProgress, ProgressStorageStats};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
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

// Global circuit breaker manager for Tauri commands
static CIRCUIT_BREAKER_MANAGER: Lazy<CircuitBreakerManager> = Lazy::new(CircuitBreakerManager::new);

// Global backup manager for configuration backups
static BACKUP_MANAGER: Lazy<BackupManager> =
    Lazy::new(|| BackupManager::new().expect("Failed to initialize backup manager"));

/// Backup security configurations after important operations
/// This is called automatically after successful Tauri commands
fn backup_security_configurations() {
    if let Ok(metadata) = BACKUP_MANAGER.create_manual_backup("tauri_security") {
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

/// Get a reference to the global validation metrics
#[allow(dead_code)]
pub fn get_validation_metrics() -> &'static ValidationMetrics {
    &VALIDATION_METRICS
}

use crate::filesystem::watcher::{DocumentWatcher, FileEvent, WatcherConfig};
use tokio::sync::{mpsc, Mutex};

// Global file watcher instance (production use with Wry runtime)
static FILE_WATCHER: Lazy<Mutex<Option<DocumentWatcher<tauri::Wry>>>> =
    Lazy::new(|| Mutex::new(None));
static EVENT_RECEIVER: Lazy<Mutex<Option<mpsc::UnboundedReceiver<FileEvent>>>> =
    Lazy::new(|| Mutex::new(None));

// Test-specific watcher storage (only used in tests)
#[cfg(test)]
static TEST_FILE_WATCHER: Lazy<Mutex<Option<Box<dyn std::any::Any + Send + Sync>>>> =
    Lazy::new(|| Mutex::new(None));
#[cfg(test)]
static TEST_EVENT_RECEIVER: Lazy<Mutex<Option<mpsc::UnboundedReceiver<FileEvent>>>> =
    Lazy::new(|| Mutex::new(None));

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
    // Ensure common extensions are allowed, including .tmp for testing
    config.allowed_extensions.insert(".txt".to_string());
    config.allowed_extensions.insert(".tmp".to_string()); // Allow temporary files for testing

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

    // First check emergency restrictions
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("validate") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Then check safe mode restrictions
    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker for validation
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_file_validation",
        Some(CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: std::time::Duration::from_secs(60),
            success_threshold: 3,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        validator
            .validate_import_path(Path::new(&path))
            .map(|p| p.to_string_lossy().to_string())
            .map_err(|e| anyhow::anyhow!("{}", e))
    });

    // Convert result to expected type for audit logging
    let audit_result: Result<PathBuf, SecurityError> = result
        .as_ref()
        .map(|_| Path::new(&path).to_path_buf())
        .map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        });

    SecurityAuditor::log_file_access_attempt(
        Path::new(&path),
        "validate_import",
        &audit_result,
        "development",
        None,
    );

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    let final_result = result.map_err(|e| CommandError::SecurityError {
        message: e.to_string(),
        code: "CIRCUIT_BREAKER".to_string(),
        severity: "medium".to_string(),
    });

    // Backup security configurations after successful validation
    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
}

#[tauri::command]
pub async fn get_file_info_secure(path: String) -> Result<FileInfo, CommandError> {
    let start_time = Instant::now();

    // First check emergency restrictions
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("read") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Then check safe mode restrictions
    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker for validation
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_file_info",
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            success_threshold: 2,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        let validated_path = validator
            .validate_import_path(Path::new(&path))
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let metadata = fs::metadata(&validated_path)
            .map_err(|e| anyhow::anyhow!("Failed to get file metadata: {}", e))?;

        Ok(FileInfo {
            size: metadata.len(),
            modified: metadata.modified().ok(),
            is_file: metadata.is_file(),
            is_dir: metadata.is_dir(),
        })
    });

    // Convert result to expected type for audit logging
    let audit_result: Result<PathBuf, SecurityError> = result
        .as_ref()
        .map(|_| Path::new(&path).to_path_buf())
        .map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        });

    SecurityAuditor::log_file_access_attempt(
        Path::new(&path),
        "get_file_info",
        &audit_result,
        "development",
        None,
    );

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    let final_result = result.map_err(|e| CommandError::SecurityError {
        message: e.to_string(),
        code: "CIRCUIT_BREAKER".to_string(),
        severity: "medium".to_string(),
    });

    // Backup security configurations after successful operation
    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
}

#[tauri::command]
pub async fn import_file(path: PathBuf) -> Result<PathBuf, CommandError> {
    let start_time = Instant::now();

    // First check emergency restrictions
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("import") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Then check safe mode restrictions
    if !SafeModeManager::global()
        .is_file_allowed(&path)
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker for import operation
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_file_import",
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            success_threshold: 2,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        validator
            .validate_import_path(&path)
            .map_err(|e| anyhow::anyhow!("{}", e))
    });

    // Convert result to expected type for audit logging
    let audit_result: Result<PathBuf, SecurityError> = result
        .as_ref()
        .map(|_| path.clone())
        .map_err(|e| SecurityError::PathOutsideWorkspace {
            path: e.to_string(),
        });

    SecurityAuditor::log_file_access_attempt(
        &path,
        "import_file",
        &audit_result,
        "development",
        None,
    );

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    let final_result = result.map_err(|e| CommandError::SecurityError {
        message: e.to_string(),
        code: "CIRCUIT_BREAKER".to_string(),
        severity: "medium".to_string(),
    });

    // Backup security configurations after successful import
    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
}

// ---------------- Document Processing Commands ----------------

#[tauri::command]
pub async fn validate_file_comprehensive(path: String) -> Result<FileValidationResult, CommandError> {
    let start_time = Instant::now();

    // Emergency and safe mode checks
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("validate") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_file_validation_comprehensive",
        Some(CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: std::time::Duration::from_secs(60),
            success_threshold: 3,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        // First validate path security
        let validated_path = validator
            .validate_import_path(Path::new(&path))
            .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

        // Then perform comprehensive file validation
        FileProcessor::validate_file(&validated_path)
            .map_err(|e| anyhow::anyhow!("File validation failed: {}", e))
    });

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    let final_result = result.map_err(|e| CommandError::Custom(e.to_string()));

    if final_result.is_ok() {
        backup_security_configurations();
    }

    final_result
}

#[tauri::command]
pub async fn check_file_corruption(path: String) -> Result<CorruptionCheckResult, CommandError> {
    let start_time = Instant::now();

    // Emergency and safe mode checks
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("validate") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_corruption_check",
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            success_threshold: 2,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        // First validate path security
        let validated_path = validator
            .validate_import_path(Path::new(&path))
            .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

        // Then check for corruption
        FileProcessor::check_corruption(&validated_path)
            .map_err(|e| anyhow::anyhow!("Corruption check failed: {}", e))
    });

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    result.map_err(|e| CommandError::Custom(e.to_string()))
}

#[tauri::command]
pub async fn extract_file_metadata(path: String) -> Result<EnhancedMetadata, CommandError> {
    let start_time = Instant::now();

    // Emergency and safe mode checks
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("read") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_metadata_extraction",
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            success_threshold: 2,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        // First validate path security
        let validated_path = validator
            .validate_import_path(Path::new(&path))
            .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

        // Then extract metadata
        MetadataExtractor::extract(&validated_path)
            .map_err(|e| anyhow::anyhow!("Metadata extraction failed: {}", e))
    });

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    result.map_err(|e| CommandError::Custom(e.to_string()))
}

#[tauri::command]
pub async fn calculate_file_hash(path: String) -> Result<ContentHash, CommandError> {
    let start_time = Instant::now();

    // Emergency and safe mode checks
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("read") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_file_hashing",
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            success_threshold: 2,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        // First validate path security
        let validated_path = validator
            .validate_import_path(Path::new(&path))
            .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

        // Then calculate hash
        ContentHash::from_file(&validated_path)
            .map_err(|e| anyhow::anyhow!("Hash calculation failed: {}", e))
    });

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    result.map_err(|e| CommandError::Custom(e.to_string()))
}

// Global batch hasher for duplicate detection across multiple files
static BATCH_HASHER: Lazy<Mutex<BatchHasher>> = Lazy::new(|| Mutex::new(BatchHasher::new()));

// Global progress manager for tracking import operations
static PROGRESS_MANAGER: Lazy<Mutex<ProgressManager>> = Lazy::new(|| Mutex::new(ProgressManager::new()));

// Global notification manager for import operations
static NOTIFICATION_MANAGER: Lazy<Mutex<ImportNotificationManager>> =
    Lazy::new(|| Mutex::new(ImportNotificationManager::new()));

// Global progress persistence manager
static PROGRESS_PERSISTENCE: Lazy<Mutex<Option<ProgressPersistenceManager>>> =
    Lazy::new(|| Mutex::new(None));

#[tauri::command]
pub async fn check_file_duplicates(path: String) -> Result<DuplicateCheckResult, CommandError> {
    let start_time = Instant::now();

    // Emergency and safe mode checks
    let emergency_manager = EmergencyManager::new().map_err(|e| CommandError::SecurityError {
        message: format!("Emergency system error: {}", e),
        code: "EMERGENCY_ERROR".to_string(),
        severity: "high".to_string(),
    })?;

    if emergency_manager.is_kill_switch_active() {
        return Err(CommandError::SecurityError {
            message: "Kill switch active - all operations disabled".to_string(),
            code: "KILL_SWITCH_ACTIVE".to_string(),
            severity: "critical".to_string(),
        });
    }

    if !emergency_manager.can_perform_operation("read") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&path))
        .map_err(|e| CommandError::SecurityError {
            message: format!("Safe mode restriction: {}", e),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        })?
    {
        return Err(CommandError::SecurityError {
            message: "File blocked by safe mode restrictions".to_string(),
            code: "SAFE_MODE_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker
    let breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "tauri_duplicate_check",
        Some(CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: std::time::Duration::from_secs(30),
            success_threshold: 2,
        }),
    );

    let validator = create_default_validator();
    let result = breaker.call(|| {
        // First validate path security
        let validated_path = validator
            .validate_import_path(Path::new(&path))
            .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

        // Then process file for duplicates using global hasher
        // We need to use a blocking operation here since we're inside a sync closure
        let hasher_future = BATCH_HASHER.lock();
        let mut hasher = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(hasher_future)
        });
        hasher.process_file(&validated_path)
            .map_err(|e| anyhow::anyhow!("Duplicate check failed: {}", e))
    });

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => VALIDATION_METRICS.record_success(duration),
        Err(_) => VALIDATION_METRICS.record_failure(duration),
    }

    result.map_err(|e| CommandError::Custom(e.to_string()))
}

#[tauri::command]
pub async fn clear_duplicate_cache() -> Result<(), CommandError> {
    let mut hasher = BATCH_HASHER.lock().await;
    hasher.clear();
    Ok(())
}

// ---------------- Progress Tracking Commands ----------------

#[tauri::command]
pub async fn start_import_operation(file_count: u64) -> Result<String, CommandError> {
    let manager = PROGRESS_MANAGER.lock().await;
    let tracker = manager.start_operation(file_count).await;

    // Set initial steps
    tracker.add_step("validation".to_string(), "Validating files for import".to_string()).await;
    tracker.add_step("processing".to_string(), "Processing file contents".to_string()).await;
    tracker.add_step("storage".to_string(), "Storing processed files".to_string()).await;
    tracker.add_step("indexing".to_string(), "Updating search index".to_string()).await;

    Ok(tracker.operation_id().to_string())
}

#[tauri::command]
pub async fn get_import_progress(operation_id: String) -> Result<Option<ImportProgress>, CommandError> {
    let manager = PROGRESS_MANAGER.lock().await;
    Ok(manager.get_operation_progress(&operation_id).await)
}

#[tauri::command]
pub async fn cancel_import_operation(operation_id: String) -> Result<bool, CommandError> {
    let manager = PROGRESS_MANAGER.lock().await;
    Ok(manager.cancel_operation(&operation_id).await)
}

#[tauri::command]
pub async fn get_all_import_operations() -> Result<Vec<ImportProgress>, CommandError> {
    let manager = PROGRESS_MANAGER.lock().await;
    Ok(manager.get_all_operations().await)
}

#[tauri::command]
pub async fn cleanup_import_operation(operation_id: String) -> Result<(), CommandError> {
    let manager = PROGRESS_MANAGER.lock().await;
    manager.cleanup_operation(&operation_id).await;
    Ok(())
}

#[tauri::command]
pub async fn cleanup_old_import_operations(max_age_hours: u64) -> Result<(), CommandError> {
    let manager = PROGRESS_MANAGER.lock().await;
    let max_age = std::time::Duration::from_secs(max_age_hours * 3600);
    manager.cleanup_old_operations(max_age).await;
    Ok(())
}

// ---------------- Error Handling and Notification Commands ----------------

#[tauri::command]
pub async fn report_import_error(
    file_path: Option<String>,
    error_type: String,
    details: String
) -> Result<(), CommandError> {
    let path = file_path.map(PathBuf::from);

    // Create appropriate error type based on error_type string
    let import_error = match error_type.as_str() {
        "file_not_found" => ImportError::FileNotFound {
            path: path.unwrap_or_else(|| PathBuf::from("unknown"))
        },
        "permission_denied" => ImportError::PermissionDenied {
            path: path.unwrap_or_else(|| PathBuf::from("unknown"))
        },
        "file_too_large" => ImportError::FileTooLarge {
            path: path.unwrap_or_else(|| PathBuf::from("unknown")),
            size: 0, // Would be provided in real scenario
            max_size: 0, // Would be provided in real scenario
        },
        "unsupported_type" => ImportError::UnsupportedFileType {
            path: path.unwrap_or_else(|| PathBuf::from("unknown")),
            detected_type: None,
            expected_types: vec!["pdf".to_string(), "docx".to_string()],
        },
        "corrupted" => ImportError::FileCorrupted {
            path: path.unwrap_or_else(|| PathBuf::from("unknown")),
            details: details.clone(),
        },
        "cancelled" => ImportError::OperationCancelled,
        _ => ImportError::Unknown { details },
    };

    let error_info = import_error.to_error_info();
    let manager = NOTIFICATION_MANAGER.lock().await;
    manager.notify_error(&error_info).await;

    Ok(())
}

#[tauri::command]
pub async fn notify_import_success(
    title: String,
    message: String,
    file_count: Option<u32>
) -> Result<(), CommandError> {
    let manager = NOTIFICATION_MANAGER.lock().await;
    manager.notify_success(title, message, file_count).await;
    Ok(())
}

#[tauri::command]
pub async fn notify_import_info(title: String, message: String) -> Result<(), CommandError> {
    let manager = NOTIFICATION_MANAGER.lock().await;
    manager.notify_info(title, message).await;
    Ok(())
}

#[tauri::command]
pub async fn get_import_notifications() -> Result<Vec<ImportNotification>, CommandError> {
    let manager = NOTIFICATION_MANAGER.lock().await;
    Ok(manager.get_notifications().await)
}

#[tauri::command]
pub async fn clear_import_notifications() -> Result<(), CommandError> {
    let manager = NOTIFICATION_MANAGER.lock().await;
    manager.clear_notifications().await;
    Ok(())
}

#[tauri::command]
pub async fn remove_import_notification(index: usize) -> Result<(), CommandError> {
    let manager = NOTIFICATION_MANAGER.lock().await;
    manager.remove_notification(index).await;
    Ok(())
}

#[tauri::command]
pub async fn convert_error_to_info(error_message: String) -> Result<ImportErrorInfo, CommandError> {
    // Parse common error types and convert to structured error info
    let import_error = if error_message.contains("Permission denied") {
        ImportError::PermissionDenied {
            path: PathBuf::from("unknown")
        }
    } else if error_message.contains("No such file") {
        ImportError::FileNotFound {
            path: PathBuf::from("unknown")
        }
    } else if error_message.contains("too large") {
        ImportError::FileTooLarge {
            path: PathBuf::from("unknown"),
            size: 0,
            max_size: 0,
        }
    } else {
        ImportError::Unknown {
            details: error_message
        }
    };

    Ok(import_error.to_error_info())
}

// ---------------- Progress Persistence Commands ----------------

#[tauri::command]
pub async fn initialize_progress_persistence(storage_dir: String) -> Result<(), CommandError> {
    let manager = ProgressPersistenceManager::new(&storage_dir)
        .map_err(|e| CommandError::Custom(format!("Failed to initialize progress persistence: {}", e)))?;

    let mut persistence = PROGRESS_PERSISTENCE.lock().await;
    *persistence = Some(manager);

    Ok(())
}

#[tauri::command]
pub async fn persist_operation_progress(
    operation_id: String,
    remaining_files: Vec<String>,
    processed_files: Vec<String>,
    failed_files: Vec<(String, String)>,
) -> Result<(), CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard.as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    // Get current progress
    let progress_manager = PROGRESS_MANAGER.lock().await;
    let progress = progress_manager.get_operation_progress(&operation_id).await
        .ok_or_else(|| CommandError::Custom("Operation not found".to_string()))?;

    // Convert string paths to PathBuf
    let remaining: Vec<PathBuf> = remaining_files.into_iter().map(PathBuf::from).collect();
    let processed: Vec<PathBuf> = processed_files.into_iter().map(PathBuf::from).collect();
    let failed: Vec<(PathBuf, String)> = failed_files.into_iter()
        .map(|(path, error)| (PathBuf::from(path), error))
        .collect();

    persistence.persist_progress(&operation_id, &progress, remaining, processed, failed).await
        .map_err(|e| CommandError::Custom(format!("Failed to persist progress: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn load_persisted_progress(operation_id: String) -> Result<Option<PersistedProgress>, CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard.as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence.load_progress(&operation_id).await
        .map_err(|e| CommandError::Custom(format!("Failed to load progress: {}", e)))
}

#[tauri::command]
pub async fn list_resumable_operations() -> Result<Vec<PersistedProgress>, CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard.as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence.get_resumable_operations().await
        .map_err(|e| CommandError::Custom(format!("Failed to get resumable operations: {}", e)))
}

#[tauri::command]
pub async fn mark_operation_completed(operation_id: String) -> Result<(), CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard.as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence.mark_completed(&operation_id).await
        .map_err(|e| CommandError::Custom(format!("Failed to mark operation completed: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn mark_file_processed(
    operation_id: String,
    file_path: String,
    success: bool,
    error: Option<String>,
) -> Result<(), CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard.as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence.mark_file_completed(&operation_id, PathBuf::from(file_path), success, error).await
        .map_err(|e| CommandError::Custom(format!("Failed to mark file processed: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn cleanup_persisted_operations(max_age_hours: u64) -> Result<u32, CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard.as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    let max_age = std::time::Duration::from_secs(max_age_hours * 3600);
    persistence.cleanup_old_operations(max_age).await
        .map_err(|e| CommandError::Custom(format!("Failed to cleanup operations: {}", e)))
}

#[tauri::command]
pub async fn get_progress_storage_stats() -> Result<ProgressStorageStats, CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard.as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence.get_storage_stats().await
        .map_err(|e| CommandError::Custom(format!("Failed to get storage stats: {}", e)))
}

#[tauri::command]
pub async fn remove_persisted_progress(operation_id: String) -> Result<(), CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard.as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence.remove_progress(&operation_id).await
        .map_err(|e| CommandError::Custom(format!("Failed to remove progress: {}", e)))?;

    Ok(())
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

// ---------------- File Watcher Commands ----------------

#[tauri::command]
pub async fn start_file_watching(
    workspace_path: String,
    app_handle: tauri::AppHandle<tauri::Wry>,
) -> Result<(), CommandError> {
    start_file_watching_internal(workspace_path, app_handle).await
}

/// Internal implementation for production use with Wry runtime
async fn start_file_watching_internal(
    workspace_path: String,
    app_handle: tauri::AppHandle<tauri::Wry>,
) -> Result<(), CommandError> {
    let mut watcher_guard = FILE_WATCHER.lock().await;
    let mut receiver_guard = EVENT_RECEIVER.lock().await;

    // If watcher already exists, stop it first
    if watcher_guard.is_some() {
        *watcher_guard = None;
        *receiver_guard = None;
    }

    // Create new watcher with default config
    let config = WatcherConfig::default();
    let (mut watcher, receiver) = DocumentWatcher::new(config, app_handle);

    // Start the watcher
    watcher
        .start()
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to start file watcher: {}", e)))?;

    // Add the workspace path with security validation
    watcher
        .add_path(&workspace_path)
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to add workspace path: {}", e)))?;

    // Store the watcher and receiver
    *watcher_guard = Some(watcher);
    *receiver_guard = Some(receiver);

    tracing::info!("File watcher started for workspace: {}", workspace_path);
    Ok(())
}

/// Test-specific implementation that can be used with different runtime types
#[cfg(test)]
async fn start_file_watching_test_internal<R: tauri::Runtime>(
    workspace_path: String,
    app_handle: tauri::AppHandle<R>,
) -> Result<(), CommandError> {
    let mut watcher_guard = TEST_FILE_WATCHER.lock().await;
    let mut receiver_guard = TEST_EVENT_RECEIVER.lock().await;

    // If watcher already exists, stop it first
    if watcher_guard.is_some() {
        *watcher_guard = None;
        *receiver_guard = None;
    }

    // Create new watcher with default config
    let config = WatcherConfig::default();
    let (mut watcher, receiver) = DocumentWatcher::new(config, app_handle);

    // Start the watcher
    watcher
        .start()
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to start file watcher: {}", e)))?;

    // Add the workspace path with security validation
    watcher
        .add_path(&workspace_path)
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to add workspace path: {}", e)))?;

    // Store the watcher and receiver
    *watcher_guard = Some(Box::new(watcher));
    *receiver_guard = Some(receiver);

    tracing::info!("File watcher started for workspace: {}", workspace_path);
    Ok(())
}

#[tauri::command]
pub async fn pause_file_watching() -> Result<(), CommandError> {
    #[cfg(not(test))]
    {
        let watcher_guard = FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_ref() {
            watcher.pause().await;
            tracing::info!("File watching paused");
            Ok(())
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }

    #[cfg(test)]
    {
        let watcher_guard = TEST_FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_ref() {
            if let Some(watcher) =
                watcher.downcast_ref::<DocumentWatcher<tauri::test::MockRuntime>>()
            {
                watcher.pause().await;
                tracing::info!("File watching paused");
                Ok(())
            } else {
                Err(CommandError::Custom(
                    "File watcher type mismatch".to_string(),
                ))
            }
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }
}

#[tauri::command]
pub async fn resume_file_watching() -> Result<(), CommandError> {
    #[cfg(not(test))]
    {
        let watcher_guard = FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_ref() {
            watcher.resume().await;
            tracing::info!("File watching resumed");
            Ok(())
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }

    #[cfg(test)]
    {
        let watcher_guard = TEST_FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_ref() {
            if let Some(watcher) =
                watcher.downcast_ref::<DocumentWatcher<tauri::test::MockRuntime>>()
            {
                watcher.resume().await;
                tracing::info!("File watching resumed");
                Ok(())
            } else {
                Err(CommandError::Custom(
                    "File watcher type mismatch".to_string(),
                ))
            }
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }
}

#[tauri::command]
pub async fn stop_file_watching() -> Result<(), CommandError> {
    #[cfg(not(test))]
    {
        let mut watcher_guard = FILE_WATCHER.lock().await;
        let mut receiver_guard = EVENT_RECEIVER.lock().await;

        *watcher_guard = None;
        *receiver_guard = None;

        tracing::info!("File watching stopped");
        Ok(())
    }

    #[cfg(test)]
    {
        let mut watcher_guard = TEST_FILE_WATCHER.lock().await;
        let mut receiver_guard = TEST_EVENT_RECEIVER.lock().await;

        *watcher_guard = None;
        *receiver_guard = None;

        tracing::info!("File watching stopped");
        Ok(())
    }
}

#[tauri::command]
pub async fn get_watched_paths() -> Result<Vec<String>, CommandError> {
    #[cfg(not(test))]
    {
        let watcher_guard = FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_ref() {
            let paths = watcher.watched_paths().await;
            let path_strings = paths
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            Ok(path_strings)
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }

    #[cfg(test)]
    {
        let watcher_guard = TEST_FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_ref() {
            if let Some(watcher) =
                watcher.downcast_ref::<DocumentWatcher<tauri::test::MockRuntime>>()
            {
                let paths = watcher.watched_paths().await;
                let path_strings = paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect();
                Ok(path_strings)
            } else {
                Err(CommandError::Custom(
                    "File watcher type mismatch".to_string(),
                ))
            }
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }
}

#[tauri::command]
pub async fn add_watch_path(path: String) -> Result<(), CommandError> {
    #[cfg(not(test))]
    {
        let mut watcher_guard = FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_mut() {
            watcher
                .add_path(&path)
                .await
                .map_err(|e| CommandError::Custom(format!("Failed to add path: {}", e)))?;
            tracing::info!("Added path to watch: {}", path);
            Ok(())
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }

    #[cfg(test)]
    {
        let mut watcher_guard = TEST_FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_mut() {
            if let Some(watcher) =
                watcher.downcast_mut::<DocumentWatcher<tauri::test::MockRuntime>>()
            {
                watcher
                    .add_path(&path)
                    .await
                    .map_err(|e| CommandError::Custom(format!("Failed to add path: {}", e)))?;
                tracing::info!("Added path to watch: {}", path);
                Ok(())
            } else {
                Err(CommandError::Custom(
                    "File watcher type mismatch".to_string(),
                ))
            }
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }
}

#[tauri::command]
pub async fn remove_watch_path(path: String) -> Result<(), CommandError> {
    #[cfg(not(test))]
    {
        let mut watcher_guard = FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_mut() {
            watcher
                .remove_path(&path)
                .await
                .map_err(|e| CommandError::Custom(format!("Failed to remove path: {}", e)))?;
            tracing::info!("Removed path from watch: {}", path);
            Ok(())
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }

    #[cfg(test)]
    {
        let mut watcher_guard = TEST_FILE_WATCHER.lock().await;

        if let Some(watcher) = watcher_guard.as_mut() {
            if let Some(watcher) =
                watcher.downcast_mut::<DocumentWatcher<tauri::test::MockRuntime>>()
            {
                watcher
                    .remove_path(&path)
                    .await
                    .map_err(|e| CommandError::Custom(format!("Failed to remove path: {}", e)))?;
                tracing::info!("Removed path from watch: {}", path);
                Ok(())
            } else {
                Err(CommandError::Custom(
                    "File watcher type mismatch".to_string(),
                ))
            }
        } else {
            Err(CommandError::Custom("File watcher not running".to_string()))
        }
    }
}

#[tauri::command]
pub async fn emit_file_event(event: FileEvent) -> Result<(), CommandError> {
    // This command is used by the frontend to emit file events
    // that will be processed by the watcher system
    tracing::debug!("File event emitted: {:?}", event);
    Ok(())
}

// ---------------- Tests ----------------

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::tempdir;

    #[tokio::test]
    #[serial]
    async fn test_file_watcher_lifecycle() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let test_path = temp_dir.path();
        let test_path_str = test_path.to_string_lossy().to_string();

        // Create a mock AppHandle for testing
        let app = tauri::test::mock_app();
        let app_handle = app.handle().clone();

        // Test starting the watcher using the internal function that handles mock runtime
        let result = start_file_watching_test_internal(test_path_str.clone(), app_handle).await;
        assert!(
            result.is_ok(),
            "Failed to start file watching: {:?}",
            result
        );

        // Test getting watched paths
        let paths = get_watched_paths().await;
        assert!(paths.is_ok(), "Failed to get watched paths: {:?}", paths);
        let paths = paths.unwrap();
        let canonical_test_path = std::fs::canonicalize(test_path).unwrap();
        assert!(
            paths
                .iter()
                .any(|p| PathBuf::from(p) == canonical_test_path),
            "Test path not in watched paths. Watched: {:?}, Expected: {:?}",
            paths,
            canonical_test_path
        );

        // Test adding another path
        let another_path = temp_dir.path().join("subdir");
        std::fs::create_dir(&another_path).expect("Failed to create subdirectory");
        let another_path_str = another_path.to_string_lossy().to_string();
        let add_result = add_watch_path(another_path_str.clone()).await;
        assert!(
            add_result.is_ok(),
            "Failed to add watch path: {:?}",
            add_result
        );

        // Test getting updated paths
        let updated_paths = get_watched_paths().await.unwrap();
        assert_eq!(updated_paths.len(), 2, "Should have 2 watched paths");
        let canonical_another_path = std::fs::canonicalize(&another_path).unwrap();
        assert!(
            updated_paths
                .iter()
                .any(|p| PathBuf::from(p) == canonical_another_path),
            "Second path not found"
        );

        // Test removing a path
        let remove_result = remove_watch_path(another_path_str.clone()).await;
        assert!(
            remove_result.is_ok(),
            "Failed to remove watch path: {:?}",
            remove_result
        );

        // Test getting final paths
        let final_paths = get_watched_paths().await.unwrap();
        assert_eq!(
            final_paths.len(),
            1,
            "Should have 1 watched path after removal"
        );
        assert!(
            final_paths
                .iter()
                .any(|p| PathBuf::from(p) == canonical_test_path),
            "Original path not preserved"
        );

        // Test stopping the watcher
        let stop_result = stop_file_watching().await;
        assert!(
            stop_result.is_ok(),
            "Failed to stop file watching: {:?}",
            stop_result
        );

        // Test that watcher is no longer running
        let paths_after_stop = get_watched_paths().await;
        assert!(paths_after_stop.is_err(), "Watcher should be stopped");
    }

    #[tokio::test]
    #[serial]
    async fn test_pause_resume_watcher() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let test_path = temp_dir.path().to_string_lossy().to_string();

        // Create a mock AppHandle for testing
        let app = tauri::test::mock_app();
        let app_handle = app.handle().clone();

        // Start the watcher using the internal function that handles mock runtime
        start_file_watching_test_internal(test_path.clone(), app_handle)
            .await
            .unwrap();

        // Pause the watcher
        let pause_result = pause_file_watching().await;
        assert!(
            pause_result.is_ok(),
            "Failed to pause file watching: {:?}",
            pause_result
        );

        // Resume the watcher
        let resume_result = resume_file_watching().await;
        assert!(
            resume_result.is_ok(),
            "Failed to resume file watching: {:?}",
            resume_result
        );

        // Clean up
        stop_file_watching().await.unwrap();
    }

    #[tokio::test]
    async fn test_emit_file_event() {
        let event = FileEvent::Created(PathBuf::from("/test/path/file.txt"));

        let result = emit_file_event(event).await;
        assert!(result.is_ok(), "Failed to emit file event: {:?}", result);
    }
}
