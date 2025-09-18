// src-tauri/src/resource_monitor.rs
// Enhanced resource usage monitoring for file watching operations

use crate::memory_monitor::{MemoryMonitor, MemoryMonitorConfig, MemoryStats};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// CPU usage statistics for file watching operations
#[derive(Debug, Default)]
pub struct CpuStats {
    pub current_cpu_percent: AtomicU64, // Stored as u64 * 100 to avoid floats
    pub peak_cpu_percent: AtomicU64,
    pub sample_count: AtomicU64,
    pub total_cpu_time_ms: AtomicU64,
}

impl CpuStats {
    /// Update CPU statistics with current usage (percentage * 100)
    pub fn update(&self, cpu_percent_x100: u64, cpu_time_ms: u64) {
        self.current_cpu_percent
            .store(cpu_percent_x100, Ordering::Relaxed);

        let current_peak = self.peak_cpu_percent.load(Ordering::Relaxed);
        if cpu_percent_x100 > current_peak {
            self.peak_cpu_percent
                .store(cpu_percent_x100, Ordering::Relaxed);
        }

        self.sample_count.fetch_add(1, Ordering::Relaxed);
        self.total_cpu_time_ms
            .fetch_add(cpu_time_ms, Ordering::Relaxed);
    }

    /// Get current CPU usage as percentage (0-100)
    #[allow(dead_code)]
    pub fn current_cpu_percent(&self) -> f64 {
        self.current_cpu_percent.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// Get peak CPU usage as percentage (0-100)
    #[allow(dead_code)]
    pub fn peak_cpu_percent(&self) -> f64 {
        self.peak_cpu_percent.load(Ordering::Relaxed) as f64 / 100.0
    }

    /// Get total number of samples taken
    #[allow(dead_code)]
    pub fn sample_count(&self) -> u64 {
        self.sample_count.load(Ordering::Relaxed)
    }

    /// Get total CPU time in milliseconds
    #[allow(dead_code)]
    pub fn total_cpu_time_ms(&self) -> u64 {
        self.total_cpu_time_ms.load(Ordering::Relaxed)
    }
}

/// File descriptor usage statistics
#[derive(Debug, Default)]
pub struct FileDescriptorStats {
    pub current_fd_count: AtomicU64,
    pub peak_fd_count: AtomicU64,
    pub sample_count: AtomicU64,
    pub watcher_fd_count: AtomicU64, // File descriptors specifically for watchers
}

impl FileDescriptorStats {
    /// Update file descriptor statistics
    pub fn update(&self, total_fd_count: u64, watcher_fd_count: u64) {
        self.current_fd_count
            .store(total_fd_count, Ordering::Relaxed);
        self.watcher_fd_count
            .store(watcher_fd_count, Ordering::Relaxed);

        let current_peak = self.peak_fd_count.load(Ordering::Relaxed);
        if total_fd_count > current_peak {
            self.peak_fd_count.store(total_fd_count, Ordering::Relaxed);
        }

        self.sample_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current file descriptor count
    #[allow(dead_code)]
    pub fn current_fd_count(&self) -> u64 {
        self.current_fd_count.load(Ordering::Relaxed)
    }

    /// Get peak file descriptor count
    #[allow(dead_code)]
    pub fn peak_fd_count(&self) -> u64 {
        self.peak_fd_count.load(Ordering::Relaxed)
    }

    /// Get current watcher file descriptor count
    #[allow(dead_code)]
    pub fn watcher_fd_count(&self) -> u64 {
        self.watcher_fd_count.load(Ordering::Relaxed)
    }

    /// Get total number of samples taken
    #[allow(dead_code)]
    pub fn sample_count(&self) -> u64 {
        self.sample_count.load(Ordering::Relaxed)
    }
}

/// Combined resource usage snapshot for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub timestamp: u64,
    pub memory_kb: u64,
    pub cpu_percent: f64,
    pub file_descriptors: u64,
    pub watcher_file_descriptors: u64,
    pub watched_paths_count: usize,
}

/// Resource monitoring thresholds and configuration
#[derive(Debug, Clone)]
pub struct ResourceMonitorConfig {
    pub sampling_interval_secs: u64,
    pub memory_config: MemoryMonitorConfig,

    // CPU thresholds (percentage)
    pub cpu_warning_threshold: f64,
    pub cpu_critical_threshold: f64,

    // File descriptor thresholds
    pub fd_warning_threshold: u64,
    pub fd_critical_threshold: u64,

    // History tracking
    pub max_history_size: usize,
    pub enable_history: bool,
}

impl Default for ResourceMonitorConfig {
    fn default() -> Self {
        Self {
            sampling_interval_secs: 30,
            memory_config: MemoryMonitorConfig::default(),
            cpu_warning_threshold: 70.0,  // 70% CPU usage warning
            cpu_critical_threshold: 90.0, // 90% CPU usage critical
            fd_warning_threshold: 1000,   // 1000 file descriptors warning
            fd_critical_threshold: 2000,  // 2000 file descriptors critical
            max_history_size: 100,        // Keep last 100 samples
            enable_history: true,
        }
    }
}

// Global resource statistics
static CPU_STATS: Lazy<CpuStats> = Lazy::new(CpuStats::default);
static FD_STATS: Lazy<FileDescriptorStats> = Lazy::new(FileDescriptorStats::default);

/// Enhanced resource monitor that combines memory, CPU, and file descriptor monitoring
pub struct ResourceMonitor {
    config: ResourceMonitorConfig,
    memory_monitor: MemoryMonitor,
    last_sample_time: Mutex<Instant>,
    resource_history: Mutex<VecDeque<ResourceSnapshot>>,
    last_cpu_time: Mutex<Option<u64>>,
    last_cpu_sample: Mutex<Option<Instant>>,
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceMonitor {
    /// Create a new resource monitor with default configuration
    pub fn new() -> Self {
        Self::with_config(ResourceMonitorConfig::default())
    }

    /// Create a new resource monitor with custom configuration
    pub fn with_config(config: ResourceMonitorConfig) -> Self {
        let memory_monitor = MemoryMonitor::with_config(config.memory_config.clone());

        Self {
            config,
            memory_monitor,
            last_sample_time: Mutex::new(Instant::now()),
            resource_history: Mutex::new(VecDeque::new()),
            last_cpu_time: Mutex::new(None),
            last_cpu_sample: Mutex::new(None),
        }
    }

    /// Sample all resource usage metrics
    pub async fn sample_resources(
        &self,
        watched_paths_count: usize,
    ) -> Result<ResourceSnapshot, String> {
        let mut last_sample_time = self.last_sample_time.lock().await;
        let now = Instant::now();

        if now.duration_since(*last_sample_time)
            < Duration::from_secs(self.config.sampling_interval_secs)
        {
            return Err("Sampling interval not reached".to_string());
        }

        // Sample memory
        self.memory_monitor.sample_memory_usage().await?;
        let memory_kb = MemoryMonitor::get_memory_stats().current_memory_kb();

        // Sample CPU
        let cpu_percent = self.sample_cpu_usage().await?;

        // Sample file descriptors
        let (total_fds, watcher_fds) = self.sample_file_descriptors().await?;

        // Create snapshot
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| format!("Time error: {}", e))?
            .as_millis() as u64;

        let snapshot = ResourceSnapshot {
            timestamp,
            memory_kb,
            cpu_percent,
            file_descriptors: total_fds,
            watcher_file_descriptors: watcher_fds,
            watched_paths_count,
        };

        // Update statistics
        CPU_STATS.update((cpu_percent * 100.0) as u64, 0); // CPU time tracking would need more complex implementation
        FD_STATS.update(total_fds, watcher_fds);

        // Check thresholds and log warnings
        self.check_thresholds(&snapshot).await;

        // Add to history if enabled
        if self.config.enable_history {
            let mut history = self.resource_history.lock().await;
            history.push_back(snapshot.clone());

            // Keep only the last N samples
            while history.len() > self.config.max_history_size {
                history.pop_front();
            }
        }

        *last_sample_time = now;
        Ok(snapshot)
    }

    /// Sample CPU usage for the current process
    async fn sample_cpu_usage(&self) -> Result<f64, String> {
        #[cfg(target_os = "linux")]
        {
            self.get_cpu_usage_linux().await
        }
        #[cfg(target_os = "windows")]
        {
            self.get_cpu_usage_windows().await
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Ok(0.0) // Fallback for unsupported platforms
        }
    }

    /// Get CPU usage on Linux by reading /proc/self/stat
    #[cfg(target_os = "linux")]
    async fn get_cpu_usage_linux(&self) -> Result<f64, String> {
        use std::fs;

        let stat_content = fs::read_to_string("/proc/self/stat")
            .map_err(|e| format!("Failed to read /proc/self/stat: {}", e))?;

        let fields: Vec<&str> = stat_content.split_whitespace().collect();
        if fields.len() < 17 {
            return Err("Invalid /proc/self/stat format".to_string());
        }

        // Fields 13 and 14 are utime and stime (user and system CPU time in clock ticks)
        let utime: u64 = fields[13]
            .parse()
            .map_err(|e| format!("Failed to parse utime: {}", e))?;
        let stime: u64 = fields[14]
            .parse()
            .map_err(|e| format!("Failed to parse stime: {}", e))?;

        let total_time = utime + stime;

        let mut last_cpu_time = self.last_cpu_time.lock().await;
        let mut last_cpu_sample = self.last_cpu_sample.lock().await;
        let now = Instant::now();

        let cpu_percent =
            if let (Some(last_time), Some(last_sample)) = (*last_cpu_time, *last_cpu_sample) {
                let time_diff = total_time.saturating_sub(last_time);
                let duration_secs = now.duration_since(last_sample).as_secs_f64();

                if duration_secs > 0.0 {
                    // Convert clock ticks to seconds (assuming 100 ticks per second on most systems)
                    let clock_ticks_per_sec = 100.0;
                    let cpu_time_secs = time_diff as f64 / clock_ticks_per_sec;
                    let cpu_percent = (cpu_time_secs / duration_secs) * 100.0;
                    cpu_percent.min(100.0) // Cap at 100%
                } else {
                    0.0
                }
            } else {
                0.0 // First sample
            };

        *last_cpu_time = Some(total_time);
        *last_cpu_sample = Some(now);

        Ok(cpu_percent)
    }

    /// Get CPU usage on Windows using WinAPI
    #[cfg(target_os = "windows")]
    async fn get_cpu_usage_windows(&self) -> Result<f64, String> {
        // This would require implementing Windows-specific CPU monitoring
        // For now, return 0.0 as a placeholder
        Ok(0.0)
    }

    /// Sample file descriptor usage
    async fn sample_file_descriptors(&self) -> Result<(u64, u64), String> {
        #[cfg(target_os = "linux")]
        {
            self.get_file_descriptors_linux().await
        }
        #[cfg(target_os = "windows")]
        {
            self.get_file_descriptors_windows().await
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Ok((0, 0)) // Fallback for unsupported platforms
        }
    }

    /// Get file descriptor count on Linux by reading /proc/self/fd
    #[cfg(target_os = "linux")]
    async fn get_file_descriptors_linux(&self) -> Result<(u64, u64), String> {
        use std::fs;

        let fd_dir = fs::read_dir("/proc/self/fd")
            .map_err(|e| format!("Failed to read /proc/self/fd: {}", e))?;

        let total_fds = fd_dir.count() as u64;

        // For watcher FDs, we would need to analyze the file descriptor types
        // This is a simplified implementation that estimates based on inotify usage
        let watcher_fds = self.estimate_watcher_fds().await;

        Ok((total_fds, watcher_fds))
    }

    /// Get file descriptor count on Windows using handle enumeration
    #[cfg(target_os = "windows")]
    async fn get_file_descriptors_windows(&self) -> Result<(u64, u64), String> {
        // This would require implementing Windows-specific handle monitoring
        // For now, return 0 as a placeholder
        Ok((0, 0))
    }

    /// Estimate the number of file descriptors used by watchers
    async fn estimate_watcher_fds(&self) -> u64 {
        // This is a simplified estimation
        // In a real implementation, you might track this more precisely
        // by counting inotify instances or similar platform-specific mechanisms

        // For now, assume 1 FD per watcher instance plus some overhead
        1 // Base estimation
    }

    /// Check resource thresholds and log appropriate warnings
    async fn check_thresholds(&self, snapshot: &ResourceSnapshot) {
        // Check memory thresholds (handled by memory monitor)

        // Check CPU thresholds
        if snapshot.cpu_percent > self.config.cpu_critical_threshold {
            error!(
                "CRITICAL: CPU usage exceeded critical threshold: {:.2}% > {:.2}%",
                snapshot.cpu_percent, self.config.cpu_critical_threshold
            );
        } else if snapshot.cpu_percent > self.config.cpu_warning_threshold {
            warn!(
                "WARNING: CPU usage exceeded warning threshold: {:.2}% > {:.2}%",
                snapshot.cpu_percent, self.config.cpu_warning_threshold
            );
        }

        // Check file descriptor thresholds
        if snapshot.file_descriptors > self.config.fd_critical_threshold {
            error!(
                "CRITICAL: File descriptor usage exceeded critical threshold: {} > {}",
                snapshot.file_descriptors, self.config.fd_critical_threshold
            );
        } else if snapshot.file_descriptors > self.config.fd_warning_threshold {
            warn!(
                "WARNING: File descriptor usage exceeded warning threshold: {} > {}",
                snapshot.file_descriptors, self.config.fd_warning_threshold
            );
        }

        // Log normal resource usage
        if snapshot.cpu_percent <= self.config.cpu_warning_threshold
            && snapshot.file_descriptors <= self.config.fd_warning_threshold
        {
            info!(
                "Resource usage: Memory: {}KB, CPU: {:.2}%, FDs: {} (Watchers: {}), Paths: {}",
                snapshot.memory_kb,
                snapshot.cpu_percent,
                snapshot.file_descriptors,
                snapshot.watcher_file_descriptors,
                snapshot.watched_paths_count
            );
        }
    }

    /// Get resource usage history
    #[allow(dead_code)]
    pub async fn get_resource_history(&self) -> Vec<ResourceSnapshot> {
        self.resource_history.lock().await.iter().cloned().collect()
    }

    /// Get current resource statistics
    #[allow(dead_code)]
    pub async fn get_current_stats(&self) -> (MemoryStats, CpuStats, FileDescriptorStats) {
        (
            MemoryStats::default(), // Would need to expose from memory monitor
            CpuStats::default(),
            FileDescriptorStats::default(),
        )
    }

    /// Get global CPU statistics
    #[allow(dead_code)]
    pub fn get_cpu_stats() -> &'static CpuStats {
        &CPU_STATS
    }

    /// Get global file descriptor statistics
    #[allow(dead_code)]
    pub fn get_fd_stats() -> &'static FileDescriptorStats {
        &FD_STATS
    }

    /// Clear resource history
    #[allow(dead_code)]
    pub async fn clear_history(&self) {
        self.resource_history.lock().await.clear();
    }

    /// Get latest resource snapshot
    pub async fn get_latest_snapshot(&self) -> Option<ResourceSnapshot> {
        self.resource_history.lock().await.back().cloned()
    }

    /// Check if resource usage is within acceptable limits
    pub async fn is_resource_usage_healthy(&self) -> bool {
        if let Some(snapshot) = self.get_latest_snapshot().await {
            snapshot.cpu_percent < self.config.cpu_warning_threshold
                && snapshot.file_descriptors < self.config.fd_warning_threshold
                && snapshot.memory_kb < self.config.memory_config.memory_warning_threshold_kb
        } else {
            true // No data available, assume healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_stats_update() {
        let stats = CpuStats::default();
        stats.update(5000, 100); // 50% CPU, 100ms

        assert_eq!(stats.current_cpu_percent(), 50.0);
        assert_eq!(stats.peak_cpu_percent(), 50.0);
        assert_eq!(stats.sample_count(), 1);
        assert_eq!(stats.total_cpu_time_ms(), 100);

        stats.update(3000, 50); // 30% CPU, 50ms
        assert_eq!(stats.current_cpu_percent(), 30.0);
        assert_eq!(stats.peak_cpu_percent(), 50.0); // Peak should remain
        assert_eq!(stats.sample_count(), 2);
        assert_eq!(stats.total_cpu_time_ms(), 150);
    }

    #[test]
    fn test_fd_stats_update() {
        let stats = FileDescriptorStats::default();
        stats.update(100, 5);

        assert_eq!(stats.current_fd_count(), 100);
        assert_eq!(stats.peak_fd_count(), 100);
        assert_eq!(stats.watcher_fd_count(), 5);
        assert_eq!(stats.sample_count(), 1);

        stats.update(80, 4);
        assert_eq!(stats.current_fd_count(), 80);
        assert_eq!(stats.peak_fd_count(), 100); // Peak should remain
        assert_eq!(stats.watcher_fd_count(), 4);
        assert_eq!(stats.sample_count(), 2);
    }

    #[test]
    fn test_resource_monitor_config_default() {
        let config = ResourceMonitorConfig::default();
        assert_eq!(config.sampling_interval_secs, 30);
        assert_eq!(config.cpu_warning_threshold, 70.0);
        assert_eq!(config.cpu_critical_threshold, 90.0);
        assert_eq!(config.fd_warning_threshold, 1000);
        assert_eq!(config.fd_critical_threshold, 2000);
        assert_eq!(config.max_history_size, 100);
        assert!(config.enable_history);
    }

    #[tokio::test]
    async fn test_resource_monitor_creation() {
        let monitor = ResourceMonitor::new();
        assert!(monitor.is_resource_usage_healthy().await);

        let history = monitor.get_resource_history().await;
        assert!(history.is_empty());
    }
}
