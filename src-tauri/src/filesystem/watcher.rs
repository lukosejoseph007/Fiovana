// src-tauri/src/filesystem/watcher.rs
// File watcher system with debouncing, filtering, and security integration

use crate::filesystem::security::audit_logger::SecurityAuditor;
use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::security::security_config::SecurityConfig;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Result, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock};

/// File event types that can be emitted by the watcher
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
    Renamed { from: PathBuf, to: PathBuf },
}

impl FileEvent {
    /// Get the path associated with this event
    pub fn path(&self) -> &Path {
        match self {
            FileEvent::Created(path) => path,
            FileEvent::Modified(path) => path,
            FileEvent::Deleted(path) => path,
            FileEvent::Renamed { to, .. } => to,
        }
    }
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

            while let Ok(event) = rx.recv() {
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
                            None, // correlation_id should be Option<Uuid>, not String
                        );

                        let _ = event_sender.send(file_event);
                    }
                }
            }
        });

        let watcher = RecommendedWatcher::new(
            move |res: Result<Event>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
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

        // Validate path using existing security system
        let validated_path = self
            .path_validator
            .validate_import_path(path)
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

        if let Some(watcher) = &mut self.watcher {
            watcher.unwatch(path)?;
            self.watched_paths.write().await.retain(|p| p != path);
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

            // Validate path security
            if let Err(e) = path_validator.validate_import_path(&path) {
                tracing::warn!(
                    "Security violation in file watcher: {} - {}",
                    path.display(),
                    e
                );
                continue;
            }

            let file_event = match event.kind {
                EventKind::Create(_) => FileEvent::Created(path.clone()),
                EventKind::Modify(_) => FileEvent::Modified(path.clone()),
                EventKind::Remove(_) => FileEvent::Deleted(path.clone()),
                // Rename events in notify v6 are handled differently - they come as Modify events
                // with specific ModifyKind variants, or we need to handle them at a different level
                _ => continue,
            };

            self.pending_events.insert(path, (file_event, now));
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
