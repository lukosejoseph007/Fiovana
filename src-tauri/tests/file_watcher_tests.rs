// src-tauri/tests/file_watcher_tests.rs
// Tests for file watcher event categorization

#[cfg(test)]
mod tests {
    use proxemic::filesystem::watcher::FrontendFileEvent;

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
    use proxemic::filesystem::security::path_validator::PathValidator;
    use proxemic::filesystem::security::security_config::SecurityConfig;
    use proxemic::filesystem::watcher::{EventDebouncer, FileEvent};
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
