// User presence tracking for collaborative editing
// Tracks cursor positions, active users, and last seen timestamps

use serde::{Deserialize, Serialize};

/// Cursor position in a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
    pub selection_start: Option<(usize, usize)>,
    pub selection_end: Option<(usize, usize)>,
}

/// User presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPresence {
    pub user_id: String,
    pub user_name: String,
    pub color: String, // HSL color for user identification
    pub cursor_position: Option<CursorPosition>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
}

impl UserPresence {
    /// Create a new user presence
    #[allow(dead_code)]
    pub fn new(user_id: String, user_name: String, color: String) -> Self {
        Self {
            user_id,
            user_name,
            color,
            cursor_position: None,
            last_seen: chrono::Utc::now(),
        }
    }

    /// Update cursor position
    #[allow(dead_code)]
    pub fn update_cursor(&mut self, position: CursorPosition) {
        self.cursor_position = Some(position);
        self.last_seen = chrono::Utc::now();
    }

    /// Clear cursor position
    #[allow(dead_code)]
    pub fn clear_cursor(&mut self) {
        self.cursor_position = None;
        self.last_seen = chrono::Utc::now();
    }

    /// Check if user is active (seen within last N seconds)
    #[allow(dead_code)]
    pub fn is_active(&self, timeout_seconds: i64) -> bool {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(self.last_seen);
        duration.num_seconds() < timeout_seconds
    }

    /// Update last seen timestamp
    #[allow(dead_code)]
    pub fn touch(&mut self) {
        self.last_seen = chrono::Utc::now();
    }
}

/// Manages presence information for all users
#[allow(dead_code)]
pub struct PresenceManager {
    active_timeout: i64, // Seconds before user is considered inactive
}

#[allow(dead_code)]
impl PresenceManager {
    /// Create a new presence manager
    pub fn new(active_timeout: i64) -> Self {
        Self { active_timeout }
    }

    /// Filter active users from a list
    pub fn filter_active(&self, users: &[UserPresence]) -> Vec<UserPresence> {
        users
            .iter()
            .filter(|user| user.is_active(self.active_timeout))
            .cloned()
            .collect()
    }

    /// Count active users
    pub fn count_active(&self, users: &[UserPresence]) -> usize {
        users
            .iter()
            .filter(|user| user.is_active(self.active_timeout))
            .count()
    }
}

impl Default for PresenceManager {
    fn default() -> Self {
        Self::new(30) // 30 seconds default timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_presence_creation() {
        let presence = UserPresence::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "hsl(0, 70%, 60%)".to_string(),
        );

        assert_eq!(presence.user_id, "user-1");
        assert_eq!(presence.user_name, "Alice");
        assert_eq!(presence.color, "hsl(0, 70%, 60%)");
        assert!(presence.cursor_position.is_none());
    }

    #[test]
    fn test_cursor_update() {
        let mut presence = UserPresence::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "hsl(0, 70%, 60%)".to_string(),
        );

        let cursor = CursorPosition {
            line: 10,
            column: 5,
            selection_start: None,
            selection_end: None,
        };

        presence.update_cursor(cursor.clone());
        assert!(presence.cursor_position.is_some());
        assert_eq!(presence.cursor_position.as_ref().unwrap().line, 10);
        assert_eq!(presence.cursor_position.as_ref().unwrap().column, 5);

        presence.clear_cursor();
        assert!(presence.cursor_position.is_none());
    }

    #[test]
    fn test_is_active() {
        let presence = UserPresence::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "hsl(0, 70%, 60%)".to_string(),
        );

        // Should be active immediately
        assert!(presence.is_active(30));

        // Create old presence
        let mut old_presence = presence.clone();
        old_presence.last_seen = chrono::Utc::now() - chrono::Duration::seconds(60);

        // Should not be active after 60 seconds with 30s timeout
        assert!(!old_presence.is_active(30));

        // Should be active with longer timeout
        assert!(old_presence.is_active(120));
    }

    #[test]
    fn test_presence_manager() {
        let manager = PresenceManager::new(30);

        let active_user = UserPresence::new(
            "user-1".to_string(),
            "Alice".to_string(),
            "hsl(0, 70%, 60%)".to_string(),
        );

        let mut inactive_user = UserPresence::new(
            "user-2".to_string(),
            "Bob".to_string(),
            "hsl(120, 70%, 60%)".to_string(),
        );
        inactive_user.last_seen = chrono::Utc::now() - chrono::Duration::seconds(60);

        let users = vec![active_user.clone(), inactive_user.clone()];

        let active = manager.filter_active(&users);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].user_name, "Alice");

        let count = manager.count_active(&users);
        assert_eq!(count, 1);
    }
}
