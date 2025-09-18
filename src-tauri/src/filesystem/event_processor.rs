// src-tauri/src/filesystem/event_processor.rs
// High-performance event processing system with prioritization and backpressure handling

use crate::filesystem::security::path_validator::PathValidator;
use crate::filesystem::watcher::FileEvent;
use notify::Event;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::timeout;
use tracing::{error, info, warn};

/// Event priority levels for processing optimization
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    /// Critical events that need immediate processing (security violations, errors)
    Critical = 0,
    /// High priority events (user-initiated actions, important file changes)
    High = 1,
    /// Normal priority events (regular file modifications)
    Normal = 2,
    /// Low priority events (temporary files, system-generated events)
    Low = 3,
    /// Background events (maintenance, cleanup)
    Background = 4,
}

impl EventPriority {
    /// Determine priority based on file path and event type
    pub fn from_path_and_event(path: &Path, event: &FileEvent) -> Self {
        let path_str = path.to_string_lossy().to_lowercase();

        // Critical priority for security-related paths
        if path_str.contains("security")
            || path_str.contains("auth")
            || path_str.contains("credential")
        {
            return EventPriority::Critical;
        }

        // Low priority for temporary and system files
        if path_str.ends_with(".tmp")
            || path_str.ends_with(".temp")
            || path_str.ends_with(".swp")
            || path_str.ends_with(".lock")
            || path_str.ends_with("~")
            || path_str.contains("/.git/")
            || path_str.contains("/node_modules/")
            || path_str.contains("/.cache/")
            || path_str.contains("/target/debug/")
            || path_str.contains("/target/release/")
        {
            return EventPriority::Low;
        }

        // High priority for configuration and source code files
        if path_str.ends_with(".json")
            || path_str.ends_with(".yaml")
            || path_str.ends_with(".yml")
            || path_str.ends_with(".toml")
            || path_str.ends_with(".rs")
            || path_str.ends_with(".ts")
            || path_str.ends_with(".js")
            || path_str.ends_with(".py")
        {
            return EventPriority::High;
        }

        // Higher priority for creation and deletion events
        match event {
            FileEvent::Created(_) | FileEvent::Deleted(_) => EventPriority::High,
            FileEvent::Renamed { .. } | FileEvent::Moved { .. } => EventPriority::High,
            FileEvent::Modified(_) => EventPriority::Normal,
        }
    }
}

/// Prioritized event wrapper
#[derive(Debug, Clone)]
pub struct PrioritizedEvent {
    pub event: FileEvent,
    pub priority: EventPriority,
    pub timestamp: Instant,
    pub retry_count: u32,
}

impl PrioritizedEvent {
    pub fn new(event: FileEvent, priority: EventPriority) -> Self {
        Self {
            event,
            priority,
            timestamp: Instant::now(),
            retry_count: 0,
        }
    }

    /// Check if this event has aged and should be deprioritized
    pub fn should_deprioritize(&self, max_age: Duration) -> bool {
        self.timestamp.elapsed() > max_age && self.priority > EventPriority::Critical
    }

    /// Create a deprioritized version of this event
    pub fn deprioritize(&self) -> Self {
        let new_priority = match self.priority {
            EventPriority::Critical => EventPriority::Critical, // Never deprioritize critical
            EventPriority::High => EventPriority::Normal,
            EventPriority::Normal => EventPriority::Low,
            EventPriority::Low => EventPriority::Background,
            EventPriority::Background => EventPriority::Background,
        };

        Self {
            event: self.event.clone(),
            priority: new_priority,
            timestamp: self.timestamp,
            retry_count: self.retry_count,
        }
    }
}

/// Performance metrics for event processing
#[derive(Debug, Default)]
pub struct EventProcessingMetrics {
    pub events_processed: AtomicU64,
    pub events_dropped: AtomicU64,
    pub backpressure_events: AtomicU64,
    pub average_processing_time_us: AtomicU64,
    pub queue_size_high_water_mark: AtomicUsize,
    pub deduplication_hits: AtomicU64,
    pub batch_count: AtomicU64,
    #[allow(dead_code)] // Used for future priority adjustment features
    pub priority_promotions: AtomicU64,
    pub priority_demotions: AtomicU64,
    #[allow(dead_code)] // Used for future dynamic worker scaling
    pub dynamic_worker_adjustments: AtomicU64,
    pub congestion_events: AtomicU64,
    #[allow(dead_code)] // Used for future congestion control enhancements
    pub slow_start_activations: AtomicU64,
    pub current_worker_count: AtomicUsize,
    pub current_congestion_window: AtomicUsize,
}

impl EventProcessingMetrics {
    pub fn record_event_processed(&self, processing_time: Duration) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);

        // Update rolling average (simplified)
        let new_time_us = processing_time.as_micros() as u64;
        let current_avg = self.average_processing_time_us.load(Ordering::Relaxed);
        let new_avg = (current_avg + new_time_us) / 2;
        self.average_processing_time_us
            .store(new_avg, Ordering::Relaxed);
    }

    pub fn record_event_dropped(&self) {
        self.events_dropped.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_backpressure(&self) {
        self.backpressure_events.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_queue_size(&self, size: usize) {
        let current_max = self.queue_size_high_water_mark.load(Ordering::Relaxed);
        if size > current_max {
            self.queue_size_high_water_mark
                .store(size, Ordering::Relaxed);
        }
    }

    pub fn record_deduplication_hit(&self) {
        self.deduplication_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_batch_processed(&self) {
        self.batch_count.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)] // Used for future priority adjustment features
    pub fn record_priority_promotion(&self) {
        self.priority_promotions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_priority_demotion(&self) {
        self.priority_demotions.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)] // Used for future dynamic worker scaling
    pub fn record_worker_adjustment(&self, new_count: usize) {
        self.dynamic_worker_adjustments
            .fetch_add(1, Ordering::Relaxed);
        self.current_worker_count
            .store(new_count, Ordering::Relaxed);
    }

    pub fn record_congestion_event(&self) {
        self.congestion_events.fetch_add(1, Ordering::Relaxed);
    }

    #[allow(dead_code)] // Used for future congestion control enhancements
    pub fn record_slow_start_activation(&self) {
        self.slow_start_activations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_congestion_window(&self, new_size: usize) {
        self.current_congestion_window
            .store(new_size, Ordering::Relaxed);
    }
}

/// Advanced event processing configuration
#[derive(Debug, Clone)]
pub struct EventProcessorConfig {
    /// Maximum number of events in each priority queue
    pub max_queue_size_per_priority: usize,
    /// Maximum batch size for processing
    pub max_batch_size: usize,
    /// Minimum batch size before processing
    pub min_batch_size: usize,
    /// Maximum time to wait for a batch to fill
    pub batch_timeout: Duration,
    /// Debounce duration for similar events
    pub debounce_duration: Duration,
    /// Maximum age before events are deprioritized
    pub max_event_age: Duration,
    /// Number of concurrent processing workers
    pub worker_count: usize,
    /// Enable event deduplication
    pub enable_deduplication: bool,
    #[allow(dead_code)] // Used for future adaptive batch sizing features
    /// Enable adaptive batch sizing
    pub enable_adaptive_batching: bool,
    /// Backpressure threshold (percentage of queue capacity)
    pub backpressure_threshold: f32,
    /// Enable priority aging (demote old events)
    pub enable_priority_aging: bool,
    /// Interval for priority aging checks
    pub priority_aging_interval: Duration,
    /// Enable dynamic worker pool adjustment
    pub enable_dynamic_workers: bool,
    #[allow(dead_code)] // Used for future dynamic worker scaling
    /// Maximum number of workers for dynamic scaling
    pub max_dynamic_workers: usize,
    #[allow(dead_code)] // Used for future dynamic worker scaling
    /// Worker idle timeout before scaling down
    pub worker_idle_timeout: Duration,
    /// Enable congestion control
    pub enable_congestion_control: bool,
    /// Congestion control window size
    pub congestion_window_size: usize,
    /// Slow start threshold for congestion control
    pub slow_start_threshold: usize,
}

impl Default for EventProcessorConfig {
    fn default() -> Self {
        Self {
            max_queue_size_per_priority: 1000,
            max_batch_size: 100,
            min_batch_size: 5,
            batch_timeout: Duration::from_millis(50),
            debounce_duration: Duration::from_millis(500),
            max_event_age: Duration::from_secs(30),
            worker_count: num_cpus::get().min(8),
            enable_deduplication: true,
            enable_adaptive_batching: true,
            backpressure_threshold: 0.8,
            enable_priority_aging: true,
            priority_aging_interval: Duration::from_secs(10),
            enable_dynamic_workers: true,
            max_dynamic_workers: num_cpus::get() * 2,
            worker_idle_timeout: Duration::from_secs(30),
            enable_congestion_control: true,
            congestion_window_size: 50,
            slow_start_threshold: 25,
        }
    }
}

/// Congestion control state for adaptive event processing
#[derive(Debug)]
struct CongestionControl {
    window_size: usize,
    slow_start_threshold: usize,
    in_slow_start: bool,
    rtt_estimate: Duration,
    last_congestion_time: Instant,
}

impl CongestionControl {
    fn new(initial_window: usize, slow_start_threshold: usize) -> Self {
        Self {
            window_size: initial_window,
            slow_start_threshold,
            in_slow_start: true,
            rtt_estimate: Duration::from_millis(100), // Initial RTT estimate
            last_congestion_time: Instant::now(),
        }
    }

    fn on_success(&mut self, processing_time: Duration) {
        // Update RTT estimate using exponential moving average
        self.rtt_estimate = Duration::from_nanos(
            (self.rtt_estimate.as_nanos() as f64 * 0.875
                + processing_time.as_nanos() as f64 * 0.125) as u64,
        );

        if self.in_slow_start {
            // Exponential growth in slow start
            self.window_size += 1;
            if self.window_size >= self.slow_start_threshold {
                self.in_slow_start = false;
            }
        } else {
            // Linear growth in congestion avoidance
            self.window_size += 1.max(1 / self.window_size);
        }
    }

    fn on_congestion(&mut self) {
        self.slow_start_threshold = self.window_size / 2;
        self.window_size = self.slow_start_threshold;
        self.in_slow_start = false;
        self.last_congestion_time = Instant::now();
    }

    fn should_allow_processing(&self, current_processing: usize) -> bool {
        current_processing < self.window_size
    }
}

/// High-performance event processor with prioritization and backpressure handling
pub struct OptimizedEventProcessor {
    config: EventProcessorConfig,

    // Priority queues for different event priorities
    priority_queues: Arc<RwLock<[VecDeque<PrioritizedEvent>; 5]>>,

    // Event deduplication map (path -> latest event)
    deduplication_map: Arc<RwLock<HashMap<PathBuf, PrioritizedEvent>>>,

    // Performance metrics
    metrics: Arc<EventProcessingMetrics>,

    // Backpressure control
    processing_semaphore: Arc<Semaphore>,

    #[allow(dead_code)] // Infrastructure for future dynamic worker scaling
    // Dynamic worker pool management
    dynamic_semaphore: Arc<RwLock<Option<Arc<Semaphore>>>>,

    // Congestion control
    congestion_control: Arc<RwLock<CongestionControl>>,

    #[allow(dead_code)] // Used for graceful shutdown functionality
    // Shutdown signal
    shutdown_sender: Option<mpsc::UnboundedSender<()>>,
    shutdown_receiver: Option<mpsc::UnboundedReceiver<()>>,
}

impl OptimizedEventProcessor {
    pub fn new(config: EventProcessorConfig) -> Self {
        let (shutdown_sender, shutdown_receiver) = mpsc::unbounded_channel();

        let metrics = Arc::new(EventProcessingMetrics::default());
        metrics
            .current_worker_count
            .store(config.worker_count, Ordering::Relaxed);
        metrics
            .current_congestion_window
            .store(config.congestion_window_size, Ordering::Relaxed);

        Self {
            processing_semaphore: Arc::new(Semaphore::new(config.worker_count)),
            dynamic_semaphore: Arc::new(RwLock::new(None)),
            congestion_control: Arc::new(RwLock::new(CongestionControl::new(
                config.congestion_window_size,
                config.slow_start_threshold,
            ))),
            config,
            priority_queues: Arc::new(RwLock::new([
                VecDeque::new(), // Critical
                VecDeque::new(), // High
                VecDeque::new(), // Normal
                VecDeque::new(), // Low
                VecDeque::new(), // Background
            ])),
            deduplication_map: Arc::new(RwLock::new(HashMap::new())),
            metrics,
            shutdown_sender: Some(shutdown_sender),
            shutdown_receiver: Some(shutdown_receiver),
        }
    }

    /// Start the optimized event processing system
    pub async fn start_processing<F, Fut>(
        &mut self,
        mut event_handler: F,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnMut(Vec<PrioritizedEvent>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
            + Send,
    {
        let priority_queues = self.priority_queues.clone();
        let _deduplication_map = self.deduplication_map.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        let processing_semaphore = self.processing_semaphore.clone();
        let congestion_control = self.congestion_control.clone();

        let mut shutdown_receiver = self
            .shutdown_receiver
            .take()
            .ok_or("Event processor already started")?;

        // Spawn the main processing loop
        tokio::spawn(async move {
            let mut batch_timer = tokio::time::interval(config.batch_timeout);
            let mut aging_timer = if config.enable_priority_aging {
                Some(tokio::time::interval(config.priority_aging_interval))
            } else {
                None
            };

            loop {
                tokio::select! {
                    // Check for shutdown signal
                    _ = shutdown_receiver.recv() => {
                        info!("Event processor shutting down");
                        break;
                    }

                    // Process batches when timer expires
                    _ = batch_timer.tick() => {
                        if let Some(batch) = Self::extract_batch(&priority_queues, &config, &metrics).await {
                            Self::process_batch_async(
                                batch,
                                &mut event_handler,
                                &processing_semaphore,
                                &metrics,
                                &congestion_control,
                                config.enable_congestion_control,
                            ).await;
                        }
                    }

                    // Handle priority aging
                    _ = async {
                        if let Some(ref mut timer) = aging_timer {
                            timer.tick().await
                        } else {
                            std::future::pending().await
                        }
                    } => {
                        Self::age_events(&priority_queues, &config, &metrics).await;
                    }

                    // Handle dynamic worker adjustment
                    _ = tokio::time::sleep(Duration::from_secs(60)) => {
                        if config.enable_dynamic_workers {
                            // Implement dynamic worker adjustment logic here
                            // This would analyze queue sizes and processing times
                            // to decide whether to scale workers up or down
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Submit a raw notify event for processing
    pub async fn submit_raw_event(
        &self,
        raw_event: Event,
        path_validator: &PathValidator,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Convert raw event to file events
        let file_events = self.convert_raw_event(raw_event, path_validator).await?;

        for file_event in file_events {
            self.submit_event(file_event).await?;
        }

        Ok(())
    }

    /// Submit a processed file event for optimized handling
    pub async fn submit_event(
        &self,
        file_event: FileEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let priority = EventPriority::from_path_and_event(file_event.path(), &file_event);
        let prioritized_event = PrioritizedEvent::new(file_event, priority);

        // Check backpressure
        if self.is_under_backpressure().await {
            self.metrics.record_backpressure();
            warn!("Event processor under backpressure, dropping low priority event");

            // Drop low priority events under backpressure
            if priority >= EventPriority::Low {
                self.metrics.record_event_dropped();
                return Ok(());
            }
        }

        // Handle deduplication
        if self.config.enable_deduplication {
            if self.deduplicate_event(&prioritized_event).await {
                self.metrics.record_deduplication_hit();
                return Ok(());
            }
        }

        // Add to appropriate priority queue
        self.enqueue_event(prioritized_event).await?;

        Ok(())
    }

    /// Check if the processor is under backpressure
    async fn is_under_backpressure(&self) -> bool {
        let queues = self.priority_queues.read().await;
        let total_size: usize = queues.iter().map(|q| q.len()).sum();
        let max_total_size = self.config.max_queue_size_per_priority * 5;

        self.metrics.record_queue_size(total_size);

        (total_size as f32) > (max_total_size as f32 * self.config.backpressure_threshold)
    }

    /// Handle event deduplication
    async fn deduplicate_event(&self, event: &PrioritizedEvent) -> bool {
        let path = event.event.path().to_path_buf();
        let mut dedup_map = self.deduplication_map.write().await;

        if let Some(existing_event) = dedup_map.get(&path) {
            // Check if we should replace the existing event
            if event.priority <= existing_event.priority
                || existing_event.timestamp.elapsed() > self.config.debounce_duration
            {
                dedup_map.insert(path, event.clone());
                return false; // Process this event
            } else {
                return true; // Skip this event (duplicate)
            }
        } else {
            dedup_map.insert(path, event.clone());
            return false; // Process this event
        }
    }

    /// Add event to the appropriate priority queue
    async fn enqueue_event(
        &self,
        event: PrioritizedEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut queues = self.priority_queues.write().await;
        let queue_index = event.priority as usize;

        if queues[queue_index].len() >= self.config.max_queue_size_per_priority {
            // Queue is full, drop the event
            self.metrics.record_event_dropped();
            return Err("Priority queue is full".into());
        }

        queues[queue_index].push_back(event);
        Ok(())
    }

    /// Extract a batch of events for processing
    async fn extract_batch(
        priority_queues: &Arc<RwLock<[VecDeque<PrioritizedEvent>; 5]>>,
        config: &EventProcessorConfig,
        metrics: &Arc<EventProcessingMetrics>,
    ) -> Option<Vec<PrioritizedEvent>> {
        let mut queues = priority_queues.write().await;
        let mut batch = Vec::new();

        // Process events in priority order
        for queue in queues.iter_mut() {
            while !queue.is_empty() && batch.len() < config.max_batch_size {
                if let Some(event) = queue.pop_front() {
                    batch.push(event);
                }
            }
            if batch.len() >= config.max_batch_size {
                break;
            }
        }

        if batch.len() >= config.min_batch_size || !batch.is_empty() {
            metrics.record_batch_processed();
            Some(batch)
        } else {
            None
        }
    }

    /// Process a batch of events asynchronously with congestion control
    async fn process_batch_async<F, Fut>(
        batch: Vec<PrioritizedEvent>,
        event_handler: &mut F,
        processing_semaphore: &Arc<Semaphore>,
        metrics: &Arc<EventProcessingMetrics>,
        congestion_control: &Arc<RwLock<CongestionControl>>,
        enable_congestion_control: bool,
    ) where
        F: FnMut(Vec<PrioritizedEvent>) -> Fut,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>,
    {
        // Check congestion control before processing
        if enable_congestion_control {
            let congestion = congestion_control.read().await;
            let current_processing = processing_semaphore.available_permits();
            if !congestion.should_allow_processing(current_processing) {
                metrics.record_congestion_event();
                return; // Skip processing due to congestion
            }
        }

        // Acquire semaphore permit for backpressure control
        let _permit = processing_semaphore.acquire().await;

        let start_time = Instant::now();

        match timeout(Duration::from_secs(5), event_handler(batch)).await {
            Ok(Ok(())) => {
                let processing_time = start_time.elapsed();
                metrics.record_event_processed(processing_time);

                // Update congestion control on success
                if enable_congestion_control {
                    let mut congestion = congestion_control.write().await;
                    congestion.on_success(processing_time);
                    metrics.update_congestion_window(congestion.window_size);
                }
            }
            Ok(Err(e)) => {
                error!("Event processing failed: {}", e);

                // Update congestion control on failure
                if enable_congestion_control {
                    let mut congestion = congestion_control.write().await;
                    congestion.on_congestion();
                    metrics.record_congestion_event();
                    metrics.update_congestion_window(congestion.window_size);
                }
            }
            Err(_) => {
                error!("Event processing timed out");

                // Treat timeout as congestion
                if enable_congestion_control {
                    let mut congestion = congestion_control.write().await;
                    congestion.on_congestion();
                    metrics.record_congestion_event();
                    metrics.update_congestion_window(congestion.window_size);
                }
            }
        }
    }

    /// Age events and potentially demote their priority
    async fn age_events(
        priority_queues: &Arc<RwLock<[VecDeque<PrioritizedEvent>; 5]>>,
        config: &EventProcessorConfig,
        metrics: &Arc<EventProcessingMetrics>,
    ) {
        let mut queues = priority_queues.write().await;
        let mut all_aged_events = Vec::new();

        // First pass: extract aged events from all queues
        for queue in queues.iter_mut() {
            let mut aged_events = Vec::new();

            // Check for events that should be aged
            while let Some(event) = queue.pop_front() {
                if event.should_deprioritize(config.max_event_age) {
                    let aged_event = event.deprioritize();
                    metrics.record_priority_demotion();
                    aged_events.push(aged_event);
                } else {
                    queue.push_front(event);
                    break;
                }
            }

            all_aged_events.extend(aged_events);
        }

        // Second pass: re-queue aged events in their new priority queues
        for aged_event in all_aged_events {
            let new_priority_index = aged_event.priority as usize;
            if new_priority_index < queues.len() {
                queues[new_priority_index].push_back(aged_event);
            }
        }
    }

    /// Convert a raw notify event to file events
    async fn convert_raw_event(
        &self,
        raw_event: Event,
        path_validator: &PathValidator,
    ) -> Result<Vec<FileEvent>, Box<dyn std::error::Error + Send + Sync>> {
        let mut file_events = Vec::new();

        for path in raw_event.paths {
            // Skip ignored files
            if Self::should_ignore_path(&path) {
                continue;
            }

            // Validate path security
            if let Err(e) = path_validator.validate_directory_path(&path) {
                warn!(
                    "Security violation in file watcher: {} - {}",
                    path.display(),
                    e
                );
                continue;
            }

            // Convert to FileEvent
            let file_event = match &raw_event.kind {
                notify::EventKind::Create(_) => FileEvent::Created(path),
                notify::EventKind::Modify(modify_kind) => {
                    use notify::event::{ModifyKind, RenameMode};
                    match modify_kind {
                        ModifyKind::Name(RenameMode::From) => FileEvent::Renamed {
                            from: path,
                            to: PathBuf::new(),
                        },
                        ModifyKind::Name(RenameMode::To) => {
                            // For now, treat as a simple modification
                            // In a full implementation, you'd need to track rename pairs
                            FileEvent::Modified(path)
                        }
                        _ => FileEvent::Modified(path),
                    }
                }
                notify::EventKind::Remove(_) => FileEvent::Deleted(path),
                _ => continue,
            };

            file_events.push(file_event);
        }

        Ok(file_events)
    }

    /// Check if a path should be ignored
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

    /// Get current performance metrics
    pub fn get_metrics(&self) -> &EventProcessingMetrics {
        &self.metrics
    }

    #[allow(dead_code)] // Used for graceful shutdown functionality
    /// Shutdown the event processor
    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(sender) = self.shutdown_sender.take() {
            sender.send(())?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_event_priority_assignment() {
        let path = PathBuf::from("/test/security/config.json");
        let event = FileEvent::Created(path.clone());
        let priority = EventPriority::from_path_and_event(&path, &event);
        assert_eq!(priority, EventPriority::Critical);

        let temp_path = PathBuf::from("/test/file.tmp");
        let temp_event = FileEvent::Modified(temp_path.clone());
        let temp_priority = EventPriority::from_path_and_event(&temp_path, &temp_event);
        assert_eq!(temp_priority, EventPriority::Low);
    }

    #[test]
    fn test_event_aging() {
        let event = PrioritizedEvent::new(
            FileEvent::Created(PathBuf::from("/test.rs")),
            EventPriority::High,
        );

        assert!(!event.should_deprioritize(Duration::from_secs(1)));

        let aged_event = event.deprioritize();
        assert_eq!(aged_event.priority, EventPriority::Normal);
    }

    #[tokio::test]
    async fn test_event_processor_creation() {
        let config = EventProcessorConfig::default();
        let processor = OptimizedEventProcessor::new(config);

        assert_eq!(
            processor.metrics.events_processed.load(Ordering::Relaxed),
            0
        );
    }

    #[tokio::test]
    async fn test_backpressure_detection() {
        let mut config = EventProcessorConfig::default();
        config.max_queue_size_per_priority = 10;
        config.backpressure_threshold = 0.5;

        let processor = OptimizedEventProcessor::new(config);

        // Initially should not be under backpressure
        assert!(!processor.is_under_backpressure().await);
    }

    #[test]
    fn test_priority_ordering() {
        assert!(EventPriority::Critical < EventPriority::High);
        assert!(EventPriority::High < EventPriority::Normal);
        assert!(EventPriority::Normal < EventPriority::Low);
        assert!(EventPriority::Low < EventPriority::Background);
    }
}
