// src-tauri/src/document/import_errors.rs
// Comprehensive error handling and user notification system for import operations

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Comprehensive error types for import operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ImportError {
    #[error("File not found: {path}")]
    FileNotFound { path: PathBuf },

    #[error("Permission denied accessing file: {path}")]
    PermissionDenied { path: PathBuf },

    #[error("File is too large: {path} ({size} bytes, max: {max_size} bytes)")]
    FileTooLarge {
        path: PathBuf,
        size: u64,
        max_size: u64,
    },

    #[error("File type not supported: {path} (detected: {detected_type:?}, expected: {expected_types:?})")]
    UnsupportedFileType {
        path: PathBuf,
        detected_type: Option<String>,
        expected_types: Vec<String>,
    },

    #[error("File appears to be corrupted: {path} - {details}")]
    FileCorrupted { path: PathBuf, details: String },

    #[error("Duplicate file detected: {path} (matches: {existing_path})")]
    DuplicateFile {
        path: PathBuf,
        existing_path: PathBuf,
    },

    #[error("File is empty: {path}")]
    EmptyFile { path: PathBuf },

    #[error("Path contains invalid characters: {path}")]
    InvalidPath { path: PathBuf },

    #[error("Filename is invalid: {filename}")]
    InvalidFilename { filename: String },

    #[error("Operation was cancelled by user")]
    OperationCancelled,

    #[error("Insufficient disk space: required {required} bytes, available {available} bytes")]
    InsufficientSpace { required: u64, available: u64 },

    #[error("Network error during file access: {details}")]
    NetworkError { details: String },

    #[error("File is locked by another process: {path}")]
    FileLocked { path: PathBuf },

    #[error("Security validation failed: {path} - {reason}")]
    SecurityViolation { path: PathBuf, reason: String },

    #[error("Metadata extraction failed: {path} - {reason}")]
    MetadataExtractionFailed { path: PathBuf, reason: String },

    #[error("Hash calculation failed: {path} - {reason}")]
    HashCalculationFailed { path: PathBuf, reason: String },

    #[error("Database operation failed: {operation} - {reason}")]
    DatabaseError { operation: String, reason: String },

    #[error("System resource limit exceeded: {resource} - {details}")]
    ResourceLimitExceeded { resource: String, details: String },

    #[error("Temporary file operation failed: {reason}")]
    TemporaryFileError { reason: String },

    #[error("Backup operation failed: {reason}")]
    BackupFailed { reason: String },

    #[error("Unknown error occurred: {details}")]
    Unknown { details: String },
}

/// Error severity levels for UI presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Low severity - warnings that don't prevent operation
    Warning,
    /// Medium severity - errors that affect individual files but allow batch to continue
    Error,
    /// High severity - critical errors that may require stopping the operation
    Critical,
    /// Fatal - errors that prevent any further processing
    Fatal,
}

/// Error category for grouping and filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    FileSystem,
    Security,
    Validation,
    Network,
    Resource,
    UserCancellation,
    System,
}

/// Comprehensive error information for user presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportErrorInfo {
    /// The underlying error
    pub error: ImportError,
    /// Severity level
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// User-friendly title
    pub title: String,
    /// Detailed user-friendly message
    pub message: String,
    /// Suggested actions for the user
    pub suggested_actions: Vec<String>,
    /// Whether this error can be retried
    pub retryable: bool,
    /// Whether this error should stop batch processing
    pub should_stop_batch: bool,
    /// Technical details for troubleshooting
    pub technical_details: Option<String>,
    /// Timestamp when error occurred
    pub timestamp: std::time::SystemTime,
    /// File path associated with error (if any)
    pub file_path: Option<PathBuf>,
}

impl ImportError {
    /// Convert error to comprehensive error info for user presentation
    pub fn to_error_info(&self) -> ImportErrorInfo {
        let (severity, category, title, message, suggested_actions, retryable, should_stop_batch) =
            self.get_error_details();

        ImportErrorInfo {
            error: self.clone(),
            severity,
            category,
            title,
            message,
            suggested_actions,
            retryable,
            should_stop_batch,
            technical_details: Some(format!("{:?}", self)),
            timestamp: std::time::SystemTime::now(),
            file_path: self.get_file_path(),
        }
    }

    /// Get file path from error if available
    pub fn get_file_path(&self) -> Option<PathBuf> {
        match self {
            ImportError::FileNotFound { path }
            | ImportError::PermissionDenied { path }
            | ImportError::FileTooLarge { path, .. }
            | ImportError::UnsupportedFileType { path, .. }
            | ImportError::FileCorrupted { path, .. }
            | ImportError::DuplicateFile { path, .. }
            | ImportError::EmptyFile { path }
            | ImportError::InvalidPath { path }
            | ImportError::FileLocked { path }
            | ImportError::SecurityViolation { path, .. }
            | ImportError::MetadataExtractionFailed { path, .. }
            | ImportError::HashCalculationFailed { path, .. } => Some(path.clone()),
            _ => None,
        }
    }

    /// Get detailed error information for user presentation
    fn get_error_details(
        &self,
    ) -> (
        ErrorSeverity,
        ErrorCategory,
        String,
        String,
        Vec<String>,
        bool,
        bool,
    ) {
        match self {
            ImportError::FileNotFound { path } => (
                ErrorSeverity::Error,
                ErrorCategory::FileSystem,
                "File Not Found".to_string(),
                format!("The file '{}' could not be found. It may have been moved, deleted, or the path is incorrect.", path.display()),
                vec![
                    "Verify the file path is correct".to_string(),
                    "Check if the file has been moved or renamed".to_string(),
                    "Ensure you have access to the directory".to_string(),
                ],
                false,
                false,
            ),

            ImportError::PermissionDenied { path } => (
                ErrorSeverity::Error,
                ErrorCategory::Security,
                "Permission Denied".to_string(),
                format!("Access to the file '{}' was denied. You may not have sufficient permissions.", path.display()),
                vec![
                    "Check file permissions".to_string(),
                    "Run as administrator if necessary".to_string(),
                    "Ensure the file is not being used by another application".to_string(),
                ],
                true,
                false,
            ),

            ImportError::FileTooLarge { path, size, max_size } => (
                ErrorSeverity::Warning,
                ErrorCategory::Validation,
                "File Too Large".to_string(),
                format!("The file '{}' is too large ({} MB). Maximum allowed size is {} MB.",
                    path.display(),
                    *size / 1024 / 1024,
                    *max_size / 1024 / 1024
                ),
                vec![
                    "Split the file into smaller parts".to_string(),
                    "Compress the file to reduce size".to_string(),
                    "Contact administrator to increase size limits".to_string(),
                ],
                false,
                false,
            ),

            ImportError::UnsupportedFileType { path, detected_type, expected_types } => (
                ErrorSeverity::Warning,
                ErrorCategory::Validation,
                "Unsupported File Type".to_string(),
                format!("The file '{}' has an unsupported format{}. Supported formats: {}.",
                    path.display(),
                    if let Some(detected) = detected_type {
                        format!(" (detected: {})", detected)
                    } else {
                        String::new()
                    },
                    expected_types.join(", ")
                ),
                vec![
                    "Convert the file to a supported format".to_string(),
                    "Check if the file extension is correct".to_string(),
                ],
                false,
                false,
            ),

            ImportError::FileCorrupted { path, details } => (
                ErrorSeverity::Error,
                ErrorCategory::Validation,
                "File Corrupted".to_string(),
                format!("The file '{}' appears to be corrupted and cannot be processed. {}", path.display(), details),
                vec![
                    "Try opening the file in its native application".to_string(),
                    "Restore from a backup if available".to_string(),
                    "Re-download or re-create the file".to_string(),
                ],
                false,
                false,
            ),

            ImportError::DuplicateFile { path, existing_path } => (
                ErrorSeverity::Warning,
                ErrorCategory::Validation,
                "Duplicate File Detected".to_string(),
                format!("The file '{}' is identical to an existing file '{}'.", path.display(), existing_path.display()),
                vec![
                    "Skip this file".to_string(),
                    "Replace the existing file".to_string(),
                    "Import with a different name".to_string(),
                ],
                false,
                false,
            ),

            ImportError::OperationCancelled => (
                ErrorSeverity::Warning,
                ErrorCategory::UserCancellation,
                "Operation Cancelled".to_string(),
                "The import operation was cancelled by the user.".to_string(),
                vec![
                    "Restart the import if needed".to_string(),
                ],
                true,
                true,
            ),

            ImportError::InsufficientSpace { required, available } => (
                ErrorSeverity::Critical,
                ErrorCategory::Resource,
                "Insufficient Disk Space".to_string(),
                format!("Not enough disk space available. Required: {} MB, Available: {} MB.",
                    *required / 1024 / 1024,
                    *available / 1024 / 1024
                ),
                vec![
                    "Free up disk space by deleting unnecessary files".to_string(),
                    "Move files to a different drive with more space".to_string(),
                    "Use disk cleanup tools".to_string(),
                ],
                true,
                true,
            ),

            ImportError::NetworkError { details } => (
                ErrorSeverity::Error,
                ErrorCategory::Network,
                "Network Error".to_string(),
                format!("A network error occurred while accessing the file: {}", details),
                vec![
                    "Check your network connection".to_string(),
                    "Verify the network path is accessible".to_string(),
                    "Try again after a few moments".to_string(),
                ],
                true,
                false,
            ),

            ImportError::SecurityViolation { path, reason } => (
                ErrorSeverity::Critical,
                ErrorCategory::Security,
                "Security Violation".to_string(),
                format!("Security check failed for file '{}': {}", path.display(), reason),
                vec![
                    "Verify the file is from a trusted source".to_string(),
                    "Contact your administrator".to_string(),
                ],
                false,
                true,
            ),

            _ => (
                ErrorSeverity::Error,
                ErrorCategory::System,
                "Import Error".to_string(),
                format!("An error occurred during import: {}", self),
                vec![
                    "Try the operation again".to_string(),
                    "Contact support if the problem persists".to_string(),
                ],
                true,
                false,
            ),
        }
    }
}

/// Notification types for user alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationType {
    Info,
    Warning,
    Error,
    Success,
}

/// User notification for import operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportNotification {
    /// Notification type
    pub notification_type: NotificationType,
    /// Title for the notification
    pub title: String,
    /// Message content
    pub message: String,
    /// Whether notification should be persistent
    pub persistent: bool,
    /// Duration to show notification (milliseconds)
    pub duration_ms: Option<u32>,
    /// Actions available to user
    pub actions: Vec<NotificationAction>,
    /// Associated file path (if any)
    pub file_path: Option<PathBuf>,
    /// Timestamp
    pub timestamp: std::time::SystemTime,
}

/// Action available in a notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Action identifier
    pub id: String,
    /// Display label
    pub label: String,
    /// Action type
    pub action_type: ActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    /// Dismiss the notification
    Dismiss,
    /// Retry the failed operation
    Retry,
    /// Open file location
    OpenLocation,
    /// View details
    ViewDetails,
    /// Custom action
    Custom { command: String },
}

/// Notification manager for import operations
pub struct ImportNotificationManager {
    notifications: std::sync::Arc<tokio::sync::RwLock<Vec<ImportNotification>>>,
}

impl ImportNotificationManager {
    /// Create a new notification manager
    pub fn new() -> Self {
        Self {
            notifications: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Create notification from error
    pub async fn notify_error(&self, error_info: &ImportErrorInfo) {
        let notification_type = match error_info.severity {
            ErrorSeverity::Warning => NotificationType::Warning,
            ErrorSeverity::Error => NotificationType::Error,
            ErrorSeverity::Critical | ErrorSeverity::Fatal => NotificationType::Error,
        };

        let mut actions = vec![NotificationAction {
            id: "dismiss".to_string(),
            label: "Dismiss".to_string(),
            action_type: ActionType::Dismiss,
        }];

        if error_info.retryable {
            actions.push(NotificationAction {
                id: "retry".to_string(),
                label: "Retry".to_string(),
                action_type: ActionType::Retry,
            });
        }

        if error_info.file_path.is_some() {
            actions.push(NotificationAction {
                id: "open_location".to_string(),
                label: "Open Location".to_string(),
                action_type: ActionType::OpenLocation,
            });
        }

        let notification = ImportNotification {
            notification_type,
            title: error_info.title.clone(),
            message: error_info.message.clone(),
            persistent: matches!(
                error_info.severity,
                ErrorSeverity::Critical | ErrorSeverity::Fatal
            ),
            duration_ms: Some(match error_info.severity {
                ErrorSeverity::Warning => 5000,
                ErrorSeverity::Error => 8000,
                ErrorSeverity::Critical | ErrorSeverity::Fatal => 10000,
            }),
            actions,
            file_path: error_info.file_path.clone(),
            timestamp: std::time::SystemTime::now(),
        };

        self.add_notification(notification).await;
    }

    /// Create success notification
    pub async fn notify_success(&self, title: String, message: String, file_count: Option<u32>) {
        let notification = ImportNotification {
            notification_type: NotificationType::Success,
            title,
            message: if let Some(count) = file_count {
                format!("{} ({} files processed)", message, count)
            } else {
                message
            },
            persistent: false,
            duration_ms: Some(4000),
            actions: vec![NotificationAction {
                id: "dismiss".to_string(),
                label: "Dismiss".to_string(),
                action_type: ActionType::Dismiss,
            }],
            file_path: None,
            timestamp: std::time::SystemTime::now(),
        };

        self.add_notification(notification).await;
    }

    /// Create info notification
    pub async fn notify_info(&self, title: String, message: String) {
        let notification = ImportNotification {
            notification_type: NotificationType::Info,
            title,
            message,
            persistent: false,
            duration_ms: Some(3000),
            actions: vec![NotificationAction {
                id: "dismiss".to_string(),
                label: "Dismiss".to_string(),
                action_type: ActionType::Dismiss,
            }],
            file_path: None,
            timestamp: std::time::SystemTime::now(),
        };

        self.add_notification(notification).await;
    }

    /// Add notification to queue
    async fn add_notification(&self, notification: ImportNotification) {
        let mut notifications = self.notifications.write().await;
        notifications.push(notification);

        // Keep only the last 50 notifications to prevent memory bloat
        if notifications.len() > 50 {
            let excess = notifications.len() - 50;
            notifications.drain(..excess);
        }
    }

    /// Get all notifications
    pub async fn get_notifications(&self) -> Vec<ImportNotification> {
        self.notifications.read().await.clone()
    }

    /// Clear all notifications
    pub async fn clear_notifications(&self) {
        let mut notifications = self.notifications.write().await;
        notifications.clear();
    }

    /// Remove specific notification
    pub async fn remove_notification(&self, index: usize) {
        let mut notifications = self.notifications.write().await;
        if index < notifications.len() {
            notifications.remove(index);
        }
    }
}

impl Default for ImportNotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_error_info_creation() {
        let error = ImportError::FileNotFound {
            path: PathBuf::from("/test/file.txt"),
        };

        let error_info = error.to_error_info();
        assert_eq!(error_info.title, "File Not Found");
        assert!(matches!(error_info.severity, ErrorSeverity::Error));
        assert!(matches!(error_info.category, ErrorCategory::FileSystem));
        assert!(!error_info.retryable);
        assert!(!error_info.should_stop_batch);
    }

    #[test]
    fn test_file_path_extraction() {
        let path = PathBuf::from("/test/file.txt");
        let error = ImportError::PermissionDenied { path: path.clone() };
        assert_eq!(error.get_file_path(), Some(path));

        let error = ImportError::OperationCancelled;
        assert_eq!(error.get_file_path(), None);
    }

    #[tokio::test]
    async fn test_notification_manager() {
        let manager = ImportNotificationManager::new();

        // Test error notification
        let error = ImportError::FileTooLarge {
            path: PathBuf::from("/test/large.txt"),
            size: 100_000_000,
            max_size: 50_000_000,
        };
        let error_info = error.to_error_info();
        manager.notify_error(&error_info).await;

        // Test success notification
        manager
            .notify_success(
                "Import Complete".to_string(),
                "Files imported successfully".to_string(),
                Some(5),
            )
            .await;

        let notifications = manager.get_notifications().await;
        assert_eq!(notifications.len(), 2);
        assert!(matches!(
            notifications[0].notification_type,
            NotificationType::Warning
        ));
        assert!(matches!(
            notifications[1].notification_type,
            NotificationType::Success
        ));
    }
}
