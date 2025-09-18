// src-tauri/src/filesystem/health_monitor.rs
// Comprehensive health monitoring and recovery system for file watchers

use crate::resource_monitor::{ResourceMonitor, ResourceSnapshot};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};

/// Health status of a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is operating normally
    Healthy,
    /// Component is experiencing issues but still functional
    Degraded,
    /// Component is not functioning properly
    Unhealthy,
    /// Component has failed and requires intervention
    Critical,
    /// Component status is unknown or not being monitored
    Unknown,
}

impl HealthStatus {
    pub fn is_healthy(self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }

    #[allow(dead_code)] // Used for operational status checks in future health policies
    pub fn is_operational(self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    pub fn requires_action(self) -> bool {
        matches!(self, HealthStatus::Unhealthy | HealthStatus::Critical)
    }
}

/// Different types of health checks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthCheckType {
    /// File watcher responsiveness
    WatcherResponsiveness,
    /// Event processing performance
    EventProcessing,
    /// Resource usage (CPU, memory, file handles)
    ResourceUsage,
    /// Queue health (backlog, processing rate)
    QueueHealth,
    /// Error rate monitoring
    ErrorRate,
    /// System connectivity and file system access
    SystemConnectivity,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_type: HealthCheckType,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
    pub recovery_actions: Vec<String>,
}

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// Circuit is closed, operations flow normally
    Closed,
    /// Circuit is open, operations are blocked
    Open,
    /// Circuit is half-open, testing if operations can resume
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    #[allow(dead_code)] // Used for failure threshold configuration in circuit breaker logic
    /// Number of failures to trigger circuit opening
    pub failure_threshold: usize,
    #[allow(dead_code)] // Used for time window calculations in failure detection
    /// Time window for counting failures
    pub failure_window: Duration,
    #[allow(dead_code)] // Used for recovery timeout logic in circuit breaker
    /// Time to wait before attempting recovery
    pub recovery_timeout: Duration,
    #[allow(dead_code)] // Used for success threshold configuration in circuit breaker logic
    /// Number of successful operations to close circuit
    pub success_threshold: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_secs(300),
            success_threshold: 3,
        }
    }
}

/// Circuit breaker for automatic failure handling
#[derive(Debug)]
pub struct CircuitBreaker {
    #[allow(dead_code)] // Used for circuit breaker configuration and thresholds
    config: CircuitBreakerConfig,
    state: AtomicU64, // Encoded CircuitBreakerState
    failure_count: AtomicUsize,
    success_count: AtomicUsize,
    #[allow(dead_code)] // Used for recovery timeout calculations
    last_failure_time: RwLock<Option<Instant>>,
    #[allow(dead_code)] // Used for time window-based failure tracking
    failure_times: RwLock<VecDeque<Instant>>,
}

impl CircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: AtomicU64::new(CircuitBreakerState::Closed as u64),
            failure_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            last_failure_time: RwLock::new(None),
            failure_times: RwLock::new(VecDeque::new()),
        }
    }

    pub fn state(&self) -> CircuitBreakerState {
        match self.state.load(Ordering::Relaxed) {
            0 => CircuitBreakerState::Closed,
            1 => CircuitBreakerState::Open,
            2 => CircuitBreakerState::HalfOpen,
            _ => CircuitBreakerState::Closed,
        }
    }

    #[allow(dead_code)] // Used for wrapping operations with circuit breaker protection
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        match self.state() {
            CircuitBreakerState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    if last_failure.elapsed() >= self.config.recovery_timeout {
                        self.state
                            .store(CircuitBreakerState::HalfOpen as u64, Ordering::Relaxed);
                        self.success_count.store(0, Ordering::Relaxed);
                    } else {
                        return Err(CircuitBreakerError::CircuitOpen);
                    }
                }
            }
            CircuitBreakerState::HalfOpen | CircuitBreakerState::Closed => {
                // Allow operation
            }
        }

        // Execute operation
        match operation() {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(error) => {
                self.on_failure().await;
                Err(CircuitBreakerError::Operation(error))
            }
        }
    }

    #[allow(dead_code)] // Used for circuit breaker success state management
    async fn on_success(&self) {
        let current_state = self.state();

        match current_state {
            CircuitBreakerState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;
                if success_count >= self.config.success_threshold {
                    self.state
                        .store(CircuitBreakerState::Closed as u64, Ordering::Relaxed);
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                    info!("Circuit breaker closed after successful recovery");
                }
            }
            CircuitBreakerState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::Relaxed);
            }
            CircuitBreakerState::Open => {
                // Should not reach here
            }
        }
    }

    #[allow(dead_code)] // Used for circuit breaker failure state management
    async fn on_failure(&self) {
        let now = Instant::now();

        // Add failure timestamp
        {
            let mut failure_times = self.failure_times.write().await;
            failure_times.push_back(now);

            // Remove old failures outside the window
            while let Some(&front_time) = failure_times.front() {
                if now.duration_since(front_time) > self.config.failure_window {
                    failure_times.pop_front();
                } else {
                    break;
                }
            }
        }

        *self.last_failure_time.write().await = Some(now);
        let _failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        // Check if we should open the circuit
        let failure_times = self.failure_times.read().await;
        if failure_times.len() >= self.config.failure_threshold {
            self.state
                .store(CircuitBreakerState::Open as u64, Ordering::Relaxed);
            warn!(
                "Circuit breaker opened due to {} failures in {}s",
                failure_times.len(),
                self.config.failure_window.as_secs()
            );
        }
    }

    pub fn metrics(&self) -> CircuitBreakerMetrics {
        CircuitBreakerMetrics {
            state: self.state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CircuitBreakerMetrics {
    pub state: CircuitBreakerState,
    pub failure_count: usize,
    pub success_count: usize,
}

/// Circuit breaker error types
#[derive(Debug)]
#[allow(dead_code)] // Used for circuit breaker error handling and operation wrapping
pub enum CircuitBreakerError<E> {
    /// Circuit is open, operation blocked
    CircuitOpen,
    /// The wrapped operation failed
    Operation(E),
}

impl<E: std::fmt::Display> std::fmt::Display for CircuitBreakerError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitBreakerError::CircuitOpen => write!(f, "Circuit breaker is open"),
            CircuitBreakerError::Operation(e) => write!(f, "Operation failed: {}", e),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for CircuitBreakerError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CircuitBreakerError::CircuitOpen => None,
            CircuitBreakerError::Operation(e) => Some(e),
        }
    }
}

/// Health monitoring configuration
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    /// Interval between health checks
    pub check_interval: Duration,
    #[allow(dead_code)] // Used for health check timeout configuration
    /// Maximum response time before marking as unhealthy
    pub response_timeout: Duration,
    /// Number of health check results to keep in history
    pub history_size: usize,
    /// Thresholds for different health metrics
    pub cpu_threshold: f64,
    pub memory_threshold: f64,
    pub error_rate_threshold: f64,
    #[allow(dead_code)] // Used for queue health monitoring thresholds
    pub queue_size_threshold: usize,
    /// Enable automatic recovery attempts
    pub enable_auto_recovery: bool,
    /// Circuit breaker configuration
    pub circuit_breaker_config: CircuitBreakerConfig,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            response_timeout: Duration::from_secs(10),
            history_size: 100,
            cpu_threshold: 80.0,
            memory_threshold: 85.0,
            error_rate_threshold: 5.0,
            queue_size_threshold: 1000,
            enable_auto_recovery: true,
            circuit_breaker_config: CircuitBreakerConfig::default(),
        }
    }
}

/// Recovery action types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryAction {
    /// Restart the file watcher
    RestartWatcher,
    /// Clear event queues
    ClearQueues,
    /// Reduce monitoring frequency
    ReduceFrequency,
    /// Notify administrators
    NotifyAdmin,
    /// Garbage collection
    GarbageCollection,
    /// Reset connections
    ResetConnections,
}

/// Comprehensive health monitor for file watcher system
pub struct WatcherHealthMonitor {
    config: HealthMonitorConfig,

    // Circuit breakers for different components
    watcher_circuit_breaker: Arc<CircuitBreaker>,
    processor_circuit_breaker: Arc<CircuitBreaker>,

    // Health check history
    health_history: Arc<RwLock<VecDeque<HealthCheckResult>>>,

    // Metrics (wrapped in Arc for safe sharing across threads)
    total_checks: Arc<AtomicU64>,
    healthy_checks: Arc<AtomicU64>,
    recovery_attempts: Arc<AtomicU64>,

    // Control
    is_monitoring: AtomicBool,
    #[allow(dead_code)] // Used for graceful shutdown functionality
    shutdown_sender: Option<mpsc::UnboundedSender<()>>,
}

impl WatcherHealthMonitor {
    pub fn new(config: HealthMonitorConfig) -> Self {
        let (shutdown_sender, _) = mpsc::unbounded_channel();

        Self {
            watcher_circuit_breaker: Arc::new(CircuitBreaker::new(
                config.circuit_breaker_config.clone(),
            )),
            processor_circuit_breaker: Arc::new(CircuitBreaker::new(
                config.circuit_breaker_config.clone(),
            )),
            config,
            health_history: Arc::new(RwLock::new(VecDeque::new())),
            total_checks: Arc::new(AtomicU64::new(0)),
            healthy_checks: Arc::new(AtomicU64::new(0)),
            recovery_attempts: Arc::new(AtomicU64::new(0)),
            is_monitoring: AtomicBool::new(false),
            shutdown_sender: Some(shutdown_sender),
        }
    }

    /// Start health monitoring background task
    pub async fn start_monitoring(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_monitoring.swap(true, Ordering::Relaxed) {
            return Err("Health monitoring is already running".into());
        }

        let config = self.config.clone();
        let health_history = self.health_history.clone();
        let total_checks = Arc::clone(&self.total_checks);
        let healthy_checks = Arc::clone(&self.healthy_checks);
        let recovery_attempts = Arc::clone(&self.recovery_attempts);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(config.check_interval);

            loop {
                interval.tick().await;

                // Create simple health check results (in a real implementation,
                // this would use actual callbacks or channels to get health data)
                let results = vec![HealthCheckResult {
                    check_type: HealthCheckType::SystemConnectivity,
                    status: HealthStatus::Healthy,
                    message: "System connectivity check passed".to_string(),
                    timestamp: Utc::now(),
                    details: serde_json::json!({}),
                    recovery_actions: vec![],
                }];

                total_checks.fetch_add(1, Ordering::Relaxed);

                let mut all_healthy = true;
                let mut recovery_needed = Vec::new();

                for result in results {
                    if !result.status.is_healthy() {
                        all_healthy = false;

                        if result.status.requires_action() {
                            recovery_needed.extend(result.recovery_actions.clone());
                        }
                    }

                    // Add to history
                    let mut history = health_history.write().await;
                    history.push_back(result);

                    // Maintain history size
                    while history.len() > config.history_size {
                        history.pop_front();
                    }
                }

                if all_healthy {
                    healthy_checks.fetch_add(1, Ordering::Relaxed);
                } else if config.enable_auto_recovery && !recovery_needed.is_empty() {
                    recovery_attempts.fetch_add(1, Ordering::Relaxed);
                    info!("Attempting automatic recovery: {:?}", recovery_needed);

                    // TODO: Implement actual recovery actions
                    // This would integrate with the watcher system to perform recovery
                }
            }
        });

        Ok(())
    }

    /// Perform immediate health check
    pub async fn check_health(
        &self,
        resource_monitor: Option<&ResourceMonitor>,
    ) -> Vec<HealthCheckResult> {
        let mut results = Vec::new();
        let now = Utc::now();

        // Check resource usage
        if let Some(monitor) = resource_monitor {
            if let Some(snapshot) = monitor.get_latest_snapshot().await {
                results.push(self.check_resource_health(&snapshot, now));
            }
        }

        // Check watcher responsiveness
        results.push(self.check_watcher_responsiveness(now).await);

        // Check circuit breaker states
        results.push(self.check_circuit_breakers(now));

        // Check error rates
        results.push(self.check_error_rates(now).await);

        results
    }

    fn check_resource_health(
        &self,
        snapshot: &ResourceSnapshot,
        timestamp: DateTime<Utc>,
    ) -> HealthCheckResult {
        let cpu_usage = snapshot.cpu_percent;
        let memory_usage_mb = snapshot.memory_kb as f64 / 1024.0;
        // Assume 8GB system for percentage calculation (this should come from system info in practice)
        let memory_usage = (memory_usage_mb / 8192.0) * 100.0;

        let status = if cpu_usage > self.config.cpu_threshold
            || memory_usage > self.config.memory_threshold
        {
            if cpu_usage > 95.0 || memory_usage > 95.0 {
                HealthStatus::Critical
            } else if cpu_usage > 90.0 || memory_usage > 90.0 {
                HealthStatus::Unhealthy
            } else {
                HealthStatus::Degraded
            }
        } else {
            HealthStatus::Healthy
        };

        let recovery_actions = if status.requires_action() {
            vec![
                "GarbageCollection".to_string(),
                "ReduceFrequency".to_string(),
            ]
        } else {
            vec![]
        };

        HealthCheckResult {
            check_type: HealthCheckType::ResourceUsage,
            status,
            message: format!("CPU: {:.1}%, Memory: {:.1}%", cpu_usage, memory_usage),
            timestamp,
            details: serde_json::json!({
                "cpu_usage": cpu_usage,
                "memory_usage": memory_usage,
                "file_descriptors": snapshot.file_descriptors,
                "watcher_file_descriptors": snapshot.watcher_file_descriptors,
                "watched_paths_count": snapshot.watched_paths_count
            }),
            recovery_actions,
        }
    }

    async fn check_watcher_responsiveness(&self, timestamp: DateTime<Utc>) -> HealthCheckResult {
        // This would involve testing actual watcher responsiveness
        // For now, we'll simulate based on circuit breaker state

        let watcher_state = self.watcher_circuit_breaker.state();
        let processor_state = self.processor_circuit_breaker.state();

        let status = match (watcher_state, processor_state) {
            (CircuitBreakerState::Closed, CircuitBreakerState::Closed) => HealthStatus::Healthy,
            (CircuitBreakerState::HalfOpen, _) | (_, CircuitBreakerState::HalfOpen) => {
                HealthStatus::Degraded
            }
            (CircuitBreakerState::Open, _) | (_, CircuitBreakerState::Open) => {
                HealthStatus::Critical
            }
        };

        let recovery_actions = if status.requires_action() {
            vec![
                "RestartWatcher".to_string(),
                "ClearQueues".to_string(),
                "NotifyAdmin".to_string(),
            ]
        } else {
            vec![]
        };

        HealthCheckResult {
            check_type: HealthCheckType::WatcherResponsiveness,
            status,
            message: format!(
                "Watcher: {:?}, Processor: {:?}",
                watcher_state, processor_state
            ),
            timestamp,
            details: serde_json::json!({
                "watcher_circuit_state": watcher_state,
                "processor_circuit_state": processor_state
            }),
            recovery_actions,
        }
    }

    fn check_circuit_breakers(&self, timestamp: DateTime<Utc>) -> HealthCheckResult {
        let watcher_metrics = self.watcher_circuit_breaker.metrics();
        let processor_metrics = self.processor_circuit_breaker.metrics();

        let status = match (watcher_metrics.state, processor_metrics.state) {
            (CircuitBreakerState::Closed, CircuitBreakerState::Closed) => HealthStatus::Healthy,
            (CircuitBreakerState::HalfOpen, _) | (_, CircuitBreakerState::HalfOpen) => {
                HealthStatus::Degraded
            }
            (CircuitBreakerState::Open, _) | (_, CircuitBreakerState::Open) => {
                HealthStatus::Unhealthy
            }
        };

        HealthCheckResult {
            check_type: HealthCheckType::ErrorRate,
            status,
            message: format!("Circuit breakers operational"),
            timestamp,
            details: serde_json::json!({
                "watcher_circuit": watcher_metrics,
                "processor_circuit": processor_metrics
            }),
            recovery_actions: if status.requires_action() {
                vec!["ResetConnections".to_string(), "RestartWatcher".to_string()]
            } else {
                vec![]
            },
        }
    }

    async fn check_error_rates(&self, timestamp: DateTime<Utc>) -> HealthCheckResult {
        // Calculate error rate from recent health history
        let history = self.health_history.read().await;
        let recent_checks: Vec<_> = history
            .iter()
            .filter(|result| {
                let age = timestamp.signed_duration_since(result.timestamp);
                age.num_minutes() <= 5
            })
            .collect();

        let error_rate = if recent_checks.is_empty() {
            0.0
        } else {
            let errors = recent_checks
                .iter()
                .filter(|result| !result.status.is_healthy())
                .count();
            (errors as f64 / recent_checks.len() as f64) * 100.0
        };

        let status = if error_rate > self.config.error_rate_threshold {
            if error_rate > 50.0 {
                HealthStatus::Critical
            } else if error_rate > 20.0 {
                HealthStatus::Unhealthy
            } else {
                HealthStatus::Degraded
            }
        } else {
            HealthStatus::Healthy
        };

        HealthCheckResult {
            check_type: HealthCheckType::ErrorRate,
            status,
            message: format!("Error rate: {:.1}% over last 5 minutes", error_rate),
            timestamp,
            details: serde_json::json!({
                "error_rate": error_rate,
                "recent_checks": recent_checks.len(),
                "threshold": self.config.error_rate_threshold
            }),
            recovery_actions: if status.requires_action() {
                vec!["NotifyAdmin".to_string(), "ReduceFrequency".to_string()]
            } else {
                vec![]
            },
        }
    }

    /// Get overall health status
    pub async fn get_overall_health(&self) -> HealthStatus {
        let history = self.health_history.read().await;

        if history.is_empty() {
            return HealthStatus::Unknown;
        }

        // Get recent results (last 5 minutes)
        let now = Utc::now();
        let recent: Vec<_> = history
            .iter()
            .filter(|result| {
                let age = now.signed_duration_since(result.timestamp);
                age.num_minutes() <= 5
            })
            .collect();

        if recent.is_empty() {
            return HealthStatus::Unknown;
        }

        // Determine worst status from recent checks
        let worst_status = recent
            .iter()
            .map(|result| result.status)
            .max_by_key(|&status| match status {
                HealthStatus::Critical => 4,
                HealthStatus::Unhealthy => 3,
                HealthStatus::Degraded => 2,
                HealthStatus::Healthy => 1,
                HealthStatus::Unknown => 0,
            })
            .unwrap_or(HealthStatus::Unknown);

        worst_status
    }

    /// Get health metrics
    pub fn get_metrics(&self) -> HealthMetrics {
        let total = self.total_checks.load(Ordering::Relaxed);
        let healthy = self.healthy_checks.load(Ordering::Relaxed);

        HealthMetrics {
            total_checks: total,
            healthy_checks: healthy,
            health_percentage: if total > 0 {
                (healthy as f64 / total as f64) * 100.0
            } else {
                100.0
            },
            recovery_attempts: self.recovery_attempts.load(Ordering::Relaxed),
            is_monitoring: self.is_monitoring.load(Ordering::Relaxed),
        }
    }

    /// Get recent health history
    pub async fn get_health_history(&self, limit: Option<usize>) -> Vec<HealthCheckResult> {
        let history = self.health_history.read().await;
        let limit = limit.unwrap_or(self.config.history_size);

        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get circuit breaker for watcher operations
    pub fn get_watcher_circuit_breaker(&self) -> Arc<CircuitBreaker> {
        self.watcher_circuit_breaker.clone()
    }

    /// Get circuit breaker for processor operations
    pub fn get_processor_circuit_breaker(&self) -> Arc<CircuitBreaker> {
        self.processor_circuit_breaker.clone()
    }
}

#[derive(Debug, Serialize)]
pub struct HealthMetrics {
    pub total_checks: u64,
    pub healthy_checks: u64,
    pub health_percentage: f64,
    pub recovery_attempts: u64,
    pub is_monitoring: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_classification() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Degraded.is_healthy());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(HealthStatus::Critical.requires_action());
    }

    #[tokio::test]
    async fn test_circuit_breaker_basic_flow() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            failure_window: Duration::from_secs(60),
            recovery_timeout: Duration::from_millis(100),
            success_threshold: 1,
        };

        let circuit_breaker = CircuitBreaker::new(config);

        // Should start closed
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Closed);

        // Trigger failures
        let _ = circuit_breaker.call(|| Err::<(), _>("error")).await;
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Closed);

        let _ = circuit_breaker.call(|| Err::<(), _>("error")).await;
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Open);

        // Should reject calls when open
        let result = circuit_breaker.call(|| Ok::<(), &str>(())).await;
        assert!(matches!(result, Err(CircuitBreakerError::CircuitOpen)));

        // Wait for recovery timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should allow one call in half-open
        let result = circuit_breaker.call(|| Ok::<(), &str>(())).await;
        assert!(result.is_ok());

        // Should be closed after success
        assert_eq!(circuit_breaker.state(), CircuitBreakerState::Closed);
    }

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let config = HealthMonitorConfig::default();
        let monitor = WatcherHealthMonitor::new(config);

        let metrics = monitor.get_metrics();
        assert_eq!(metrics.total_checks, 0);
        assert_eq!(metrics.healthy_checks, 0);
        assert!(!metrics.is_monitoring);
    }
}
