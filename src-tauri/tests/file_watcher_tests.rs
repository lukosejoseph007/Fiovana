// src-tauri/tests/file_watcher_tests.rs
// Tests for file watcher event categorization

#[cfg(test)]
mod tests {
    use fiovana::filesystem::watcher::FrontendFileEvent;

    #[test]
    fn test_event_categorization() {
        // Test created event
        let created_event = FrontendFileEvent {
            type_: "file-created".to_string(),
            path: "/test/file.txt".to_string(),
            old_path: None,
            timestamp: 1234567890,
            size: Some(1024),
            is_directory: false,
        };
        assert_eq!(created_event.type_, "file-created");

        // Test modified event
        let modified_event = FrontendFileEvent {
            type_: "file-modified".to_string(),
            path: "/test/file.txt".to_string(),
            old_path: None,
            timestamp: 1234567890,
            size: Some(2048),
            is_directory: false,
        };
        assert_eq!(modified_event.type_, "file-modified");

        // Test deleted event
        let deleted_event = FrontendFileEvent {
            type_: "file-deleted".to_string(),
            path: "/test/file.txt".to_string(),
            old_path: None,
            timestamp: 1234567890,
            size: None,
            is_directory: false,
        };
        assert_eq!(deleted_event.type_, "file-deleted");

        // Test moved event (rename with different paths)
        let moved_event = FrontendFileEvent {
            type_: "file-moved".to_string(),
            path: "/test/new_file.txt".to_string(),
            old_path: Some("/test/old_file.txt".to_string()),
            timestamp: 1234567890,
            size: Some(1024),
            is_directory: false,
        };
        assert_eq!(moved_event.type_, "file-moved");

        // Test renamed event (rename with same directory)
        let renamed_event = FrontendFileEvent {
            type_: "file-renamed".to_string(),
            path: "/test/renamed_file.txt".to_string(),
            old_path: Some("/test/original_file.txt".to_string()),
            timestamp: 1234567890,
            size: Some(1024),
            is_directory: false,
        };
        assert_eq!(renamed_event.type_, "file-renamed");
    }

    #[test]
    fn test_event_serialization() {
        let event = FrontendFileEvent {
            type_: "file-created".to_string(),
            path: "/test/file.txt".to_string(),
            old_path: None,
            timestamp: 1234567890,
            size: Some(1024),
            is_directory: false,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        assert!(serialized.contains("file-created"));
        assert!(serialized.contains("/test/file.txt"));
        assert!(serialized.contains("1024"));
    }

    #[test]
    fn test_directory_events() {
        // Test directory created event
        let dir_event = FrontendFileEvent {
            type_: "file-created".to_string(),
            path: "/test/directory".to_string(),
            old_path: None,
            timestamp: 1234567890,
            size: None,
            is_directory: true,
        };
        assert_eq!(dir_event.type_, "file-created");
        assert!(dir_event.is_directory);

        // Test directory moved event
        let dir_moved_event = FrontendFileEvent {
            type_: "file-moved".to_string(),
            path: "/new/location/directory".to_string(),
            old_path: Some("/test/directory".to_string()),
            timestamp: 1234567890,
            size: None,
            is_directory: true,
        };
        assert_eq!(dir_moved_event.type_, "file-moved");
        assert!(dir_moved_event.is_directory);
    }
}

// Tests for batch processing functionality
#[cfg(test)]
mod batch_tests {
    use notify::event::ModifyKind;
    use notify::{Event, EventKind};
    use fiovana::filesystem::security::path_validator::PathValidator;
    use fiovana::filesystem::security::security_config::SecurityConfig;
    use fiovana::filesystem::watcher::{EventDebouncer, FileEvent};
    use std::path::PathBuf;
    use std::time::Duration;

    #[tokio::test]
    async fn test_batch_processing_size_limit() {
        let path_validator = PathValidator::new(SecurityConfig::default(), vec![]);
        let mut debouncer = EventDebouncer::new(Duration::from_millis(500));

        // Create multiple events to trigger batch size limit
        for i in 0..60 {
            let event = Event {
                paths: vec![PathBuf::from(format!("/test/file{}.txt", i))],
                kind: EventKind::Create(notify::event::CreateKind::Any),
                attrs: Default::default(),
            };

            let result = debouncer.process_event(event, &path_validator).await;

            // Should return a batch when we hit the max batch size (50)
            if i == 49 {
                assert!(result.is_some());
                let batch = result.unwrap();
                assert_eq!(batch.len(), 50);
            }
        }
    }

    #[tokio::test]
    async fn test_batch_processing_time_limit() {
        let path_validator = PathValidator::new(SecurityConfig::default(), vec![]);
        let mut debouncer = EventDebouncer::new(Duration::from_millis(500));

        // Create a single event
        let event = Event {
            paths: vec![PathBuf::from("/test/file.txt")],
            kind: EventKind::Create(notify::event::CreateKind::Any),
            attrs: Default::default(),
        };

        let result = debouncer.process_event(event, &path_validator).await;
        assert!(result.is_none()); // Should not batch immediately

        // Wait longer than batch duration (100ms)
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Create another event to trigger time-based batching
        let event2 = Event {
            paths: vec![PathBuf::from("/test/file2.txt")],
            kind: EventKind::Create(notify::event::CreateKind::Any),
            attrs: Default::default(),
        };

        let result = debouncer.process_event(event2, &path_validator).await;
        assert!(result.is_some()); // Should batch due to time expiration
        let batch = result.unwrap();
        assert_eq!(batch.len(), 2); // Both events should be batched
    }

    #[tokio::test]
    async fn test_flush_method() {
        let path_validator = PathValidator::new(SecurityConfig::default(), vec![]);
        let mut debouncer = EventDebouncer::new(Duration::from_millis(500));

        // Add some events to the batch
        for i in 0..10 {
            let event = Event {
                paths: vec![PathBuf::from(format!("/test/file{}.txt", i))],
                kind: EventKind::Create(notify::event::CreateKind::Any),
                attrs: Default::default(),
            };

            let result = debouncer.process_event(event, &path_validator).await;
            assert!(result.is_none()); // Should not batch immediately
        }

        // Flush should return all batched events
        let flushed_events = debouncer.flush();
        assert_eq!(flushed_events.len(), 10);
    }

    #[tokio::test]
    async fn test_mixed_event_types_in_batch() {
        let path_validator = PathValidator::new(SecurityConfig::default(), vec![]);
        let mut debouncer = EventDebouncer::new(Duration::from_millis(500));

        // Create different types of events
        let create_event = Event {
            paths: vec![PathBuf::from("/test/file1.txt")],
            kind: EventKind::Create(notify::event::CreateKind::Any),
            attrs: Default::default(),
        };

        let modify_event = Event {
            paths: vec![PathBuf::from("/test/file2.txt")],
            kind: EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Any)),
            attrs: Default::default(),
        };

        let delete_event = Event {
            paths: vec![PathBuf::from("/test/file3.txt")],
            kind: EventKind::Remove(notify::event::RemoveKind::Any),
            attrs: Default::default(),
        };

        // Process all events
        debouncer.process_event(create_event, &path_validator).await;
        debouncer.process_event(modify_event, &path_validator).await;
        debouncer.process_event(delete_event, &path_validator).await;

        // Wait for batch timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Add one more event to trigger batching
        let trigger_event = Event {
            paths: vec![PathBuf::from("/test/file4.txt")],
            kind: EventKind::Create(notify::event::CreateKind::Any),
            attrs: Default::default(),
        };

        let result = debouncer
            .process_event(trigger_event, &path_validator)
            .await;
        assert!(result.is_some());

        let batch = result.unwrap();
        assert_eq!(batch.len(), 4);

        // Verify event types are preserved
        let event_types: Vec<&str> = batch
            .iter()
            .map(|e| match e {
                FileEvent::Created(_) => "created",
                FileEvent::Modified(_) => "modified",
                FileEvent::Deleted(_) => "deleted",
                _ => "other",
            })
            .collect();

        assert!(event_types.contains(&"created"));
        assert!(event_types.contains(&"modified"));
        assert!(event_types.contains(&"deleted"));
    }
}

// Tests for conflict detection functionality
#[cfg(test)]
mod conflict_tests {
    use chrono::Utc;
    use fiovana::filesystem::watcher::{
        ConflictDetector, ConflictResult, ConflictType, FileSnapshot,
    };
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn test_file_snapshot_creation() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create test file
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;

        // Create snapshot
        let snapshot = FileSnapshot::create(&file_path)?;

        assert_eq!(snapshot.path, file_path);
        assert_eq!(snapshot.size, 13);
        assert!(!snapshot.hash.is_empty());

        Ok(())
    }

    #[test]
    fn test_file_snapshot_hash_consistency() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create test file with specific content
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;

        // Create two snapshots - should have same hash
        let snapshot1 = FileSnapshot::create(&file_path)?;
        let snapshot2 = FileSnapshot::create(&file_path)?;

        assert_eq!(snapshot1.hash, snapshot2.hash);

        Ok(())
    }

    #[test]
    fn test_file_snapshot_hash_different_content() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path1 = temp_dir.path().join("test1.txt");
        let file_path2 = temp_dir.path().join("test2.txt");

        // Create two files with different content
        let mut file1 = File::create(&file_path1)?;
        file1.write_all(b"Hello, world!")?;

        let mut file2 = File::create(&file_path2)?;
        file2.write_all(b"Hello, universe!")?;

        // Create snapshots
        let snapshot1 = FileSnapshot::create(&file_path1)?;
        let snapshot2 = FileSnapshot::create(&file_path2)?;

        assert_ne!(snapshot1.hash, snapshot2.hash);

        Ok(())
    }

    #[test]
    fn test_snapshot_comparison_no_conflict() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create test file
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;

        // Create two identical snapshots
        let snapshot1 = FileSnapshot::create(&file_path)?;
        let snapshot2 = FileSnapshot::create(&file_path)?;

        // Should not detect conflict
        let conflict = snapshot1.compare(&snapshot2);
        assert!(conflict.is_none());

        Ok(())
    }

    #[test]
    fn test_snapshot_comparison_content_conflict() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create first version
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;
        let snapshot1 = FileSnapshot::create(&file_path)?;

        // Modify file content
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, universe!")?;
        let snapshot2 = FileSnapshot::create(&file_path)?;

        // Should detect content conflict
        let conflict = snapshot1.compare(&snapshot2).unwrap();

        assert_eq!(conflict.conflict_type, ConflictType::ContentConflict);
        assert_eq!(conflict.file_path, file_path);
        assert_eq!(conflict.severity, "high");
        assert!(conflict.resolution_required);

        Ok(())
    }

    #[tokio::test]
    async fn test_conflict_detector_snapshot() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create test file
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;

        let mut detector = ConflictDetector::new(Duration::from_secs(60));
        let watched_paths = vec![file_path.clone()];

        // Take snapshot
        let results = detector.take_snapshot(&watched_paths).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_conflict_detector_check_no_conflicts() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create test file
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;

        let mut detector = ConflictDetector::new(Duration::from_secs(60));
        let watched_paths = vec![file_path.clone()];

        // Take snapshot
        detector.take_snapshot(&watched_paths).await;

        // Check for conflicts - should find none
        let conflicts = detector.check_conflicts(&watched_paths).await;
        assert!(conflicts.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_conflict_detector_external_modification() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create initial version
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;

        let mut detector = ConflictDetector::new(Duration::from_secs(60));
        let watched_paths = vec![file_path.clone()];

        // Take snapshot
        detector.take_snapshot(&watched_paths).await;

        // Modify file externally (simulate external change)
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, universe!")?;

        // Check for conflicts - should detect external modification
        let conflicts = detector.check_conflicts(&watched_paths).await;
        assert_eq!(conflicts.len(), 1);

        let conflict = &conflicts[0];
        assert_eq!(conflict.conflict_type, ConflictType::ContentConflict);
        assert_eq!(conflict.file_path, file_path);
        assert_eq!(conflict.severity, "high");

        Ok(())
    }

    #[tokio::test]
    async fn test_conflict_detector_external_deletion() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create initial file
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;

        let mut detector = ConflictDetector::new(Duration::from_secs(60));
        let watched_paths = vec![file_path.clone()];

        // Take snapshot
        detector.take_snapshot(&watched_paths).await;

        // Delete file externally
        fs::remove_file(&file_path)?;

        // Check for conflicts - should detect external deletion
        let conflicts = detector.check_conflicts(&watched_paths).await;
        assert_eq!(conflicts.len(), 1);

        let conflict = &conflicts[0];
        assert_eq!(conflict.conflict_type, ConflictType::ExternalDeletion);
        assert_eq!(conflict.file_path, file_path);
        assert_eq!(conflict.severity, "high");

        Ok(())
    }

    #[tokio::test]
    async fn test_conflict_detector_should_check() {
        let detector = ConflictDetector::new(Duration::from_secs(1));

        // Should not check immediately after creation
        assert!(!detector.should_check());

        // Wait a bit and check again
        tokio::time::sleep(Duration::from_millis(1100)).await;
        assert!(detector.should_check());
    }

    #[tokio::test]
    async fn test_conflict_detector_clear_snapshot() -> std::io::Result<()> {
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create test file
        let mut file = File::create(&file_path)?;
        file.write_all(b"Hello, world!")?;

        let mut detector = ConflictDetector::new(Duration::from_secs(60));
        let watched_paths = vec![file_path.clone()];

        // Take snapshot
        detector.take_snapshot(&watched_paths).await;

        // Clear snapshot
        detector.clear_snapshot(&file_path);

        // Check for conflicts - should find none since snapshot was cleared
        let conflicts = detector.check_conflicts(&watched_paths).await;
        assert!(conflicts.is_empty());

        Ok(())
    }

    #[test]
    fn test_conflict_type_serialization() {
        use serde_json;

        let conflict_result = ConflictResult {
            conflict_type: ConflictType::ContentConflict,
            file_path: PathBuf::from("/test/file.txt"),
            external_timestamp: Some(Utc::now()),
            application_timestamp: Some(Utc::now()),
            external_hash: Some("abc123".to_string()),
            application_hash: Some("def456".to_string()),
            severity: "high".to_string(),
            resolution_required: true,
        };

        let serialized = serde_json::to_string(&conflict_result).unwrap();
        assert!(serialized.contains("ContentConflict"));
        assert!(serialized.contains("high"));
        assert!(serialized.contains("resolution_required"));
    }
}
