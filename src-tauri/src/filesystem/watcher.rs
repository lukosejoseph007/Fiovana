// src-tauri/src/filesystem/watcher.rs
// File watcher system with debouncing, filtering, and security integration

use crate::filesystem::event_persistence::{
    EventPersistence, OfflineReconciliation, PersistenceConfig,
};
use crate::filesystem::event_processor::{
    EventProcessorConfig, OptimizedEventProcessor, PrioritizedEvent,
};
use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use crate::resource_monitor::{ResourceMonitor, ResourceMonitorConfig, ResourceSnapshot};
// NotificationEmitter will be created in the event processing task
use chrono::{DateTime, Utc};
use notify::event::{ModifyKind, RenameMode};
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};
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
    pub enable_persistence: bool,
    pub persistence_config: PersistenceConfig,
    pub workspace_id: Option<String>,
    pub enable_resource_monitoring: bool,
    pub resource_monitor_config: ResourceMonitorConfig,
    pub enable_optimized_processing: bool,
    pub event_processor_config: EventProcessorConfig,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        Self {
            debounce_duration: Duration::from_millis(500),
            security_config: SecurityConfig::default(),
            enable_persistence: false,
            persistence_config: PersistenceConfig::default(),
            workspace_id: None,
            enable_resource_monitoring: true,
            resource_monitor_config: ResourceMonitorConfig::default(),
            enable_optimized_processing: true,
            event_processor_config: EventProcessorConfig::default(),
        }
    }
}

/// Main file watcher that manages watching directories with security integration
pub struct DocumentWatcher<R: tauri::Runtime> {
    watcher: Option<RecommendedWatcher>,
    event_sender: mpsc::UnboundedSender<FileEvent>,
    watched_paths: Arc<RwLock<Vec<PathBuf>>>,
    config: WatcherConfig,
    is_paused: Arc<RwLock<bool>>,
    path_validator: PathValidator,
    #[allow(dead_code)] // Used for app-specific operations and notifications
    app_handle: AppHandle<R>,
    persistence: Option<Arc<EventPersistence>>,
    #[allow(dead_code)] // Used for offline reconciliation when persistence is enabled
    reconciliation: Option<Arc<OfflineReconciliation>>,
    resource_monitor: Option<Arc<ResourceMonitor>>,
    event_processor: Option<Arc<RwLock<OptimizedEventProcessor>>>,
}

impl<R: tauri::Runtime> DocumentWatcher<R> {
    /// Create a new DocumentWatcher with the given configuration
    pub fn new(
        config: WatcherConfig,
        app_handle: AppHandle<R>,
    ) -> (Self, mpsc::UnboundedReceiver<FileEvent>) {
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

        // Initialize persistence if enabled
        let (persistence, reconciliation) = if config.enable_persistence {
            let app_data_dir = app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("proxemic"));

            // Ensure the directory exists
            if let Err(e) = std::fs::create_dir_all(&app_data_dir) {
                tracing::error!("Failed to create app data directory: {}", e);
                (None, None)
            } else {
                let events_file = app_data_dir.join("file_events.jsonl");
                match EventPersistence::new(events_file, config.persistence_config.clone()) {
                    Ok(persistence) => {
                        let persistence_arc = Arc::new(persistence);
                        let reconciliation_arc =
                            Arc::new(OfflineReconciliation::new((*persistence_arc).clone()));
                        (Some(persistence_arc), Some(reconciliation_arc))
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize event persistence: {}", e);
                        (None, None)
                    }
                }
            }
        } else {
            (None, None)
        };

        // Initialize resource monitoring if enabled
        let resource_monitor = if config.enable_resource_monitoring {
            Some(Arc::new(ResourceMonitor::with_config(
                config.resource_monitor_config.clone(),
            )))
        } else {
            None
        };

        // Initialize optimized event processor if enabled
        let event_processor = if config.enable_optimized_processing {
            Some(Arc::new(RwLock::new(OptimizedEventProcessor::new(
                config.event_processor_config.clone(),
            ))))
        } else {
            None
        };

        (
            Self {
                watcher: None,
                event_sender,
                watched_paths: Arc::new(RwLock::new(Vec::new())),
                config,
                is_paused: Arc::new(RwLock::new(false)),
                path_validator,
                app_handle,
                persistence,
                reconciliation,
                resource_monitor,
                event_processor,
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

        // Clone necessary data before starting processing
        let persistence_clone = self.persistence.clone();
        let resource_monitor_clone = self.resource_monitor.clone();
        let watched_paths_clone = self.watched_paths.clone();
        let workspace_id = self.config.workspace_id.clone();

        // Check if optimized processing is enabled
        if let Some(ref event_processor) = self.event_processor {
            self.start_optimized_processing(
                tx.clone(),
                rx,
                event_processor.clone(),
                path_validator,
                event_sender,
                persistence_clone,
                resource_monitor_clone,
                watched_paths_clone,
                workspace_id,
                is_paused,
            )
            .await?;
        } else {
            // Fallback to legacy event processing
            self.start_legacy_processing(
                rx,
                event_sender,
                config,
                path_validator,
                persistence_clone,
                resource_monitor_clone,
                watched_paths_clone,
                workspace_id,
                is_paused,
            )
            .await;
        }

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

    /// Flush any pending batched events (useful for cleanup or shutdown)
    #[allow(dead_code)]
    pub async fn flush_events(&self) {
        // Flush any pending events in persistence
        if let Some(ref persistence) = self.persistence {
            if let Err(e) = persistence.flush_pending_events().await {
                tracing::error!("Failed to flush pending events to persistence: {}", e);
            }
        }
        tracing::debug!("Event flushing completed");
    }

    /// Start persistence maintenance tasks
    #[allow(dead_code)] // Part of the persistence API for maintenance
    pub async fn start_persistence_maintenance(&self) -> Option<mpsc::UnboundedReceiver<String>> {
        if let Some(ref persistence) = self.persistence {
            Some(persistence.start_maintenance_tasks().await)
        } else {
            None
        }
    }

    /// Reconcile offline events
    #[allow(dead_code)] // Part of the persistence API for offline reconciliation
    pub async fn reconcile_offline_events(
        &self,
    ) -> std::result::Result<
        Vec<crate::filesystem::event_persistence::PersistedFileEvent>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        if let Some(ref reconciliation) = self.reconciliation {
            reconciliation
                .reconcile_offline_events(self.config.workspace_id.as_deref())
                .await
        } else {
            Ok(Vec::new())
        }
    }

    /// Get persistence statistics
    #[allow(dead_code)] // Part of the persistence API for monitoring
    pub async fn get_persistence_statistics(
        &self,
    ) -> Option<crate::filesystem::event_persistence::PersistenceStatistics> {
        if let Some(ref persistence) = self.persistence {
            persistence.get_statistics().await.ok()
        } else {
            None
        }
    }

    /// Get unprocessed events for manual reconciliation
    #[allow(dead_code)] // Part of the persistence API for manual reconciliation
    pub async fn get_unprocessed_events(
        &self,
        limit: Option<usize>,
    ) -> std::result::Result<
        Vec<crate::filesystem::event_persistence::PersistedFileEvent>,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        if let Some(ref persistence) = self.persistence {
            persistence
                .get_unprocessed_events(self.config.workspace_id.as_deref(), limit)
                .await
        } else {
            Ok(Vec::new())
        }
    }

    /// Mark events as processed
    #[allow(dead_code)] // Part of the persistence API for marking events as processed
    pub async fn mark_events_processed(
        &self,
        event_ids: &[i64],
    ) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(ref persistence) = self.persistence {
            persistence.mark_events_processed(event_ids).await
        } else {
            Ok(())
        }
    }

    /// Get current resource usage statistics
    #[allow(dead_code)]
    pub async fn get_resource_usage(&self) -> Option<ResourceSnapshot> {
        if let Some(ref monitor) = self.resource_monitor {
            monitor.get_latest_snapshot().await
        } else {
            None
        }
    }

    /// Get resource usage history
    #[allow(dead_code)]
    pub async fn get_resource_history(&self) -> Vec<ResourceSnapshot> {
        if let Some(ref monitor) = self.resource_monitor {
            monitor.get_resource_history().await
        } else {
            Vec::new()
        }
    }

    /// Check if resource usage is healthy
    #[allow(dead_code)]
    pub async fn is_resource_usage_healthy(&self) -> bool {
        if let Some(ref monitor) = self.resource_monitor {
            monitor.is_resource_usage_healthy().await
        } else {
            true // No monitoring, assume healthy
        }
    }

    /// Force a resource monitoring sample
    #[allow(dead_code)]
    pub async fn sample_resource_usage_now(&self) -> std::result::Result<(), String> {
        if let Some(ref monitor) = self.resource_monitor {
            let watched_count = self.watched_paths.read().await.len();
            monitor.sample_resources(watched_count).await.map(|_| ())
        } else {
            Err("Resource monitoring not enabled".to_string())
        }
    }

    /// Start optimized event processing using the high-performance processor
    async fn start_optimized_processing(
        &self,
        _tx: crossbeam_channel::Sender<Event>,
        rx: crossbeam_channel::Receiver<Event>,
        event_processor: Arc<RwLock<OptimizedEventProcessor>>,
        path_validator: PathValidator,
        event_sender: mpsc::UnboundedSender<FileEvent>,
        persistence: Option<Arc<EventPersistence>>,
        resource_monitor: Option<Arc<ResourceMonitor>>,
        watched_paths: Arc<RwLock<Vec<PathBuf>>>,
        workspace_id: Option<String>,
        is_paused: Arc<RwLock<bool>>,
    ) -> Result<()> {
        // Start the optimized event processor
        let mut processor = event_processor.write().await;

        // Create the event handler closure
        let event_handler = {
            let event_sender = event_sender.clone();
            let persistence = persistence.clone();
            let resource_monitor = resource_monitor.clone();
            let watched_paths = watched_paths.clone();
            let workspace_id = workspace_id.clone();

            move |prioritized_events: Vec<PrioritizedEvent>| {
                let event_sender = event_sender.clone();
                let persistence = persistence.clone();
                let resource_monitor = resource_monitor.clone();
                let watched_paths = watched_paths.clone();
                let workspace_id = workspace_id.clone();

                async move {
                    for prioritized_event in prioritized_events {
                        let file_event = prioritized_event.event;

                        // Log security audit event
                        SecurityAuditor::log_file_access_attempt(
                            file_event.path(),
                            "file_watch_optimized",
                            &Ok(file_event.path().to_path_buf()),
                            "development",
                            None::<uuid::Uuid>,
                        );

                        // Store event in persistence if enabled
                        if let Some(ref persistence) = persistence {
                            if let Err(e) = persistence
                                .store_event(file_event.clone(), workspace_id.clone())
                                .await
                            {
                                tracing::warn!("Failed to persist file event: {}", e);
                            }
                        }

                        // Send file event to channel for UI updates
                        if event_sender.send(file_event).is_err() {
                            tracing::warn!("Event receiver dropped, stopping event processing");
                            break;
                        }
                    }

                    // Sample resource usage after processing batch
                    if let Some(ref monitor) = resource_monitor {
                        let watched_count = watched_paths.read().await.len();
                        if let Err(e) = monitor.sample_resources(watched_count).await {
                            if !e.contains("Sampling interval not reached") {
                                tracing::debug!("Resource monitoring sample failed: {}", e);
                            }
                        }
                    }

                    Ok(())
                }
            }
        };

        if let Err(e) = processor.start_processing(event_handler).await {
            return Err(notify::Error::generic(&format!(
                "Failed to start event processor: {}",
                e
            )));
        }
        drop(processor); // Release the write lock

        // Spawn raw event forwarder
        let event_processor_clone = event_processor.clone();
        let path_validator_clone = path_validator.clone();
        tokio::spawn(async move {
            while let Ok(Ok(event)) = tokio::task::spawn_blocking({
                let rx = rx.clone();
                move || rx.recv()
            })
            .await
            {
                if *is_paused.read().await {
                    continue;
                }

                // Submit raw event to optimized processor
                let processor = event_processor_clone.read().await;
                if let Err(e) = processor
                    .submit_raw_event(event, &path_validator_clone)
                    .await
                {
                    tracing::warn!("Failed to submit event to optimized processor: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Start legacy event processing for backward compatibility
    async fn start_legacy_processing(
        &self,
        rx: crossbeam_channel::Receiver<Event>,
        event_sender: mpsc::UnboundedSender<FileEvent>,
        config: WatcherConfig,
        path_validator: PathValidator,
        persistence: Option<Arc<EventPersistence>>,
        resource_monitor: Option<Arc<ResourceMonitor>>,
        watched_paths: Arc<RwLock<Vec<PathBuf>>>,
        workspace_id: Option<String>,
        is_paused: Arc<RwLock<bool>>,
    ) {
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
                            "file_watch_legacy",
                            &Ok(file_event.path().to_path_buf()),
                            "development",
                            None::<uuid::Uuid>,
                        );

                        // Store event in persistence if enabled
                        if let Some(ref persistence) = persistence {
                            if let Err(e) = persistence
                                .store_event(file_event.clone(), workspace_id.clone())
                                .await
                            {
                                tracing::warn!("Failed to persist file event: {}", e);
                            }
                        }

                        // Send file event to channel for UI updates
                        if event_sender.send(file_event).is_err() {
                            break;
                        }
                    }

                    // Sample resource usage after processing events
                    if let Some(ref monitor) = resource_monitor {
                        let watched_count = watched_paths.read().await.len();
                        if let Err(e) = monitor.sample_resources(watched_count).await {
                            if !e.contains("Sampling interval not reached") {
                                tracing::debug!("Resource monitoring sample failed: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }

    /// Get event processing performance metrics
    #[allow(dead_code)]
    pub async fn get_event_processing_metrics(&self) -> Option<(u64, u64, u64, u64)> {
        if let Some(ref processor) = self.event_processor {
            let processor = processor.read().await;
            let metrics = processor.get_metrics();
            Some((
                metrics.events_processed.load(Ordering::Relaxed),
                metrics.events_dropped.load(Ordering::Relaxed),
                metrics.backpressure_events.load(Ordering::Relaxed),
                metrics.average_processing_time_us.load(Ordering::Relaxed),
            ))
        } else {
            None
        }
    }
}

/// Handles event debouncing and batching to prevent rapid file change spam
pub struct EventDebouncer {
    pending_events: HashMap<PathBuf, (FileEvent, tokio::time::Instant)>,
    batched_events: Vec<FileEvent>,
    debounce_duration: Duration,
    batch_duration: Duration,
    max_batch_size: usize,
    last_batch_time: tokio::time::Instant,
}

impl EventDebouncer {
    pub fn new(debounce_duration: Duration) -> Self {
        Self {
            pending_events: HashMap::new(),
            batched_events: Vec::new(),
            debounce_duration,
            batch_duration: Duration::from_millis(100), // 100ms batch window
            max_batch_size: 50,                         // Max 50 events per batch
            last_batch_time: tokio::time::Instant::now(),
        }
    }

    pub async fn process_event(
        &mut self,
        event: Event,
        path_validator: &PathValidator,
    ) -> Option<Vec<FileEvent>> {
        let now = tokio::time::Instant::now();
        let _immediate_events: Vec<FileEvent> = Vec::new();

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
            // For other events, add to batch
            if matches!(
                &file_event,
                FileEvent::Renamed { to, .. } if to.as_os_str().is_empty()
            ) {
                // This is an incomplete rename event waiting for the "to" part
                self.pending_events.insert(path, (file_event, now));
            } else {
                // This is a complete event, add to batch
                self.batched_events.push(file_event);
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
                self.batched_events.push(event);
            }
        }

        // Check if we should flush the batch
        let should_flush = self.batched_events.len() >= self.max_batch_size
            || now.duration_since(self.last_batch_time) >= self.batch_duration;

        if should_flush && !self.batched_events.is_empty() {
            let batch = std::mem::take(&mut self.batched_events);
            self.last_batch_time = now;
            Some(batch)
        } else {
            None
        }
    }

    /// Flush any remaining batched events (useful for cleanup or shutdown)
    #[allow(dead_code)]
    pub fn flush(&mut self) -> Vec<FileEvent> {
        std::mem::take(&mut self.batched_events)
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

/// Conflict detection types for simultaneous external changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// File modified externally while application was running
    ExternalModification,
    /// File deleted externally while application was running
    ExternalDeletion,
    /// File created externally with same name as modified file
    ExternalCreationConflict,
    /// File content conflict between application and external changes
    ContentConflict,
    /// Timestamp conflict (external modification has newer timestamp)
    TimestampConflict,
}

/// Conflict detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictResult {
    pub conflict_type: ConflictType,
    pub file_path: PathBuf,
    pub external_timestamp: Option<DateTime<Utc>>,
    pub application_timestamp: Option<DateTime<Utc>>,
    pub external_hash: Option<String>,
    pub application_hash: Option<String>,
    pub severity: String, // "low", "medium", "high", "critical"
    pub resolution_required: bool,
}

/// File metadata snapshot for conflict detection
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FileSnapshot {
    pub path: PathBuf,
    pub size: u64,
    pub modified_time: DateTime<Utc>,
    pub hash: String,
    pub created_time: DateTime<Utc>,
}

impl FileSnapshot {
    /// Create a snapshot of a file's current state
    #[allow(dead_code)]
    pub fn create(path: &Path) -> io::Result<Self> {
        let metadata = std::fs::metadata(path)?;

        let modified_time: DateTime<Utc> = metadata.modified()?.into();
        let created_time: DateTime<Utc> = metadata.created()?.into();

        let hash = Self::calculate_file_hash(path)?;

        Ok(Self {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified_time,
            hash,
            created_time,
        })
    }

    /// Calculate SHA-256 hash of file content
    #[allow(dead_code)]
    fn calculate_file_hash(path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Compare two snapshots to detect conflicts
    #[allow(dead_code)]
    pub fn compare(&self, other: &Self) -> Option<ConflictResult> {
        if self.path != other.path {
            return None;
        }

        // Check for content conflicts (hash mismatch) regardless of timestamp
        if self.hash != other.hash {
            return Some(ConflictResult {
                conflict_type: ConflictType::ContentConflict,
                file_path: self.path.clone(),
                external_timestamp: Some(other.modified_time),
                application_timestamp: Some(self.modified_time),
                external_hash: Some(other.hash.clone()),
                application_hash: Some(self.hash.clone()),
                severity: "high".to_string(),
                resolution_required: true,
            });
        }

        // Check for timestamp conflicts (timestamp mismatch but same content)
        if self.modified_time != other.modified_time {
            return Some(ConflictResult {
                conflict_type: ConflictType::TimestampConflict,
                file_path: self.path.clone(),
                external_timestamp: Some(other.modified_time),
                application_timestamp: Some(self.modified_time),
                external_hash: Some(other.hash.clone()),
                application_hash: Some(self.hash.clone()),
                severity: "medium".to_string(),
                resolution_required: true,
            });
        }

        None
    }
}

/// Conflict detector for simultaneous external changes
#[derive(Clone)]
#[allow(dead_code)]
pub struct ConflictDetector {
    pub file_snapshots: HashMap<PathBuf, FileSnapshot>,
    last_check_time: DateTime<Utc>,
    check_interval: Duration,
}

impl ConflictDetector {
    #[allow(dead_code)]
    pub fn new(check_interval: Duration) -> Self {
        Self {
            file_snapshots: HashMap::new(),
            last_check_time: Utc::now(),
            check_interval,
        }
    }

    /// Take a snapshot of watched files for conflict detection
    #[allow(dead_code)]
    pub async fn take_snapshot(
        &mut self,
        watched_paths: &[PathBuf],
    ) -> Vec<io::Result<FileSnapshot>> {
        let mut results = Vec::new();

        for path in watched_paths {
            // Only snapshot files, not directories
            if path.is_file() {
                match FileSnapshot::create(path) {
                    Ok(snapshot) => {
                        self.file_snapshots.insert(path.clone(), snapshot.clone());
                        results.push(Ok(snapshot));
                    }
                    Err(e) => {
                        if e.kind() == io::ErrorKind::NotFound {
                            // File doesn't exist (may have been deleted)
                            results.push(Err(e));
                        } else {
                            // Other error
                            results.push(Err(e));
                        }
                    }
                }
            }
        }

        self.last_check_time = Utc::now();
        results
    }

    /// Check for conflicts between current state and stored snapshots
    #[allow(dead_code)]
    pub async fn check_conflicts(&self, watched_paths: &[PathBuf]) -> Vec<ConflictResult> {
        let mut conflicts = Vec::new();

        for path in watched_paths {
            // Check if we have a snapshot for this path (indicating it was previously a file)
            if let Some(stored_snapshot) = self.file_snapshots.get(path) {
                // Try to create a current snapshot - this will fail if the file was deleted
                match FileSnapshot::create(path) {
                    Ok(current_snapshot) => {
                        // File still exists, check for content/timestamp conflicts
                        if let Some(conflict) = stored_snapshot.compare(&current_snapshot) {
                            conflicts.push(conflict);
                        }
                    }
                    Err(e) => {
                        if e.kind() == io::ErrorKind::NotFound {
                            // File was deleted externally
                            conflicts.push(ConflictResult {
                                conflict_type: ConflictType::ExternalDeletion,
                                file_path: path.clone(),
                                external_timestamp: Some(Utc::now()),
                                application_timestamp: Some(stored_snapshot.modified_time),
                                external_hash: None,
                                application_hash: Some(stored_snapshot.hash.clone()),
                                severity: "high".to_string(),
                                resolution_required: true,
                            });
                        }
                    }
                }
            }
        }

        conflicts
    }

    /// Check if it's time for the next conflict detection scan
    #[allow(dead_code)]
    pub fn should_check(&self) -> bool {
        Utc::now().signed_duration_since(self.last_check_time)
            >= chrono::Duration::from_std(self.check_interval).unwrap()
    }

    /// Clear snapshots for a specific path
    #[allow(dead_code)]
    pub fn clear_snapshot(&mut self, path: &Path) {
        self.file_snapshots.remove(path);
    }

    /// Clear all snapshots
    #[allow(dead_code)]
    pub fn clear_all_snapshots(&mut self) {
        self.file_snapshots.clear();
    }
}

/// Integrate conflict detection with the file watcher
#[allow(dead_code)]
pub struct WatcherWithConflictDetection<R: tauri::Runtime> {
    watcher: Arc<RwLock<DocumentWatcher<R>>>,
    conflict_detector: ConflictDetector,
    pub conflict_sender: Option<mpsc::UnboundedSender<Vec<ConflictResult>>>,
}

impl<R: tauri::Runtime> WatcherWithConflictDetection<R> {
    /// Create a new watcher with conflict detection
    #[allow(dead_code)]
    pub fn new(
        config: WatcherConfig,
        app_handle: AppHandle<R>,
        check_interval: Duration,
    ) -> (Self, mpsc::UnboundedReceiver<FileEvent>) {
        let (watcher, event_receiver) = DocumentWatcher::new(config, app_handle);
        let conflict_detector = ConflictDetector::new(check_interval);

        (
            Self {
                watcher: Arc::new(RwLock::new(watcher)),
                conflict_detector,
                conflict_sender: None,
            },
            event_receiver,
        )
    }

    /// Set up conflict notification channel
    #[allow(dead_code)]
    pub fn set_conflict_channel(&mut self, sender: mpsc::UnboundedSender<Vec<ConflictResult>>) {
        self.conflict_sender = Some(sender);
    }

    /// Start the watcher with periodic conflict detection
    #[allow(dead_code)]
    pub async fn start_with_conflict_detection(&mut self) -> Result<()> {
        self.watcher.write().await.start().await?;

        // Start conflict detection background task
        let conflict_detector = self.conflict_detector.clone();
        let watcher = self.watcher.clone();
        let conflict_sender = self.conflict_sender.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await; // Check every 30 seconds

                if conflict_detector.should_check() {
                    let watched_paths = watcher.read().await.watched_paths().await;
                    let conflicts = conflict_detector.check_conflicts(&watched_paths).await;

                    if !conflicts.is_empty() {
                        if let Some(sender) = &conflict_sender {
                            let _ = sender.send(conflicts);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Take a manual snapshot of watched files
    #[allow(dead_code)]
    pub async fn take_snapshot(&mut self) -> Vec<io::Result<FileSnapshot>> {
        let watched_paths = self.watcher.read().await.watched_paths().await;
        self.conflict_detector.take_snapshot(&watched_paths).await
    }

    /// Check for conflicts manually
    #[allow(dead_code)]
    pub async fn check_conflicts_manual(&self) -> Vec<ConflictResult> {
        let watched_paths = self.watcher.read().await.watched_paths().await;
        self.conflict_detector.check_conflicts(&watched_paths).await
    }
}
