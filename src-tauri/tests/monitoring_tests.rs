// src-tauri/tests/monitoring_tests.rs
// Tests for the monitoring and performance tracking features

use fiovana::commands::ValidationMetrics;
use fiovana::filesystem::operations::FileOperationMetrics;
use std::sync::atomic::Ordering;
use std::time::Duration;

#[test]
fn test_file_operations_metrics() {
    let metrics = FileOperationMetrics::default();

    // Test initial state
    assert_eq!(
        metrics
            .total_operations
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        metrics
            .successful_operations
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        metrics
            .failed_operations
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        metrics
            .total_duration_ns
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // Test recording operations
    metrics.record_success(Duration::from_millis(100));
    metrics.record_failure(Duration::from_millis(50));

    assert_eq!(
        metrics
            .total_operations
            .load(std::sync::atomic::Ordering::Relaxed),
        2
    );
    assert_eq!(
        metrics
            .successful_operations
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert_eq!(
        metrics
            .failed_operations
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert!(metrics.total_duration_ns.load(Ordering::Relaxed) > 0);

    // Test average time calculation
    let avg_time = metrics.average_duration_ns();
    assert!(avg_time > 0);

    // Test failure rate calculation
    let failure_rate = metrics.error_rate();
    assert_eq!(failure_rate, 50.0); // 1 out of 2 operations failed = 50%

    // Test reset
    metrics.reset();
    assert_eq!(
        metrics
            .total_operations
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn test_validation_metrics() {
    let metrics = ValidationMetrics::default();

    // Test initial state
    assert_eq!(
        metrics
            .total_validations
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        metrics
            .successful_validations
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        metrics
            .failed_validations
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert_eq!(
        metrics
            .total_validation_time_ns
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // Test recording validations
    metrics.record_success(Duration::from_millis(200));
    metrics.record_success(Duration::from_millis(100));
    metrics.record_failure(Duration::from_millis(50));

    assert_eq!(
        metrics
            .total_validations
            .load(std::sync::atomic::Ordering::Relaxed),
        3
    );
    assert_eq!(
        metrics
            .successful_validations
            .load(std::sync::atomic::Ordering::Relaxed),
        2
    );
    assert_eq!(
        metrics
            .failed_validations
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert!(
        metrics
            .total_validation_time_ns
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0
    );

    // Test average time calculation
    let avg_time = metrics.average_validation_time_ns();
    assert!(avg_time > 0);

    // Test failure rate calculation
    let failure_rate = metrics.failure_rate();
    assert!((failure_rate - 33.33).abs() < 0.1); // 1 out of 3 validations failed ≈ 33.33%

    // Test reset
    metrics.reset();
    assert_eq!(
        metrics
            .total_validations
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn test_memory_monitor_creation() {
    // Test that memory monitor can be created
    let _monitor = fiovana::memory_monitor::MemoryMonitor::new();

    // Test memory usage reading (should be non-zero on most systems)
    let usage = fiovana::memory_monitor::MemoryMonitor::get_current_memory_usage();

    // Only assert if we successfully got a reading
    // (might fail in CI environments or unsupported platforms)
    if let Ok(usage_kb) = usage {
        assert!(usage_kb > 0, "Memory usage should be greater than 0");
    } else {
        println!("Memory monitoring not available on this platform");
    }
}

#[test]
fn test_memory_monitor_config() {
    use fiovana::memory_monitor::MemoryMonitorConfig;

    let config = MemoryMonitorConfig::default();
    assert_eq!(config.sampling_interval_secs, 30);
    assert!(config.memory_warning_threshold_kb > 0);
    assert!(config.memory_critical_threshold_kb > config.memory_warning_threshold_kb);

    // Test custom config
    let custom_config = MemoryMonitorConfig {
        sampling_interval_secs: 60,
        memory_warning_threshold_kb: 256 * 1024,  // 256MB
        memory_critical_threshold_kb: 512 * 1024, // 512MB
    };

    assert_eq!(custom_config.sampling_interval_secs, 60);
    assert_eq!(custom_config.memory_warning_threshold_kb, 256 * 1024);
    assert_eq!(custom_config.memory_critical_threshold_kb, 512 * 1024);
}

#[test]
fn test_security_config_monitoring_fields() {
    use fiovana::filesystem::security::security_config::SecurityConfig;

    // Test production defaults include monitoring thresholds
    let config = SecurityConfig::production_defaults();

    assert!(config.memory_warning_threshold_kb > 0);
    assert!(config.memory_critical_threshold_kb > config.memory_warning_threshold_kb);
    assert!(config.operation_latency_warning_ms > 0);
    assert!(config.operation_latency_critical_ms > config.operation_latency_warning_ms);
    assert!(config.error_rate_warning_percent > 0.0);
    assert!(config.error_rate_critical_percent > config.error_rate_warning_percent);
    assert!(config.monitoring_enabled);

    // Test development defaults include monitoring thresholds
    let dev_config = SecurityConfig::default();
    assert!(dev_config.memory_warning_threshold_kb > 0);
    assert!(dev_config.memory_critical_threshold_kb > dev_config.memory_warning_threshold_kb);
    assert!(dev_config.operation_latency_warning_ms > 0);
    assert!(dev_config.operation_latency_critical_ms > dev_config.operation_latency_warning_ms);
    assert!(dev_config.error_rate_warning_percent >= 0.0);
    assert!(dev_config.error_rate_critical_percent > dev_config.error_rate_warning_percent);
    // Development defaults should have monitoring disabled by default
    assert!(!dev_config.monitoring_enabled);
}

#[test]
fn test_memory_stats() {
    use fiovana::memory_monitor::MemoryStats;

    let stats = MemoryStats::default();

    // Test initial state
    assert_eq!(stats.current_memory_kb(), 0);
    assert_eq!(stats.peak_memory_kb(), 0);
    assert_eq!(stats.sample_count(), 0);

    // Test updating stats
    stats.update(1024);
    assert_eq!(stats.current_memory_kb(), 1024);
    assert_eq!(stats.peak_memory_kb(), 1024);
    assert_eq!(stats.sample_count(), 1);

    // Test peak tracking
    stats.update(512);
    assert_eq!(stats.current_memory_kb(), 512);
    assert_eq!(stats.peak_memory_kb(), 1024); // Peak should remain
    assert_eq!(stats.sample_count(), 2);

    // Test new peak
    stats.update(2048);
    assert_eq!(stats.current_memory_kb(), 2048);
    assert_eq!(stats.peak_memory_kb(), 2048); // New peak
    assert_eq!(stats.sample_count(), 3);
}

#[tokio::test]
async fn test_memory_monitor_sampling() {
    use fiovana::memory_monitor::{MemoryMonitor, MemoryMonitorConfig};
    use std::time::Duration;

    let config = MemoryMonitorConfig {
        sampling_interval_secs: 1,                 // 1 second for testing
        memory_warning_threshold_kb: 1024 * 1024,  // 1GB - high threshold to avoid warnings
        memory_critical_threshold_kb: 2048 * 1024, // 2GB - very high threshold
    };

    let monitor = MemoryMonitor::with_config(config);

    // First sample should work
    let _result1 = monitor.sample_memory_usage().await;
    // Don't assert success since memory monitoring might not be available on all platforms

    // Second sample immediately should be skipped due to interval
    let result2 = monitor.sample_memory_usage().await;
    assert!(result2.is_ok()); // Should succeed but not actually sample

    // Wait and try again (in a real scenario)
    tokio::time::sleep(Duration::from_millis(100)).await;
    let result3 = monitor.sample_memory_usage().await;
    assert!(result3.is_ok());
}

#[test]
fn test_performance_metrics_integration() {
    use fiovana::commands::get_validation_metrics;
    use fiovana::filesystem::operations::get_file_operation_metrics;

    // Test that global metrics instances are available
    let file_metrics = get_file_operation_metrics();
    let validation_metrics = get_validation_metrics();

    // Reset to ensure clean state
    file_metrics.reset();
    validation_metrics.reset();

    // Test initial states
    assert_eq!(file_metrics.total_operations.load(Ordering::Relaxed), 0);
    assert_eq!(
        validation_metrics.total_validations.load(Ordering::Relaxed),
        0
    );

    // Test that we can record metrics
    file_metrics.record_success(Duration::from_millis(100));
    validation_metrics.record_success(Duration::from_millis(50));

    assert_eq!(file_metrics.total_operations.load(Ordering::Relaxed), 1);
    assert_eq!(
        validation_metrics.total_validations.load(Ordering::Relaxed),
        1
    );
}
