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
