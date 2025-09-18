// src-tauri/tests/conflict_detection_tests.rs
// Tests for conflict detection functionality

use chrono::Utc;
use proxemic::filesystem::event_persistence::PersistenceConfig;
use proxemic::filesystem::security::security_config::SecurityConfig;
use proxemic::filesystem::watcher::{ConflictDetector, ConflictType, FileSnapshot, WatcherConfig};
use std::fs::{self, File};
use std::io::Write;
use std::time::Duration;
use tempfile::tempdir;

#[tokio::test]
async fn test_file_snapshot_creation() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create test file
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, world!").unwrap();
    drop(file);

    // Create snapshot
    let snapshot = FileSnapshot::create(&test_file).unwrap();

    assert_eq!(snapshot.path, test_file);
    assert!(snapshot.size > 0);
    assert!(!snapshot.hash.is_empty());
    assert!(snapshot.modified_time <= Utc::now());
    assert!(snapshot.created_time <= Utc::now());
}

#[tokio::test]
async fn test_file_snapshot_hash_calculation() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create test file with specific content
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, world!").unwrap();
    drop(file);

    let snapshot1 = FileSnapshot::create(&test_file).unwrap();

    // Modify file content
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, modified world!").unwrap();
    drop(file);

    let snapshot2 = FileSnapshot::create(&test_file).unwrap();

    // Hashes should be different
    assert_ne!(snapshot1.hash, snapshot2.hash);
}

#[tokio::test]
async fn test_snapshot_comparison_no_conflict() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create test file
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, world!").unwrap();
    drop(file);

    let snapshot1 = FileSnapshot::create(&test_file).unwrap();
    let snapshot2 = FileSnapshot::create(&test_file).unwrap();

    // Same file, same content - no conflict
    assert!(snapshot1.compare(&snapshot2).is_none());
}

#[tokio::test]
async fn test_snapshot_comparison_content_conflict() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create initial file
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, world!").unwrap();
    drop(file);

    let snapshot1 = FileSnapshot::create(&test_file).unwrap();

    // Modify file content externally
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, modified world!").unwrap();
    drop(file);

    let snapshot2 = FileSnapshot::create(&test_file).unwrap();

    // Should detect content conflict
    let conflict = snapshot1.compare(&snapshot2).unwrap();

    assert_eq!(conflict.conflict_type, ConflictType::ContentConflict);
    assert_eq!(conflict.file_path, test_file);
    assert!(conflict.external_timestamp.is_some());
    assert!(conflict.application_timestamp.is_some());
    assert!(conflict.external_hash.is_some());
    assert!(conflict.application_hash.is_some());
    assert_eq!(conflict.severity, "high");
    assert!(conflict.resolution_required);
}

#[tokio::test]
async fn test_snapshot_comparison_timestamp_conflict() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create initial file
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, world!").unwrap();
    drop(file);

    let snapshot1 = FileSnapshot::create(&test_file).unwrap();

    // Touch file to update timestamp without changing content
    filetime::set_file_mtime(&test_file, filetime::FileTime::now()).unwrap();

    let snapshot2 = FileSnapshot::create(&test_file).unwrap();

    // Should detect timestamp conflict
    let conflict = snapshot1.compare(&snapshot2).unwrap();

    assert_eq!(conflict.conflict_type, ConflictType::TimestampConflict);
    assert_eq!(conflict.file_path, test_file);
    assert!(conflict.external_timestamp.is_some());
    assert!(conflict.application_timestamp.is_some());
    assert_eq!(conflict.external_hash, conflict.application_hash);
    assert_eq!(conflict.severity, "medium");
    assert!(conflict.resolution_required);
}

#[tokio::test]
async fn test_conflict_detector_take_snapshot() {
    let temp_dir = tempdir().unwrap();
    let test_file1 = temp_dir.path().join("test1.txt");
    let test_file2 = temp_dir.path().join("test2.txt");

    // Create test files
    File::create(&test_file1).unwrap();
    File::create(&test_file2).unwrap();

    let mut detector = ConflictDetector::new(Duration::from_secs(60));
    let watched_paths = vec![test_file1.clone(), test_file2.clone()];

    let results = detector.take_snapshot(&watched_paths).await;

    assert_eq!(results.len(), 2);
    assert!(results.iter().all(|r| r.is_ok()));

    // Check that snapshots are stored
    assert!(detector.file_snapshots.contains_key(&test_file1));
    assert!(detector.file_snapshots.contains_key(&test_file2));
}

#[tokio::test]
async fn test_conflict_detector_check_conflicts() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create initial file
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, world!").unwrap();
    drop(file);

    let mut detector = ConflictDetector::new(Duration::from_secs(60));
    let watched_paths = vec![test_file.clone()];

    // Take initial snapshot
    detector.take_snapshot(&watched_paths).await;

    // Modify file externally
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, modified world!").unwrap();
    drop(file);

    // Check for conflicts
    let conflicts = detector.check_conflicts(&watched_paths).await;

    assert_eq!(conflicts.len(), 1);
    let conflict = &conflicts[0];

    assert_eq!(conflict.conflict_type, ConflictType::ContentConflict);
    assert_eq!(conflict.file_path, test_file);
}

#[tokio::test]
async fn test_conflict_detector_external_deletion() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create initial file
    File::create(&test_file).unwrap();

    let mut detector = ConflictDetector::new(Duration::from_secs(60));
    let watched_paths = vec![test_file.clone()];

    // Take initial snapshot
    detector.take_snapshot(&watched_paths).await;

    // Delete file externally
    fs::remove_file(&test_file).unwrap();

    // Check for conflicts
    let conflicts = detector.check_conflicts(&watched_paths).await;

    // Should detect external deletion even though file no longer exists
    assert_eq!(conflicts.len(), 1);
    let conflict = &conflicts[0];

    assert_eq!(conflict.conflict_type, ConflictType::ExternalDeletion);
    assert_eq!(conflict.file_path, test_file);
    assert!(conflict.external_hash.is_none());
    assert!(conflict.application_hash.is_some());
}

#[tokio::test]
async fn test_conflict_detector_should_check() {
    let detector = ConflictDetector::new(Duration::from_secs(1));

    // Immediately after creation, should not need to check
    assert!(!detector.should_check());

    // Wait a bit and check again
    tokio::time::sleep(Duration::from_millis(1100)).await;
    assert!(detector.should_check());
}

#[tokio::test]
async fn test_watcher_config_creation() {
    let config = WatcherConfig {
        debounce_duration: Duration::from_millis(500),
        security_config: SecurityConfig::default(),
        enable_persistence: false,
        persistence_config: PersistenceConfig::default(),
        workspace_id: None,
        enable_resource_monitoring: false,
        resource_monitor_config: proxemic::resource_monitor::ResourceMonitorConfig::default(),
        enable_optimized_processing: false,
        event_processor_config:
            proxemic::filesystem::event_processor::EventProcessorConfig::default(),
        enable_health_monitoring: false,
        health_monitor_config: proxemic::filesystem::health_monitor::HealthMonitorConfig::default(),
    };

    assert_eq!(config.debounce_duration, Duration::from_millis(500));
    // SecurityConfig doesn't implement PartialEq, so we can't compare directly
    // Just verify the config was created successfully
    assert!(config.debounce_duration == Duration::from_millis(500));
}

#[tokio::test]
async fn test_conflict_detector_clear_snapshots() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    File::create(&test_file).unwrap();

    let mut detector = ConflictDetector::new(Duration::from_secs(60));
    let watched_paths = vec![test_file.clone()];

    detector.take_snapshot(&watched_paths).await;
    assert!(detector.file_snapshots.contains_key(&test_file));

    detector.clear_snapshot(&test_file);
    assert!(!detector.file_snapshots.contains_key(&test_file));

    detector.take_snapshot(&watched_paths).await;
    assert!(detector.file_snapshots.contains_key(&test_file));

    detector.clear_all_snapshots();
    assert!(detector.file_snapshots.is_empty());
}

#[tokio::test]
async fn test_conflict_detector_ignore_directories() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    let test_dir = temp_dir.path().join("subdir");

    File::create(&test_file).unwrap();
    fs::create_dir(&test_dir).unwrap();

    let mut detector = ConflictDetector::new(Duration::from_secs(60));
    let watched_paths = vec![test_file.clone(), test_dir.clone()];

    let results = detector.take_snapshot(&watched_paths).await;

    // Should only snapshot the file, not the directory
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok());
    assert!(detector.file_snapshots.contains_key(&test_file));
    assert!(!detector.file_snapshots.contains_key(&test_dir));
}

#[tokio::test]
async fn test_conflict_detector_nonexistent_file() {
    let temp_dir = tempdir().unwrap();
    let nonexistent_file = temp_dir.path().join("nonexistent.txt");

    let mut detector = ConflictDetector::new(Duration::from_secs(60));
    let watched_paths = vec![nonexistent_file.clone()];

    let results = detector.take_snapshot(&watched_paths).await;

    // Should skip nonexistent files (they're not files, so they get filtered out)
    assert_eq!(results.len(), 0);
    assert!(!detector.file_snapshots.contains_key(&nonexistent_file));
}

#[tokio::test]
async fn test_conflict_severity_levels() {
    let temp_dir = tempdir().unwrap();
    let test_file = temp_dir.path().join("test.txt");

    // Create initial file
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, world!").unwrap();
    drop(file);

    let snapshot1 = FileSnapshot::create(&test_file).unwrap();

    // Test content conflict (high severity)
    let mut file = File::create(&test_file).unwrap();
    writeln!(file, "Hello, modified world!").unwrap();
    drop(file);

    let snapshot2 = FileSnapshot::create(&test_file).unwrap();
    let content_conflict = snapshot1.compare(&snapshot2).unwrap();
    assert_eq!(content_conflict.severity, "high");

    // Test timestamp conflict (medium severity)
    let snapshot3 = FileSnapshot::create(&test_file).unwrap();
    filetime::set_file_mtime(&test_file, filetime::FileTime::now()).unwrap();
    let snapshot4 = FileSnapshot::create(&test_file).unwrap();
    let timestamp_conflict = snapshot3.compare(&snapshot4).unwrap();
    assert_eq!(timestamp_conflict.severity, "medium");
}
