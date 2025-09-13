// src-tauri/src/notifications.rs
// Notification system for UI updates

use crate::filesystem::watcher::{ConflictResult, FileEvent};
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use uuid::Uuid;

/// Notification types for UI updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Notification {
    FileChange(FileChangeNotification),
    Conflict(ConflictNotification),
    Security(SecurityNotification),
}

/// File change notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
    pub duration: Option<u64>,
    pub file_path: String,
    pub event_type: String,
    pub size: Option<u64>,
    pub is_directory: bool,
}

/// Conflict notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
    pub duration: Option<u64>,
    pub conflict_type: String,
    pub file_path: String,
    pub severity: String,
    pub resolution_required: bool,
    pub external_timestamp: Option<u64>,
    pub application_timestamp: Option<u64>,
    pub external_hash: Option<String>,
    pub application_hash: Option<String>,
}

/// Security notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityNotification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub timestamp: u64,
    pub duration: Option<u64>,
    pub security_level: String,
    pub operation: String,
    pub path: String,
    pub reason: String,
}

/// Notification emitter for sending notifications to frontend
pub struct NotificationEmitter {
    app_handle: tauri::AppHandle,
}

impl NotificationEmitter {
    /// Create a new notification emitter
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self { app_handle }
    }

    /// Emit a notification to the frontend
    pub fn emit(&self, notification: Notification) -> Result<(), tauri::Error> {
        self.app_handle.emit("notification", notification)
    }

    /// Create and emit a file change notification
    #[allow(dead_code)]
    pub fn emit_file_change(&self, event: &FileEvent) -> Result<(), tauri::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let (event_type, path, size, is_directory) = match event {
            FileEvent::Created(path) => {
                let metadata = std::fs::metadata(path).ok();
                (
                    "created",
                    path.to_string_lossy().to_string(),
                    metadata.as_ref().map(|m| m.len()),
                    metadata.map(|m| m.is_dir()).unwrap_or(false),
                )
            }
            FileEvent::Modified(path) => {
                let metadata = std::fs::metadata(path).ok();
                (
                    "modified",
                    path.to_string_lossy().to_string(),
                    metadata.as_ref().map(|m| m.len()),
                    metadata.map(|m| m.is_dir()).unwrap_or(false),
                )
            }
            FileEvent::Deleted(path) => (
                "deleted",
                path.to_string_lossy().to_string(),
                None,
                path.is_dir(),
            ),
            FileEvent::Renamed { from: _, to } => {
                let metadata = std::fs::metadata(to).ok();
                (
                    "renamed",
                    to.to_string_lossy().to_string(),
                    metadata.as_ref().map(|m| m.len()),
                    metadata.map(|m| m.is_dir()).unwrap_or(false),
                )
            }
            FileEvent::Moved { from: _, to } => {
                let metadata = std::fs::metadata(to).ok();
                (
                    "moved",
                    to.to_string_lossy().to_string(),
                    metadata.as_ref().map(|m| m.len()),
                    metadata.map(|m| m.is_dir()).unwrap_or(false),
                )
            }
        };

        let title = format!("File {}", event_type);
        let message = format!("File {}: {}", event_type, path);

        let notification = Notification::FileChange(FileChangeNotification {
            id: Uuid::new_v4().to_string(),
            title,
            message,
            timestamp,
            duration: Some(5000), // 5 seconds
            file_path: path,
            event_type: event_type.to_string(),
            size,
            is_directory,
        });

        self.emit(notification)
    }

    /// Create and emit a conflict notification
    #[allow(dead_code)]
    pub fn emit_conflict(&self, conflict: &ConflictResult) -> Result<(), tauri::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let conflict_type = match conflict.conflict_type {
            crate::filesystem::watcher::ConflictType::ExternalModification => {
                "external-modification"
            }
            crate::filesystem::watcher::ConflictType::ExternalDeletion => "external-deletion",
            crate::filesystem::watcher::ConflictType::ExternalCreationConflict => {
                "external-creation"
            }
            crate::filesystem::watcher::ConflictType::ContentConflict => "content",
            crate::filesystem::watcher::ConflictType::TimestampConflict => "timestamp",
        };

        let title = format!("File Conflict: {}", conflict_type);
        let message = format!(
            "Conflict detected in file: {} (Severity: {})",
            conflict.file_path.display(),
            conflict.severity
        );

        let notification = Notification::Conflict(ConflictNotification {
            id: Uuid::new_v4().to_string(),
            title,
            message,
            timestamp,
            duration: Some(10000), // 10 seconds for conflicts
            conflict_type: conflict_type.to_string(),
            file_path: conflict.file_path.to_string_lossy().to_string(),
            severity: conflict.severity.clone(),
            resolution_required: conflict.resolution_required,
            external_timestamp: conflict
                .external_timestamp
                .map(|ts| ts.timestamp_millis() as u64),
            application_timestamp: conflict
                .application_timestamp
                .map(|ts| ts.timestamp_millis() as u64),
            external_hash: conflict.external_hash.clone(),
            application_hash: conflict.application_hash.clone(),
        });

        self.emit(notification)
    }

    /// Create and emit a security notification
    pub fn emit_security(
        &self,
        security_level: &str,
        operation: &str,
        path: &std::path::Path,
        reason: &str,
    ) -> Result<(), tauri::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let title = format!("Security {}: {}", security_level, operation);
        let message = format!("Security {}: {} - {}", security_level, operation, reason);

        let notification = Notification::Security(SecurityNotification {
            id: Uuid::new_v4().to_string(),
            title,
            message,
            timestamp,
            duration: Some(8000), // 8 seconds for security notifications
            security_level: security_level.to_string(),
            operation: operation.to_string(),
            path: path.to_string_lossy().to_string(),
            reason: reason.to_string(),
        });

        self.emit(notification)
    }
}

// Import required modules
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

// Global notification emitter
static GLOBAL_EMITTER: Mutex<Option<NotificationEmitter>> = Mutex::new(None);

/// Set the global notification emitter
pub fn set_global_emitter(emitter: NotificationEmitter) {
    let mut global_emitter = GLOBAL_EMITTER.lock().unwrap();
    *global_emitter = Some(emitter);
}

/// Get the global notification emitter
pub fn get_global_emitter() -> Option<NotificationEmitter> {
    let global_emitter = GLOBAL_EMITTER.lock().unwrap();
    global_emitter.clone()
}

// Implement Clone for NotificationEmitter to allow cloning
impl Clone for NotificationEmitter {
    fn clone(&self) -> Self {
        Self {
            app_handle: self.app_handle.clone(),
        }
    }
}
