// src-tauri/src/filesystem/event_persistence.rs
// Safe and simple event persistence for offline reconciliation using file-based storage

use crate::filesystem::watcher::FileEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, info, warn};

/// Persisted file event with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedFileEvent {
    pub id: i64,
    pub event: FileEvent,
    pub timestamp: DateTime<Utc>,
    pub workspace_id: Option<String>,
    pub processed: bool,
    pub retry_count: u32,
}

/// Configuration for event persistence
#[derive(Debug, Clone)]
#[allow(dead_code)] // These fields are used for configuration but not directly accessed
pub struct PersistenceConfig {
    /// Maximum number of events to keep in file storage
    pub max_events: usize,
    /// How long to keep processed events (for reconciliation)
    pub retention_hours: u32,
    /// Maximum batch size for file operations
    pub batch_size: usize,
    /// Cleanup interval
    pub cleanup_interval: Duration,
    /// Maximum retry attempts for failed events
    pub max_retries: u32,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            max_events: 10_000,           // Keep at most 10k events
            retention_hours: 72,          // Keep events for 3 days
            batch_size: 100,              // Process 100 events at a time
            cleanup_interval: Duration::from_secs(3600), // Cleanup every hour
            max_retries: 3,               // Retry failed events 3 times
        }
    }
}

/// Safe event persistence manager with resource limits using file-based storage
pub struct EventPersistence {
    events_file: PathBuf,
    config: PersistenceConfig,
    pending_events: Arc<Mutex<Vec<PersistedFileEvent>>>,
    next_id: Arc<Mutex<i64>>,
    file_lock: Arc<Mutex<()>>, // Mutex to synchronize file access
}

#[allow(dead_code)] // These methods are part of the public API for future use
impl EventPersistence {
    /// Create a new event persistence manager using file-based storage
    pub fn new(events_file: PathBuf, config: PersistenceConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure the parent directory exists
        if let Some(parent) = events_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Read existing events to get next ID
        let next_id = if events_file.exists() {
            match Self::read_events_from_file(&events_file) {
                Ok(events) => events.iter().map(|e| e.id).max().unwrap_or(0) + 1,
                Err(_) => 1, // Start from 1 if file is corrupted
            }
        } else {
            1
        };

        let persistence = Self {
            events_file,
            config,
            pending_events: Arc::new(Mutex::new(Vec::new())),
            next_id: Arc::new(Mutex::new(next_id)),
            file_lock: Arc::new(Mutex::new(())),
        };

        info!("Event persistence initialized with file: {:?}", persistence.events_file);
        Ok(persistence)
    }

    /// Read events from file
    fn read_events_from_file(file_path: &PathBuf) -> Result<Vec<PersistedFileEvent>, String> {
        let file = File::open(file_path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);
        let mut events = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| e.to_string())?;
            if !line.trim().is_empty() {
                match serde_json::from_str::<PersistedFileEvent>(&line) {
                    Ok(event) => events.push(event),
                    Err(e) => {
                        warn!("Failed to parse event line: {} - {}", line, e);
                        // Continue processing other events even if one is corrupted
                    }
                }
            }
        }

        Ok(events)
    }

    /// Append events to file
    fn append_events_to_file(file_path: &PathBuf, events: &[PersistedFileEvent]) -> Result<(), String> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path).map_err(|e| e.to_string())?;

        let mut writer = BufWriter::new(file);

        for event in events {
            let json_line = serde_json::to_string(event).map_err(|e| e.to_string())?;
            writeln!(writer, "{}", json_line).map_err(|e| e.to_string())?;
        }

        writer.flush().map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Rewrite the entire file with filtered events
    fn rewrite_events_file(file_path: &PathBuf, events: &[PersistedFileEvent]) -> Result<(), String> {
        let temp_file = file_path.with_extension("tmp");

        {
            let file = File::create(&temp_file).map_err(|e| e.to_string())?;
            let mut writer = BufWriter::new(file);

            for event in events {
                let json_line = serde_json::to_string(event).map_err(|e| e.to_string())?;
                writeln!(writer, "{}", json_line).map_err(|e| e.to_string())?;
            }

            writer.flush().map_err(|e| e.to_string())?;
        }

        // Atomically replace the original file
        std::fs::rename(temp_file, file_path).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Store a file event asynchronously with resource limits
    pub async fn store_event(
        &self,
        event: FileEvent,
        workspace_id: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Check if we're hitting resource limits
        let pending_count = {
            let pending = self.pending_events.lock().await;
            pending.len()
        };

        if pending_count > self.config.batch_size * 2 {
            warn!("Too many pending events ({}), dropping oldest", pending_count);
            self.cleanup_pending_events().await;
        }

        let persisted_event = PersistedFileEvent {
            id: {
                let mut next_id = self.next_id.lock().await;
                let id = *next_id;
                *next_id += 1;
                id
            },
            event,
            timestamp: Utc::now(),
            workspace_id,
            processed: false,
            retry_count: 0,
        };

        // Add to pending events for batch processing
        let new_pending_count = {
            let mut pending = self.pending_events.lock().await;
            pending.push(persisted_event);
            pending.len()
        };

        // Trigger batch write if we have enough events
        if new_pending_count >= self.config.batch_size {
            self.flush_pending_events().await?;
        }

        Ok(())
    }

    /// Flush pending events to file in batches
    pub async fn flush_pending_events(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (events_to_write, events_count) = {
            let mut pending = self.pending_events.lock().await;
            if pending.is_empty() {
                return Ok(());
            }
            let events = std::mem::take(&mut *pending);
            let count = events.len();
            (events, count)
        };

        let events_file = self.events_file.clone();
        let file_lock = self.file_lock.clone();

        // Write events to file in background task
        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let _lock = file_lock.lock().await;
                if let Err(e) = Self::append_events_to_file(&events_file, &events_to_write) {
                    error!("Failed to write events to file: {}", e);
                }
            });
        }).await.map_err(|e| format!("File write task failed: {}", e))?;

        debug!("Flushed {} events to file", events_count);
        Ok(())
    }


    /// Get unprocessed events for offline reconciliation
    pub async fn get_unprocessed_events(
        &self,
        workspace_id: Option<&str>,
        limit: Option<usize>,
    ) -> Result<Vec<PersistedFileEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let events_file = self.events_file.clone();
        let file_lock = self.file_lock.clone();
        let workspace_filter = workspace_id.map(|s| s.to_string());
        let limit = limit.unwrap_or(self.config.batch_size);

        let events = tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let _lock = file_lock.lock().await;
                Self::read_events_from_file(&events_file)
            })
        }).await.map_err(|e| format!("File read task failed: {}", e))??;

        // Filter unprocessed events for the specified workspace
        let filtered_events: Vec<_> = events
            .into_iter()
            .filter(|event| {
                !event.processed
                    && workspace_filter
                        .as_ref()
                        .map_or(true, |ws| event.workspace_id.as_ref() == Some(ws))
            })
            .take(limit)
            .collect();

        Ok(filtered_events)
    }


    /// Mark events as processed
    pub async fn mark_events_processed(
        &self,
        event_ids: &[i64],
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if event_ids.is_empty() {
            return Ok(());
        }

        let events_file = self.events_file.clone();
        let file_lock = self.file_lock.clone();
        let event_ids_set: HashSet<i64> = event_ids.iter().copied().collect();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let _lock = file_lock.lock().await;

                // Read all events
                let mut events = Self::read_events_from_file(&events_file)
                    .unwrap_or_else(|_| Vec::new());

                // Mark specified events as processed
                for event in &mut events {
                    if event_ids_set.contains(&event.id) {
                        event.processed = true;
                    }
                }

                // Write back to file
                Self::rewrite_events_file(&events_file, &events)
            })
        }).await.map_err(|e| format!("File update task failed: {}", e))??;

        debug!("Marked {} events as processed", event_ids.len());
        Ok(())
    }

    /// Clean up old events and manage file size
    pub async fn cleanup_old_events(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let events_file = self.events_file.clone();
        let file_lock = self.file_lock.clone();
        let retention_hours = self.config.retention_hours;
        let max_events = self.config.max_events;

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let _lock = file_lock.lock().await;

                // Read all events
                let mut events = Self::read_events_from_file(&events_file)
                    .unwrap_or_else(|_| Vec::new());

                let initial_count = events.len();
                let cutoff_time = Utc::now() - chrono::Duration::hours(retention_hours as i64);

                // Remove old processed events
                events.retain(|event| {
                    !(event.processed && event.timestamp < cutoff_time)
                });

                let after_retention_cleanup = events.len();

                // If we still have too many events, keep only the most recent ones
                if events.len() > max_events {
                    events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Sort by newest first
                    events.truncate(max_events);
                }

                let final_count = events.len();
                let deleted_old = initial_count - after_retention_cleanup;
                let deleted_excess = after_retention_cleanup - final_count;

                // Write cleaned events back to file
                if initial_count != final_count {
                    if let Err(e) = Self::rewrite_events_file(&events_file, &events) {
                        error!("Failed to write cleaned events: {}", e);
                        return Err(e);
                    }
                }

                if deleted_old > 0 || deleted_excess > 0 {
                    info!("File cleanup: removed {} old events, {} excess events", deleted_old, deleted_excess);
                }

                Ok(())
            })
        }).await.map_err(|e| format!("File cleanup task failed: {}", e))??;

        Ok(())
    }

    /// Clean up pending events in memory to prevent resource exhaustion
    async fn cleanup_pending_events(&self) {
        let mut pending = self.pending_events.lock().await;
        let current_len = pending.len();
        if current_len > self.config.batch_size {
            // Keep only the most recent events
            let keep_count = self.config.batch_size / 2;
            let drain_count = current_len - keep_count;
            pending.drain(0..drain_count);
            warn!("Cleaned up pending events, kept {} most recent", keep_count);
        }
    }

    /// Start background tasks for maintenance
    pub async fn start_maintenance_tasks(&self) -> mpsc::UnboundedReceiver<String> {
        let (status_tx, status_rx) = mpsc::unbounded_channel();

        // Periodic cleanup task
        let cleanup_persistence = self.clone();
        let cleanup_status = status_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_persistence.config.cleanup_interval);
            loop {
                interval.tick().await;

                if let Err(e) = cleanup_persistence.cleanup_old_events().await {
                    error!("Maintenance cleanup failed: {}", e);
                    let _ = cleanup_status.send(format!("Cleanup failed: {}", e));
                } else {
                    debug!("Maintenance cleanup completed");
                    let _ = cleanup_status.send("Cleanup completed".to_string());
                }
            }
        });

        // Periodic flush task
        let flush_persistence = self.clone();
        let flush_status = status_tx.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30)); // Flush every 30 seconds
            loop {
                interval.tick().await;

                if let Err(e) = flush_persistence.flush_pending_events().await {
                    error!("Periodic flush failed: {}", e);
                    let _ = flush_status.send(format!("Flush failed: {}", e));
                }
            }
        });

        status_rx
    }

    /// Get file statistics
    pub async fn get_statistics(&self) -> Result<PersistenceStatistics, Box<dyn std::error::Error + Send + Sync>> {
        let events_file = self.events_file.clone();
        let file_lock = self.file_lock.clone();

        tokio::task::spawn_blocking(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                let _lock = file_lock.lock().await;

                let events = Self::read_events_from_file(&events_file)
                    .unwrap_or_else(|_| Vec::new());

                let total_events = events.len();
                let unprocessed_events = events.iter().filter(|e| !e.processed).count();
                let failed_events = events.iter().filter(|e| e.retry_count > 0).count();

                let file_size_mb = if events_file.exists() {
                    std::fs::metadata(&events_file)
                        .map(|m| m.len() as f64 / 1024.0 / 1024.0)
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                Ok(PersistenceStatistics {
                    total_events,
                    unprocessed_events,
                    failed_events,
                    file_size_mb: file_size_mb,
                })
            })
        }).await.map_err(|e| format!("File statistics task failed: {}", e))?
    }
}

impl Clone for EventPersistence {
    fn clone(&self) -> Self {
        Self {
            events_file: self.events_file.clone(),
            config: self.config.clone(),
            pending_events: self.pending_events.clone(),
            next_id: self.next_id.clone(),
            file_lock: self.file_lock.clone(),
        }
    }
}

/// Statistics about the persistence system
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PersistenceStatistics {
    pub total_events: usize,
    pub unprocessed_events: usize,
    pub failed_events: usize,
    pub file_size_mb: f64,
}

/// Offline reconciliation manager
#[allow(dead_code)] // This struct is part of the persistence API
pub struct OfflineReconciliation {
    persistence: EventPersistence,
}

#[allow(dead_code)] // These methods are part of the reconciliation API
impl OfflineReconciliation {
    /// Create a new offline reconciliation manager
    pub fn new(persistence: EventPersistence) -> Self {
        Self { persistence }
    }

    /// Reconcile events that occurred while offline
    pub async fn reconcile_offline_events(
        &self,
        workspace_id: Option<&str>,
    ) -> Result<Vec<PersistedFileEvent>, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting offline event reconciliation for workspace: {:?}", workspace_id);

        // Get all unprocessed events
        let unprocessed_events = self.persistence.get_unprocessed_events(workspace_id, None).await?;

        if unprocessed_events.is_empty() {
            debug!("No unprocessed events found for reconciliation");
            return Ok(Vec::new());
        }

        info!("Found {} unprocessed events for reconciliation", unprocessed_events.len());

        // Filter events that still need reconciliation (file exists, etc.)
        let mut reconciled_events = Vec::new();
        let mut processed_ids = Vec::new();

        for event in unprocessed_events {
            match self.validate_event(&event).await {
                Ok(true) => {
                    reconciled_events.push(event.clone());
                    processed_ids.push(event.id);
                }
                Ok(false) => {
                    // Event no longer valid (file deleted, etc.)
                    processed_ids.push(event.id);
                    debug!("Event {} no longer valid, marking as processed", event.id);
                }
                Err(e) => {
                    warn!("Failed to validate event {}: {}", event.id, e);
                    // Don't mark as processed, might retry later
                }
            }
        }

        // Mark validated events as processed
        if !processed_ids.is_empty() {
            self.persistence.mark_events_processed(&processed_ids).await?;
        }

        info!("Reconciled {} valid events from offline queue", reconciled_events.len());
        Ok(reconciled_events)
    }

    /// Validate if an event is still relevant
    async fn validate_event(&self, event: &PersistedFileEvent) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let path = event.event.path();

        // Check if the file/directory still exists for non-delete events
        match &event.event {
            FileEvent::Deleted(_) => Ok(true), // Delete events are always valid
            _ => {
                // For other events, check if the file exists
                Ok(tokio::fs::metadata(path).await.is_ok())
            }
        }
    }
}
