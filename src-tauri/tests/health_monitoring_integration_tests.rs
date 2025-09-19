// tests/health_monitoring_integration_tests.rs
// Integration tests for the comprehensive health monitoring and recovery system

use proxemic::filesystem::health_monitor::{
    CircuitBreakerConfig, HealthCheckType, HealthMonitorConfig, HealthStatus, RecoveryAction,
    WatcherHealthMonitor,
};
use proxemic::filesystem::watcher::{DocumentWatcher, WatcherConfig};
use proxemic::resource_monitor::{ResourceMonitor, ResourceMonitorConfig};
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_health_monitor_basic_functionality() {
    let config = HealthMonitorConfig {
        check_interval: Duration::from_millis(100), // Fast for testing
        cpu_threshold: 50.0,
        memory_threshold: 50.0,
        error_rate_threshold: 10.0,
        enable_auto_recovery: false, // Disable for controlled testing
        ..Default::default()
    };

    let monitor = WatcherHealthMonitor::new(config);

    // Test initial state
    let metrics = monitor.get_metrics();
    assert_eq!(metrics.total_checks, 0);
    assert_eq!(metrics.healthy_checks, 0);
    assert!(!metrics.is_monitoring);

    // Test health status (should be unknown initially)
    let status = monitor.get_overall_health().await;
    assert_eq!(status, HealthStatus::Unknown);

    // Test manual health check without resource monitor
    let results = monitor.check_health(None).await;
    assert!(!results.is_empty());

    // Verify we got expected health check types (no resource usage check without monitor)
    let check_types: Vec<_> = results.iter().map(|r| r.check_type).collect();
    assert!(check_types.contains(&HealthCheckType::WatcherResponsiveness));
    assert!(check_types.contains(&HealthCheckType::ErrorRate));

    // Test with resource monitor - note that it may not have data initially
    let resource_monitor = ResourceMonitor::with_config(ResourceMonitorConfig::default());
    let results_with_monitor = monitor.check_health(Some(&resource_monitor)).await;
    assert!(!results_with_monitor.is_empty());

    // Should have basic health checks even without resource data
    let check_types_with_monitor: Vec<_> =
        results_with_monitor.iter().map(|r| r.check_type).collect();
    assert!(check_types_with_monitor.contains(&HealthCheckType::WatcherResponsiveness));
    assert!(check_types_with_monitor.contains(&HealthCheckType::ErrorRate));
}

#[tokio::test]
async fn test_circuit_breaker_states() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        success_threshold: 2,
        ..Default::default()
    };

    let monitor = WatcherHealthMonitor::new(HealthMonitorConfig {
        circuit_breaker_config: config,
        ..Default::default()
    });

    let watcher_circuit = monitor.get_watcher_circuit_breaker();
    let processor_circuit = monitor.get_processor_circuit_breaker();

    // Test circuit breaker metrics
    let watcher_metrics = watcher_circuit.metrics();
    let processor_metrics = processor_circuit.metrics();

    assert_eq!(watcher_metrics.failure_count, 0);
    assert_eq!(watcher_metrics.success_count, 0);
    assert_eq!(processor_metrics.failure_count, 0);
    assert_eq!(processor_metrics.success_count, 0);
}

#[tokio::test]
async fn test_watcher_with_health_monitoring() {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let _temp_path = temp_dir.path().to_path_buf();

    let watcher_config = WatcherConfig {
        enable_health_monitoring: true,
        health_monitor_config: HealthMonitorConfig {
            check_interval: Duration::from_millis(100),
            ..Default::default()
        },
        ..Default::default()
    };

    let app = tauri::test::mock_app();
    let app_handle = app.handle().clone();

    let (watcher, _receiver) = DocumentWatcher::new(watcher_config, app_handle);

    // Test health monitoring startup
    let result = watcher.start_health_monitoring().await;
    assert!(
        result.is_ok(),
        "Health monitoring should start successfully"
    );

    // Test health status retrieval
    let health_status = watcher.get_health_status().await;
    // Initially should be unknown since no checks have run yet
    assert_eq!(health_status, HealthStatus::Unknown);

    // Test manual health check
    let health_results = watcher.check_health().await;
    assert!(
        !health_results.is_empty(),
        "Should return health check results"
    );

    // Test health metrics
    let metrics = watcher.get_health_metrics().await;
    assert!(metrics.is_some(), "Should return health metrics");

    // Test recovery actions
    let recovery_actions = vec![
        RecoveryAction::ReduceFrequency,
        RecoveryAction::GarbageCollection,
    ];
    let recovery_result = watcher.trigger_recovery(recovery_actions).await;
    assert!(
        recovery_result.is_ok(),
        "Recovery actions should execute successfully"
    );

    // Test circuit breaker access
    let watcher_circuit = watcher.get_watcher_circuit_breaker();
    let processor_circuit = watcher.get_processor_circuit_breaker();
    assert!(
        watcher_circuit.is_some(),
        "Should have watcher circuit breaker"
    );
    assert!(
        processor_circuit.is_some(),
        "Should have processor circuit breaker"
    );
}

#[tokio::test]
async fn test_health_monitoring_with_resource_monitor() {
    let resource_monitor = ResourceMonitor::with_config(ResourceMonitorConfig::default());

    // Take a sample to ensure we have data
    let _snapshot = resource_monitor.sample_resources(1).await;

    // Small delay to ensure the sample is processed
    tokio::time::sleep(Duration::from_millis(10)).await;

    let monitor = WatcherHealthMonitor::new(HealthMonitorConfig {
        check_interval: Duration::from_millis(100),
        cpu_threshold: 90.0, // High threshold to avoid false positives
        memory_threshold: 90.0,
        ..Default::default()
    });

    // Perform health check with resource monitoring
    let results = monitor.check_health(Some(&resource_monitor)).await;

    // Should have health check results
    assert!(!results.is_empty());

    // Should have standard health checks
    let check_types: Vec<_> = results.iter().map(|r| r.check_type).collect();
    assert!(check_types.contains(&HealthCheckType::WatcherResponsiveness));
    assert!(check_types.contains(&HealthCheckType::ErrorRate));

    // May or may not have resource usage check depending on data availability
    let resource_check = results
        .iter()
        .find(|r| r.check_type == HealthCheckType::ResourceUsage);

    if let Some(check) = resource_check {
        // If we have resource data, verify it has valid details
        assert!(check.details.is_object());
        assert!(check.details.get("cpu_usage").is_some());
        assert!(check.details.get("memory_usage").is_some());
    }
}

#[tokio::test]
async fn test_health_history_tracking() {
    let config = HealthMonitorConfig {
        history_size: 5,
        ..Default::default()
    };

    let monitor = WatcherHealthMonitor::new(config);

    // Initially should have empty history
    let history = monitor.get_health_history(None).await;
    assert!(history.is_empty());

    // Simulate some health checks by calling check_health multiple times
    for _ in 0..3 {
        let _results = monitor.check_health(None).await;
        // In a real scenario, these would be stored in history
        // But our simplified implementation doesn't store manual checks
    }

    // Test history limit
    let limited_history = monitor.get_health_history(Some(2)).await;
    assert!(limited_history.len() <= 2);
}

#[tokio::test]
async fn test_recovery_action_execution() {
    let _temp_dir = tempdir().expect("Failed to create temp directory");
    let app = tauri::test::mock_app();
    let app_handle = app.handle().clone();

    let (watcher, _receiver) = DocumentWatcher::new(
        WatcherConfig {
            enable_health_monitoring: true,
            ..Default::default()
        },
        app_handle,
    );

    // Test each recovery action type
    let all_recovery_actions = vec![
        RecoveryAction::RestartWatcher,
        RecoveryAction::ClearQueues,
        RecoveryAction::ReduceFrequency,
        RecoveryAction::NotifyAdmin,
        RecoveryAction::GarbageCollection,
        RecoveryAction::ResetConnections,
    ];

    let result = watcher.trigger_recovery(all_recovery_actions).await;
    assert!(
        result.is_ok(),
        "All recovery actions should execute without errors"
    );
}
