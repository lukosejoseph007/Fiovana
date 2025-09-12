// src-tauri/src/filesystem/watcher.rs
// File watcher system with debouncing, filtering, and security integration

use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use notify::event::{ModifyKind, RenameMode};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tracing;

/// File event types that can be emitted by the watcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
    Moved { from: PathBuf, to: PathBuf },
}

impl FileEvent {
    /// Get the path associated with this event
    pub fn path(&self) -> &Path {
        match self {
            FileEvent::Created(path) => path,
            FileEvent::Modified(path) => path,
            FileEvent::Deleted(path) => path,
            FileEvent::Renamed { to, .. } => to,
            FileEvent::Moved { to, .. } => to,
        }
    }

    /// Convert Rust FileEvent to TypeScript FrontendFileEvent
    #[allow(dead_code)]
    pub fn to_frontend_event(&self) -> std::result::Result<FrontendFileEvent, std::io::Error> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(std::io::Error::other)?
            .as_millis() as u64;

        let (event_type, path, old_path, size, is_directory) = match self {
            FileEvent::Created(path) => {
                let metadata = std::fs::metadata(path)?;
                (
                    "file-created",
                    path.to_string_lossy().to_string(),
                    None,
                    Some(metadata.len()),
                    metadata.is_dir(),
                )
            }
            FileEvent::Modified(path) => {
                let metadata = std::fs::metadata(path)?;
                (
                    "file-modified",
                    path.to_string_lossy().to_string(),
                    None,
                    Some(metadata.len()),
                    metadata.is_dir(),
                )
            }
            FileEvent::Deleted(path) => {
                // For deleted files, we can't get metadata, so use defaults
                (
                    "file-deleted",
                    path.to_string_lossy().to_string(),
                    None,
                    None,
                    path.is_dir(),
                )
            }
            FileEvent::Renamed { from, to } => {
                let metadata = std::fs::metadata(to)?;
                (
                    "file-renamed",
                    to.to_string_lossy().to_string(),
                    Some(from.to_string_lossy().to_string()),
                    Some(metadata.len()),
                    metadata.is_dir(),
                )
            }
            FileEvent::Moved { from, to } => {
                let metadata = std::fs::metadata(to)?;
                (
                    "file-moved",
                    to.to_string_lossy().to_string(),
                    Some(from.to_string_lossy().to_string()),
                    Some(metadata.len()),
                    metadata.is_dir(),
                )
            }
        };

        Ok(FrontendFileEvent {
            type_: event_type.to_string(),
            path,
            old_path,
            timestamp,
            size,
            is_directory,
        })
    }
}

/// TypeScript-compatible file event structure for frontend consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendFileEvent {
    #[serde(rename = "type")]
    pub type_: String,
    pub path: String,
    pub old_path: Option<String>,
    pub timestamp: u64,
    pub size: Option<u64>,
    pub is_directory: bool,
}

/// Configuration for the file watcher
#[derive(Debug, Clone)]
pub struct WatcherConfig {
    pub debounce_duration: Duration,
    pub security_config: SecurityConfig,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_millis(500),
            security_config: SecurityConfig::default(),
        }
    }
}

/// Main file watcher that manages watching directories with security integration
pub struct DocumentWatcher {
    watcher: Option<RecommendedWatcher>,
    event_sender: mpsc::UnboundedSender<FileEvent>,
    watched_paths: Arc<RwLock<Vec<PathBuf>>>,
    config: WatcherConfig,
    is_paused: Arc<RwLock<bool>>,
    path_validator: PathValidator,
}

impl DocumentWatcher {
    /// Create a new DocumentWatcher with the given configuration
    pub fn new(config: WatcherConfig) -> (Self, mpsc::UnboundedReceiver<FileEvent>) {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        // Create path validator with security config
        let path_validator = PathValidator::new(
            config.security_config.clone(),
            vec![
                dirs::desktop_dir().unwrap_or_default(),
                dirs::document_dir().unwrap_or_default(),
                dirs::download_dir().unwrap_or_default(),
                std::env::temp_dir(),
            ],
        );

        (
            Self {
                watcher: None,
                event_sender,
                watched_paths: Arc::new(RwLock::new(Vec::new())),
                config,
                is_paused: Arc::new(RwLock::new(false)),
                path_validator,
            },
            event_receiver,
        )
    }

    /// Start the file watcher
    pub async fn start(&mut self) -> Result<()> {
        let (tx, rx) = crossbeam_channel::bounded(1000);
        let event_sender = self.event_sender.clone();
        let config = self.config.clone();
        let is_paused = self.is_paused.clone();
        let path_validator = self.path_validator.clone();

        // Spawn debounced event processor
        tokio::spawn(async move {
            let mut debouncer = EventDebouncer::new(config.debounce_duration);

            while let Ok(Ok(event)) = tokio::task::spawn_blocking({
                let rx = rx.clone();
                move || rx.recv()
            })
            .await
            {
                if *is_paused.read().await {
                    continue;
                }

                if let Some(processed_events) =
                    debouncer.process_event(event, &path_validator).await
                {
                    for file_event in processed_events {
                        // Log security audit event
                        SecurityAuditor::log_file_access_attempt(
                            file_event.path(),
                            "file_watch",
                            &Ok(file_event.path().to_path_buf()),
                            "development",
                            None::<uuid::Uuid>,
                        );

                        if event_sender.send(file_event).is_err() {
                            // If the receiver is dropped, stop the loop
                            break;
                        }
                    }
                }
            }
        });

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event>| {
                if let Ok(event) = res {
                    if let Err(e) = tx.try_send(event) {
                        tracing::warn!("Failed to send file event for processing: {}", e);
                    }
                }
            },
            Config::default(),
        )?;

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Add a path to watch with security validation
    pub async fn add_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Validate path using directory validation (not import validation)
        let validated_path = self
            .path_validator
            .validate_directory_path(path)
            .map_err(|e| notify::Error::generic(&e.to_string()))?;

        if let Some(watcher) = &mut self.watcher {
            watcher.watch(&validated_path, RecursiveMode::Recursive)?;
            self.watched_paths.write().await.push(validated_path);
        }

        Ok(())
    }

    /// Remove a path from watching
    pub async fn remove_path<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        // Canonicalize the path to match the stored format
        let canonical_path = self.path_validator.safe_canonicalize(path);

        if let Some(watcher) = &mut self.watcher {
            // Use the canonicalized path for unwatching
            if let Err(e) = watcher.unwatch(&canonical_path) {
                // Log the error but don't fail the entire operation,
                // as the path might have been already removed or invalid.
                tracing::warn!("Failed to unwatch path: {:?}. Error: {}", canonical_path, e);
            }

            // Also use the canonicalized path for removing from the list
            self.watched_paths
                .write()
                .await
                .retain(|p| p != &canonical_path);
        }

        Ok(())
    }

    /// Pause file watching
    pub async fn pause(&self) {
        *self.is_paused.write().await = true;
    }

    /// Resume file watching
    pub async fn resume(&self) {
        *self.is_paused.write().await = false;
    }

    /// Get list of currently watched paths
    pub async fn watched_paths(&self) -> Vec<PathBuf> {
        self.watched_paths.read().await.clone()
    }
}

/// Handles event debouncing to prevent rapid file change spam
struct EventDebouncer {
    pending_events: HashMap<PathBuf, (FileEvent, tokio::time::Instant)>,
    debounce_duration: Duration,
}

impl EventDebouncer {
    fn new(debounce_duration: Duration) -> Self {
        Self {
            pending_events: HashMap::new(),
            debounce_duration,
        }
    }

    async fn process_event(
        &mut self,
        event: Event,
        path_validator: &PathValidator,
    ) -> Option<Vec<FileEvent>> {
        let mut result_events = Vec::new();
        let now = tokio::time::Instant::now();

        // Process the incoming event
        for path in event.paths {
            // Skip ignored files
            if Self::should_ignore_path(&path) {
                continue;
            }

            // Validate path security - use directory validation for watching operations
            if let Err(e) = path_validator.validate_directory_path(&path) {
                tracing::warn!(
                    "Security violation in file watcher: {} - {}",
                    path.display(),
                    e
                );
                continue;
            }

            let file_event = match &event.kind {
                EventKind::Create(_) => FileEvent::Created(path.clone()),
                EventKind::Modify(modify_kind) => {
                    match modify_kind {
                        ModifyKind::Name(RenameMode::From) => {
                            // This is a "from" part of a rename/move event
                            // Store it temporarily to wait for the "to" part
                            FileEvent::Renamed {
                                from: path.clone(),
                                to: PathBuf::new(), // Will be filled when we get the "to" part
                            }
                        }
                        ModifyKind::Name(RenameMode::To) => {
                            // This is a "to" part of a rename/move event
                            // Look for matching "from" event in pending events
                            self.handle_rename_or_move_event(path.clone())
                        }
                        ModifyKind::Name(RenameMode::Both) => {
                            // Some platforms send both from and to in one event
                            // For now, treat as a rename event
                            FileEvent::Renamed {
                                from: path.clone(),
                                to: path.clone(), // This will be handled by the frontend conversion
                            }
                        }
                        ModifyKind::Name(RenameMode::Any) => {
                            // Generic rename event
                            FileEvent::Renamed {
                                from: path.clone(),
                                to: path.clone(),
                            }
                        }
                        _ => {
                            // Regular modification (data, metadata, etc.)
                            FileEvent::Modified(path.clone())
                        }
                    }
                }
                EventKind::Remove(_) => FileEvent::Deleted(path.clone()),
                EventKind::Any => continue,       // Ignore generic events
                EventKind::Other => continue,     // Ignore other events
                EventKind::Access(_) => continue, // Ignore access events
            };

            // For rename/move events that are waiting for completion, store them temporarily
            // For other events, process them immediately
            if matches!(
                &file_event,
                FileEvent::Renamed { to, .. } if to.as_os_str().is_empty()
            ) {
                // This is an incomplete rename event waiting for the "to" part
                self.pending_events.insert(path, (file_event, now));
            } else {
                // This is a complete event, add it to results
                result_events.push(file_event);
            }
        }

        // Check for events that have passed the debounce period
        let expired_paths: Vec<PathBuf> = self
            .pending_events
            .iter()
            .filter(|(_, (_, timestamp))| now.duration_since(*timestamp) >= self.debounce_duration)
            .map(|(path, _)| path.clone())
            .collect();

        for path in expired_paths {
            if let Some((event, _)) = self.pending_events.remove(&path) {
                result_events.push(event);
            }
        }

        if result_events.is_empty() {
            None
        } else {
            Some(result_events)
        }
    }

    /// Handle rename or move events by finding matching "from" events
    fn handle_rename_or_move_event(&mut self, to_path: PathBuf) -> FileEvent {
        // Look for matching "from" event in pending events
        let mut matching_from_path = None;

        for (path, (event, _)) in &self.pending_events {
            if let FileEvent::Renamed { from, to } = event {
                if to.as_os_str().is_empty() {
                    // This is an incomplete rename event waiting for completion
                    matching_from_path = Some((path.clone(), from.clone()));
                    break;
                }
            }
        }

        if let Some((from_path, from)) = matching_from_path {
            // Remove the pending event
            self.pending_events.remove(&from_path);

            // Determine if this is a rename (same directory) or move (different directory)
            let from_dir = from.parent().unwrap_or_else(|| Path::new(""));
            let to_dir = to_path.parent().unwrap_or_else(|| Path::new(""));

            if from_dir == to_dir {
                // Same directory - this is a rename
                FileEvent::Renamed { from, to: to_path }
            } else {
                // Different directory - this is a move
                FileEvent::Moved { from, to: to_path }
            }
        } else {
            // No matching "from" event found, treat as regular modify
            FileEvent::Modified(to_path)
        }
    }

    /// Check if a path should be ignored (common system files)
    fn should_ignore_path(path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();

        path_str.ends_with(".tmp")
            || path_str.ends_with(".swp")
            || path_str.ends_with(".lock")
            || path_str.ends_with("~")
            || path_str.contains("/.git/")
            || path_str.contains("/node_modules/")
            || path_str.ends_with(".ds_store")
            || path_str.ends_with("thumbs.db")
            || path_str.ends_with("desktop.ini")
    }
}
