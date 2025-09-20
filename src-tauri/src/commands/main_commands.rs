// src-tauri/src/commands.rs

use crate::document::{
    BatchConfig, BatchHasher, BatchProcessingResult, BatchProcessor, ContentHash,
    CorruptionCheckResult, DuplicateCheckResult, EnhancedMetadata, FileProcessor,
    FileValidationResult, ImportError, ImportErrorInfo, ImportNotification,
    ImportNotificationManager, ImportProgress, MetadataExtractor, PersistedProgress,
    ProcessingPriority, ProgressManager, ProgressPersistenceManager, ProgressStorageStats,
    QueueStats,
};
use crate::filesystem::errors::SecurityError;
use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::backup_manager::BackupManager;
use crate::filesystem::security::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
use crate::filesystem::security::emergency_procedures::EmergencyManager;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::safe_mode::SafeModeManager;
use crate::filesystem::security::security_config::SecurityConfig;
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
pub async fn validate_file_comprehensive(
    path: String,
) -> Result<FileValidationResult, CommandError> {
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
static PROGRESS_MANAGER: Lazy<Mutex<ProgressManager>> =
    Lazy::new(|| Mutex::new(ProgressManager::new()));

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
        hasher
            .process_file(&validated_path)
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
    tracker
        .add_step(
            "validation".to_string(),
            "Validating files for import".to_string(),
        )
        .await;
    tracker
        .add_step(
            "processing".to_string(),
            "Processing file contents".to_string(),
        )
        .await;
    tracker
        .add_step("storage".to_string(), "Storing processed files".to_string())
        .await;
    tracker
        .add_step("indexing".to_string(), "Updating search index".to_string())
        .await;

    Ok(tracker.operation_id().to_string())
}

#[tauri::command]
pub async fn get_import_progress(
    operation_id: String,
) -> Result<Option<ImportProgress>, CommandError> {
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
    details: String,
) -> Result<(), CommandError> {
    let path = file_path.map(PathBuf::from);

    // Create appropriate error type based on error_type string
    let import_error = match error_type.as_str() {
        "file_not_found" => ImportError::FileNotFound {
            path: path.unwrap_or_else(|| PathBuf::from("unknown")),
        },
        "permission_denied" => ImportError::PermissionDenied {
            path: path.unwrap_or_else(|| PathBuf::from("unknown")),
        },
        "file_too_large" => ImportError::FileTooLarge {
            path: path.unwrap_or_else(|| PathBuf::from("unknown")),
            size: 0,     // Would be provided in real scenario
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
    file_count: Option<u32>,
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
            path: PathBuf::from("unknown"),
        }
    } else if error_message.contains("No such file") {
        ImportError::FileNotFound {
            path: PathBuf::from("unknown"),
        }
    } else if error_message.contains("too large") {
        ImportError::FileTooLarge {
            path: PathBuf::from("unknown"),
            size: 0,
            max_size: 0,
        }
    } else {
        ImportError::Unknown {
            details: error_message,
        }
    };

    Ok(import_error.to_error_info())
}

// ---------------- Progress Persistence Commands ----------------

#[tauri::command]
pub async fn initialize_progress_persistence(storage_dir: String) -> Result<(), CommandError> {
    let manager = ProgressPersistenceManager::new(&storage_dir).map_err(|e| {
        CommandError::Custom(format!("Failed to initialize progress persistence: {}", e))
    })?;

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
    let persistence = persistence_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    // Get current progress
    let progress_manager = PROGRESS_MANAGER.lock().await;
    let progress = progress_manager
        .get_operation_progress(&operation_id)
        .await
        .ok_or_else(|| CommandError::Custom("Operation not found".to_string()))?;

    // Convert string paths to PathBuf
    let remaining: Vec<PathBuf> = remaining_files.into_iter().map(PathBuf::from).collect();
    let processed: Vec<PathBuf> = processed_files.into_iter().map(PathBuf::from).collect();
    let failed: Vec<(PathBuf, String)> = failed_files
        .into_iter()
        .map(|(path, error)| (PathBuf::from(path), error))
        .collect();

    persistence
        .persist_progress(&operation_id, &progress, remaining, processed, failed)
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to persist progress: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn load_persisted_progress(
    operation_id: String,
) -> Result<Option<PersistedProgress>, CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence
        .load_progress(&operation_id)
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to load progress: {}", e)))
}

#[tauri::command]
pub async fn list_resumable_operations() -> Result<Vec<PersistedProgress>, CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence
        .get_resumable_operations()
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to get resumable operations: {}", e)))
}

#[tauri::command]
pub async fn mark_operation_completed(operation_id: String) -> Result<(), CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence
        .mark_completed(&operation_id)
        .await
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
    let persistence = persistence_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence
        .mark_file_completed(&operation_id, PathBuf::from(file_path), success, error)
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to mark file processed: {}", e)))?;

    Ok(())
}

#[tauri::command]
pub async fn cleanup_persisted_operations(max_age_hours: u64) -> Result<u32, CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    let max_age = std::time::Duration::from_secs(max_age_hours * 3600);
    persistence
        .cleanup_old_operations(max_age)
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to cleanup operations: {}", e)))
}

#[tauri::command]
pub async fn get_progress_storage_stats() -> Result<ProgressStorageStats, CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence
        .get_storage_stats()
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to get storage stats: {}", e)))
}

#[tauri::command]
pub async fn remove_persisted_progress(operation_id: String) -> Result<(), CommandError> {
    let persistence_guard = PROGRESS_PERSISTENCE.lock().await;
    let persistence = persistence_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Progress persistence not initialized".to_string()))?;

    persistence
        .remove_progress(&operation_id)
        .await
        .map_err(|e| CommandError::Custom(format!("Failed to remove progress: {}", e)))?;

    Ok(())
}

// ---------------- File Import Pipeline Commands ----------------

#[tauri::command]
pub async fn process_dropped_files(
    file_paths: Vec<String>,
    check_duplicates: Option<bool>,
    extract_metadata: Option<bool>,
) -> Result<Vec<serde_json::Value>, CommandError> {
    let start_time = Instant::now();
    let mut results = Vec::new();

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

    let check_duplicates = check_duplicates.unwrap_or(true);
    let extract_metadata = extract_metadata.unwrap_or(true);
    let validator = create_default_validator();

    for file_path in file_paths {
        let file_result = async {
            // 1. Validate file path and security
            let validated_path = validator
                .validate_import_path(Path::new(&file_path))
                .map_err(|e| format!("Path validation failed: {}", e))?;

            // 2. Comprehensive file validation
            let validation_result = FileProcessor::validate_file(&validated_path)
                .map_err(|e| format!("File validation failed: {}", e))?;

            if !validation_result.is_valid {
                return Err(format!("File validation failed: {}", validation_result.message));
            }

            // 3. Calculate hash for deduplication
            let content_hash = if check_duplicates {
                Some(ContentHash::from_file(&validated_path)
                    .map_err(|e| format!("Hash calculation failed: {}", e))?)
            } else {
                None
            };

            // 4. Check for duplicates if requested
            let duplicate_info = if check_duplicates {
                let mut hasher = BATCH_HASHER.lock().await;
                Some(hasher.process_file(&validated_path)
                    .map_err(|e| format!("Duplicate check failed: {}", e))?)
            } else {
                None
            };

            // 5. Extract metadata if requested
            let metadata = if extract_metadata {
                Some(MetadataExtractor::extract(&validated_path)
                    .map_err(|e| format!("Metadata extraction failed: {}", e))?)
            } else {
                None
            };

            // 6. Get file info
            let file_metadata = fs::metadata(&validated_path)
                .map_err(|e| format!("Failed to get file metadata: {}", e))?;

            // 7. Build result object
            let mut result = serde_json::json!({
                "path": validated_path.to_string_lossy(),
                "name": validated_path.file_name().unwrap_or_default().to_string_lossy(),
                "size": file_metadata.len(),
                "modified": file_metadata.modified().ok().map(|t| t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
                "validation": {
                    "is_valid": validation_result.is_valid,
                    "message": validation_result.message,
                    "warnings": Vec::<String>::new() // FileValidationResult doesn't have warnings field
                }
            });

            if let Some(hash) = content_hash {
                result["hash"] = serde_json::json!({
                    "sha256": hash.hash,
                    "algorithm": "sha256" // ContentHash doesn't have algorithm field, using static value
                });
            }

            if let Some(dup_info) = duplicate_info {
                result["duplicate_check"] = serde_json::json!({
                    "is_duplicate": dup_info.is_duplicate,
                    "existing_paths": dup_info.duplicates.iter().map(|d| d.hash.clone()).collect::<Vec<_>>() // Map duplicates to their paths
                });
            }

            if let Some(meta) = metadata {
                result["metadata"] = serde_json::json!({
                    "file_type": meta.basic.file_extension.clone().unwrap_or_default(),
                    "mime_type": meta.content.detected_mime_type.clone().unwrap_or_default(),
                    "page_count": meta.document.as_ref().and_then(|d| d.page_count),
                    "word_count": meta.content.stats.word_count,
                    "creation_date": meta.basic.created.map(|t| format!("{:?}", t)),
                    "author": meta.document.as_ref().and_then(|d| d.author.clone()),
                    "language": meta.content.language.clone(),
                    "binary_ratio": meta.content.stats.binary_ratio,
                    "entropy": meta.technical.entropy,
                    "text_preview": meta.content.preview.clone()
                });
            }

            Ok(result)
        }.await;

        match file_result {
            Ok(result) => results.push(result),
            Err(e) => {
                // Add error result
                results.push(serde_json::json!({
                    "path": file_path,
                    "error": e,
                    "validation": {
                        "is_valid": false,
                        "message": e.clone(),
                        "warnings": Vec::<String>::new()
                    }
                }));
            }
        }
    }

    let duration = start_time.elapsed();
    VALIDATION_METRICS.record_success(duration);

    Ok(results)
}

/// Open file dialog for selecting files to import
#[tauri::command]
pub async fn open_file_dialog() -> Result<Option<Vec<String>>, String> {
    use std::process::Command;

    // Use zenity for file dialog on Linux
    let output = Command::new("zenity")
        .arg("--file-selection")
        .arg("--multiple")
        .arg("--file-filter=Supported files | *.docx *.pdf *.md *.txt *.csv *.json")
        .arg("--title=Select files to import")
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
                let paths: Vec<String> = stdout
                    .trim()
                    .split('|')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                if paths.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(paths))
                }
            } else {
                // User cancelled or error occurred
                Ok(None)
            }
        }
        Err(e) => Err(format!("Failed to open file dialog: {}", e)),
    }
}

// ---------------- Batch Processing Commands ----------------

// Global batch processor instance
#[allow(dead_code)]
static BATCH_PROCESSOR: Lazy<tokio::sync::Mutex<Option<BatchProcessor>>> =
    Lazy::new(|| tokio::sync::Mutex::new(None));

#[allow(dead_code)]
#[tauri::command]
pub async fn initialize_batch_processor(
    max_parallel_files: Option<usize>,
    max_concurrent_bytes: Option<u64>,
    file_timeout_seconds: Option<u64>,
    continue_on_failure: Option<bool>,
    max_retries: Option<u32>,
) -> Result<(), CommandError> {
    let mut config = BatchConfig::default();

    if let Some(max_parallel) = max_parallel_files {
        config.max_parallel_files = max_parallel.clamp(1, 16); // Cap at reasonable limits
    }

    if let Some(max_bytes) = max_concurrent_bytes {
        config.max_concurrent_bytes = max_bytes.max(64 * 1024 * 1024); // Minimum 64MB
    }

    if let Some(timeout) = file_timeout_seconds {
        config.file_timeout = std::time::Duration::from_secs(timeout.clamp(10, 3600));
        // 10s to 1h
    }

    if let Some(continue_fail) = continue_on_failure {
        config.continue_on_failure = continue_fail;
    }

    if let Some(retries) = max_retries {
        config.max_retries = retries.min(10); // Cap at 10 retries
    }

    let validator = create_default_validator();
    let processor = BatchProcessor::with_config(config, validator);

    let mut batch_processor_guard = BATCH_PROCESSOR.lock().await;
    *batch_processor_guard = Some(processor);

    Ok(())
}

#[allow(dead_code)]
#[tauri::command]
pub async fn process_files_batch(
    file_paths: Vec<String>,
    priority: String,
    operation_id: String,
) -> Result<BatchProcessingResult, CommandError> {
    let batch_processor_guard = BATCH_PROCESSOR.lock().await;
    let processor = batch_processor_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Batch processor not initialized".to_string()))?;

    // Convert priority string to enum
    let processing_priority = match priority.as_str() {
        "low" => ProcessingPriority::Low,
        "normal" => ProcessingPriority::Normal,
        "high" => ProcessingPriority::High,
        "critical" => ProcessingPriority::Critical,
        _ => ProcessingPriority::Normal,
    };

    // Convert string paths to PathBuf
    let files: Vec<PathBuf> = file_paths.into_iter().map(PathBuf::from).collect();
    let file_count = files.len() as u64;

    // Get or create progress tracker
    let progress_manager = PROGRESS_MANAGER.lock().await;
    let tracker = if let Some(_existing_progress) =
        progress_manager.get_operation_progress(&operation_id).await
    {
        // Use existing tracker - get it from the manager
        // For now, create a new one as we can't easily extract the tracker
        progress_manager.start_operation(file_count).await
    } else {
        progress_manager.start_operation(file_count).await
    };

    // Release the progress manager lock before long-running operation
    drop(progress_manager);

    // Process the batch
    let result = processor
        .process_batch(files, processing_priority, tracker.clone())
        .await
        .map_err(|e| CommandError::Custom(format!("Batch processing failed: {}", e)))?;

    Ok(result)
}

#[allow(dead_code)]
#[tauri::command]
pub async fn get_batch_processing_stats() -> Result<QueueStats, CommandError> {
    let batch_processor_guard = BATCH_PROCESSOR.lock().await;
    let processor = batch_processor_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Batch processor not initialized".to_string()))?;

    Ok(processor.get_stats().await)
}

#[allow(dead_code)]
#[tauri::command]
pub async fn clear_batch_processing_queue() -> Result<(), CommandError> {
    let batch_processor_guard = BATCH_PROCESSOR.lock().await;
    let processor = batch_processor_guard
        .as_ref()
        .ok_or_else(|| CommandError::Custom("Batch processor not initialized".to_string()))?;

    processor.clear_queue().await;
    Ok(())
}

#[allow(dead_code)]
#[tauri::command]
pub async fn process_single_file_with_validation(
    file_path: String,
    retry_count: Option<u32>,
) -> Result<bool, CommandError> {
    let start_time = Instant::now();

    // Security checks
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

    if !SafeModeManager::global()
        .is_file_allowed(Path::new(&file_path))
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

    let validator = create_default_validator();
    let path = Path::new(&file_path);
    let retry_count = retry_count.unwrap_or(0);

    // Use the batch processor's internal method (but we need to make it public)
    // For now, replicate the logic
    let result = async {
        // Validate path security first
        let validated_path = validator
            .validate_import_path(path)
            .map_err(|e| anyhow::anyhow!("Path validation failed: {}", e))?;

        // Validate file integrity and structure
        let validation_result = FileProcessor::validate_file(&validated_path)
            .map_err(|e| anyhow::anyhow!("File validation failed: {}", e))?;

        if !validation_result.is_valid {
            anyhow::bail!("File validation failed: {}", validation_result.message);
        }

        // Calculate content hash for deduplication
        let _hash = ContentHash::from_file(&validated_path)
            .map_err(|e| anyhow::anyhow!("Hash calculation failed: {}", e))?;

        Ok(())
    }
    .await;

    let duration = start_time.elapsed();
    match &result {
        Ok(_) => {
            VALIDATION_METRICS.record_success(duration);
            Ok(true)
        }
        Err(e) => {
            VALIDATION_METRICS.record_failure(duration);
            Err(CommandError::Custom(format!(
                "File processing failed (attempt {}): {}",
                retry_count + 1,
                e
            )))
        }
    }
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

// ---------------- Import Wizard Batch Processing Commands ----------------

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct ImportConfig {
    pub preset: String,
    pub settings: crate::workspace::ImportSettings,
    pub ai_settings: crate::workspace::WorkspaceAISettings,
    pub workspace_path: String,
}

#[tauri::command]
pub async fn batch_import_files(
    files: Vec<String>,
    config: ImportConfig,
) -> Result<BatchProcessingResult, CommandError> {
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

    if !emergency_manager.can_perform_operation("write") {
        return Err(CommandError::SecurityError {
            message: "Operation blocked by emergency restrictions".to_string(),
            code: "EMERGENCY_BLOCKED".to_string(),
            severity: "high".to_string(),
        });
    }

    // Use circuit breaker for batch operations
    let _breaker = CIRCUIT_BREAKER_MANAGER.get_or_create(
        "batch_import",
        Some(CircuitBreakerConfig {
            failure_threshold: 5,
            recovery_timeout: std::time::Duration::from_secs(60),
            success_threshold: 3,
        }),
    );

    // Create validator with current security settings
    let validator = create_default_validator();

    // Convert string paths to PathBuf
    let file_paths: Vec<PathBuf> = files.iter().map(PathBuf::from).collect();

    // Create batch configuration based on import settings
    let batch_config = BatchConfig {
        max_parallel_files: 4,                             // Conservative default
        max_concurrent_bytes: 256 * 1024 * 1024,           // 256MB
        file_timeout: std::time::Duration::from_secs(300), // 5 minutes per file
        batch_delay: std::time::Duration::from_millis(100),
        continue_on_failure: true,
        max_retries: 2,
        retry_delay: std::time::Duration::from_secs(1),
    };

    // Create batch processor
    let processor = BatchProcessor::with_config(batch_config, validator);

    // Start progress tracking
    let progress_manager = PROGRESS_MANAGER.lock().await;
    let tracker = progress_manager
        .start_operation(file_paths.len() as u64)
        .await;

    // Add processing steps
    tracker
        .add_step(
            "validation".to_string(),
            "Validating files for import".to_string(),
        )
        .await;
    tracker
        .add_step(
            "processing".to_string(),
            "Processing file contents".to_string(),
        )
        .await;
    tracker
        .add_step(
            "storage".to_string(),
            "Storing files in workspace".to_string(),
        )
        .await;

    // Process the batch with normal priority
    let processing_result = processor
        .process_batch(file_paths, ProcessingPriority::Normal, tracker.clone())
        .await
        .map_err(|e| CommandError::Custom(format!("Batch processing failed: {}", e)))?;

    // Notify success or partial success
    let notification_manager = NOTIFICATION_MANAGER.lock().await;
    if processing_result.failed_files == 0 {
        notification_manager
            .notify_success(
                "Import Completed".to_string(),
                format!(
                    "Successfully imported {} files using {} preset",
                    processing_result.successful_files, config.preset
                ),
                Some(processing_result.successful_files as u32),
            )
            .await;
    } else if processing_result.successful_files > 0 {
        notification_manager
            .notify_info(
                "Import Partially Completed".to_string(),
                format!(
                    "Imported {} files, {} failed using {} preset",
                    processing_result.successful_files,
                    processing_result.failed_files,
                    config.preset
                ),
            )
            .await;
    } else {
        // Create error info with correct structure
        let import_error = ImportError::Unknown {
            details: format!(
                "All {} files failed to import",
                processing_result.total_files
            ),
        };
        let error_info = import_error.to_error_info();
        notification_manager.notify_error(&error_info).await;
    }

    let duration = start_time.elapsed();

    // Record metrics
    if processing_result.failed_files == 0 {
        VALIDATION_METRICS.record_success(duration);
    } else {
        VALIDATION_METRICS.record_failure(duration);
    }

    Ok(processing_result)
}

#[tauri::command]
pub async fn validate_import_files(
    files: Vec<String>,
    settings: crate::workspace::ImportSettings,
) -> Result<Vec<FileValidationResult>, CommandError> {
    let validator = create_default_validator();
    let mut results = Vec::new();

    for file_path in files {
        let path = PathBuf::from(&file_path);

        // Validate path security first
        match validator.validate_import_path(&path) {
            Ok(validated_path) => {
                // Validate file against import settings
                match FileProcessor::validate_file(&validated_path) {
                    Ok(mut validation_result) => {
                        // Additional checks based on import settings
                        let file_size = std::fs::metadata(&validated_path)
                            .map(|m| m.len())
                            .unwrap_or(0);

                        if file_size > settings.max_file_size {
                            validation_result.is_valid = false;
                            validation_result.message = format!(
                                "File size ({} bytes) exceeds maximum allowed size ({} bytes)",
                                file_size, settings.max_file_size
                            );
                        }

                        // Check file extension
                        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                            let ext_with_dot = format!(".{}", extension.to_lowercase());
                            if !settings.allowed_extensions.contains(&ext_with_dot) {
                                validation_result.is_valid = false;
                                validation_result.message = format!(
                                    "File extension '{}' not allowed. Allowed: {}",
                                    ext_with_dot,
                                    settings.allowed_extensions.join(", ")
                                );
                            }
                        }

                        results.push(validation_result);
                    }
                    Err(e) => {
                        results.push(FileValidationResult {
                            is_valid: false,
                            status: crate::document::ValidationStatus::Invalid,
                            message: format!("Validation failed: {}", e),
                            corruption_check: None,
                        });
                    }
                }
            }
            Err(e) => {
                results.push(FileValidationResult {
                    is_valid: false,
                    status: crate::document::ValidationStatus::Invalid,
                    message: format!("Path validation failed: {}", e),
                    corruption_check: None,
                });
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_import_preset_templates() -> Result<Vec<String>, CommandError> {
    // Return available workspace templates for import presets
    Ok(vec![
        "Basic".to_string(),
        "Research".to_string(),
        "Documentation".to_string(),
        "Collaboration".to_string(),
    ])
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
