// src-tauri/src/memory_monitor.rs
// Cross-platform memory usage monitoring

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{error, info, warn};

/// Memory usage statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    pub current_memory_kb: AtomicU64,
    pub peak_memory_kb: AtomicU64,
    pub sample_count: AtomicU64,
}

impl MemoryStats {
    /// Update memory statistics with current usage
    pub fn update(&self, memory_kb: u64) {
        self.current_memory_kb.store(memory_kb, Ordering::Relaxed);

        let current_peak = self.peak_memory_kb.load(Ordering::Relaxed);
        if memory_kb > current_peak {
            self.peak_memory_kb.store(memory_kb, Ordering::Relaxed);
        }

        self.sample_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current memory usage in KB
    pub fn current_memory_kb(&self) -> u64 {
        self.current_memory_kb.load(Ordering::Relaxed)
    }

    /// Get peak memory usage in KB
    pub fn peak_memory_kb(&self) -> u64 {
        self.peak_memory_kb.load(Ordering::Relaxed)
    }

    /// Get total number of samples taken
    pub fn sample_count(&self) -> u64 {
        self.sample_count.load(Ordering::Relaxed)
    }
}

// Global memory statistics
static MEMORY_STATS: once_cell::sync::Lazy<MemoryStats> =
    once_cell::sync::Lazy::new(MemoryStats::default);

/// Memory monitor configuration
#[derive(Debug, Clone)]
pub struct MemoryMonitorConfig {
    pub sampling_interval_secs: u64,
    pub memory_warning_threshold_kb: u64,
    pub memory_critical_threshold_kb: u64,
}

impl Default for MemoryMonitorConfig {
    fn default() -> Self {
        Self {
            sampling_interval_secs: 30,               // Sample every 30 seconds
            memory_warning_threshold_kb: 512 * 1024,  // 512MB warning threshold
            memory_critical_threshold_kb: 768 * 1024, // 768MB critical threshold
        }
    }
}

/// Memory monitor state
pub struct MemoryMonitor {
    config: MemoryMonitorConfig,
    last_sample_time: Mutex<Instant>,
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryMonitor {
    /// Create a new memory monitor with default configuration
    pub fn new() -> Self {
        Self {
            config: MemoryMonitorConfig::default(),
            last_sample_time: Mutex::new(Instant::now()),
        }
    }

    /// Create a new memory monitor with custom configuration
    pub fn with_config(config: MemoryMonitorConfig) -> Self {
        Self {
            config,
            last_sample_time: Mutex::new(Instant::now()),
        }
    }

    /// Get current memory usage in KB
    pub fn get_current_memory_usage() -> Result<u64, String> {
        #[cfg(target_os = "linux")]
        {
            get_memory_usage_linux()
        }
        #[cfg(target_os = "windows")]
        {
            get_memory_usage_windows()
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Err("Unsupported operating system".to_string())
        }
    }

    /// Sample memory usage if enough time has passed since last sample
    pub async fn sample_memory_usage(&self) -> Result<(), String> {
        let mut last_sample_time = self.last_sample_time.lock().await;
        let now = Instant::now();

        if now.duration_since(*last_sample_time)
            < Duration::from_secs(self.config.sampling_interval_secs)
        {
            return Ok(()); // Not time to sample yet
        }

        match Self::get_current_memory_usage() {
            Ok(memory_kb) => {
                MEMORY_STATS.update(memory_kb);

                // Check thresholds and log warnings if needed
                if memory_kb > self.config.memory_critical_threshold_kb {
                    error!(
                        "CRITICAL: Memory usage exceeded critical threshold: {}KB > {}KB",
                        memory_kb, self.config.memory_critical_threshold_kb
                    );
                } else if memory_kb > self.config.memory_warning_threshold_kb {
                    warn!(
                        "WARNING: Memory usage exceeded warning threshold: {}KB > {}KB",
                        memory_kb, self.config.memory_warning_threshold_kb
                    );
                } else {
                    info!("Memory usage: {}KB", memory_kb);
                }

                *last_sample_time = now;
                Ok(())
            }
            Err(e) => {
                error!("Failed to sample memory usage: {}", e);
                Err(e)
            }
        }
    }

    /// Get a reference to the global memory statistics
    pub fn get_memory_stats() -> &'static MemoryStats {
        &MEMORY_STATS
    }
}

/// Get memory usage on Linux systems by reading /proc/self/status
#[cfg(target_os = "linux")]
fn get_memory_usage_linux() -> Result<u64, String> {
    use std::fs;

    let status_content = fs::read_to_string("/proc/self/status")
        .map_err(|e| format!("Failed to read /proc/self/status: {}", e))?;

    for line in status_content.lines() {
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let value: u64 = parts[1]
                    .parse()
                    .map_err(|e| format!("Failed to parse VmRSS value: {}", e))?;
                let unit = parts.get(2).unwrap_or(&"kB");

                // Convert to KB if needed
                return match unit.to_lowercase().as_str() {
                    "kb" | "kib" => Ok(value),
                    "mb" | "mib" => Ok(value * 1024),
                    "gb" | "gib" => Ok(value * 1024 * 1024),
                    _ => Ok(value), // Assume KB by default
                };
            }
        }
    }

    Err("VmRSS not found in /proc/self/status".to_string())
}

/// Get memory usage on Windows systems using GetProcessMemoryInfo API
#[cfg(target_os = "windows")]
fn get_memory_usage_windows() -> Result<u64, String> {
    use std::mem;
    use winapi::um::processthreadsapi;
    use winapi::um::psapi;

    unsafe {
        let process = processthreadsapi::GetCurrentProcess();
        let mut memory_info: psapi::PROCESS_MEMORY_COUNTERS = mem::zeroed();
        let memory_info_size = mem::size_of::<psapi::PROCESS_MEMORY_COUNTERS>() as u32;

        if psapi::GetProcessMemoryInfo(process, &mut memory_info, memory_info_size) != 0 {
            // WorkingSetSize is in bytes, convert to KB
            Ok((memory_info.WorkingSetSize / 1024) as u64)
        } else {
            Err("Failed to get process memory info".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_stats_update() {
        let stats = MemoryStats::default();
        stats.update(1024);

        assert_eq!(stats.current_memory_kb(), 1024);
        assert_eq!(stats.peak_memory_kb(), 1024);
        assert_eq!(stats.sample_count(), 1);

        stats.update(512);
        assert_eq!(stats.current_memory_kb(), 512);
        assert_eq!(stats.peak_memory_kb(), 1024); // Peak should remain
        assert_eq!(stats.sample_count(), 2);

        stats.update(2048);
        assert_eq!(stats.current_memory_kb(), 2048);
        assert_eq!(stats.peak_memory_kb(), 2048); // Peak should update
        assert_eq!(stats.sample_count(), 3);
    }

    #[test]
    fn test_memory_monitor_config_default() {
        let config = MemoryMonitorConfig::default();
        assert_eq!(config.sampling_interval_secs, 30);
        assert_eq!(config.memory_warning_threshold_kb, 512 * 1024);
        assert_eq!(config.memory_critical_threshold_kb, 768 * 1024);
    }
}
