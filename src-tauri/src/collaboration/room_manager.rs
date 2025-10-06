// Room management for collaborative editing sessions
// Each room represents a single document being edited

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::presence::UserPresence;

/// Information about a collaboration room
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    pub document_id: String,
    pub active_users: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

/// A collaboration room for a single document
pub struct Room {
    #[allow(dead_code)]
    document_id: String,
    users: Arc<DashMap<String, UserPresence>>, // connection_id -> UserPresence
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
    last_activity: Arc<tokio::sync::RwLock<chrono::DateTime<chrono::Utc>>>,
    // Yjs document state storage
    updates: Arc<tokio::sync::RwLock<Vec<Vec<u8>>>>, // Stored Yjs updates
    state_vector: Arc<tokio::sync::RwLock<Vec<u8>>>, // Current state vector
}

impl Room {
    /// Create a new room for a document
    pub fn new(document_id: String) -> Self {
        Self {
            document_id,
            users: Arc::new(DashMap::new()),
            created_at: chrono::Utc::now(),
            last_activity: Arc::new(tokio::sync::RwLock::new(chrono::Utc::now())),
            updates: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            state_vector: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Add a user to the room
    pub fn add_user(&self, connection_id: String, presence: UserPresence) {
        self.users.insert(connection_id, presence);
        self.update_activity();
    }

    /// Remove a user by connection ID
    #[allow(dead_code)]
    pub fn remove_user(&self, connection_id: &str) {
        self.users.remove(connection_id);
        self.update_activity();
    }

    /// Remove a user by user ID (may have multiple connections)
    pub fn remove_user_by_id(&self, user_id: &str) {
        self.users.retain(|_, presence| presence.user_id != user_id);
        self.update_activity();
    }

    /// Get all users in the room
    pub fn get_users(&self) -> Vec<(String, UserPresence)> {
        self.users
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Get user count
    #[allow(dead_code)]
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Check if room is empty
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    /// Update last activity timestamp
    fn update_activity(&self) {
        if let Ok(mut last_activity) = self.last_activity.try_write() {
            *last_activity = chrono::Utc::now();
        }
    }

    /// Get room information
    #[allow(dead_code)]
    pub async fn get_info(&self) -> RoomInfo {
        let last_activity = *self.last_activity.read().await;
        RoomInfo {
            document_id: self.document_id.clone(),
            active_users: self.user_count(),
            created_at: self.created_at,
            last_activity,
        }
    }

    /// Get the current state vector
    pub fn get_state_vector(&self) -> Vec<u8> {
        // Try to read the state vector, return empty if locked
        if let Ok(state_vector) = self.state_vector.try_read() {
            state_vector.clone()
        } else {
            Vec::new()
        }
    }

    /// Get updates since a given state vector
    pub fn get_updates_since(&self, client_state_vector: &[u8]) -> Vec<u8> {
        // Parse client state vector to determine which updates they need
        // For a simplified implementation, we check if client has any state
        if let Ok(updates) = self.updates.try_read() {
            if client_state_vector.is_empty() {
                // Client has no state, send all updates
                updates
                    .iter()
                    .flat_map(|update| update.iter())
                    .copied()
                    .collect()
            } else {
                // Client has some state, send recent updates
                // In a full implementation, we would parse the state vector properly
                // For now, we send the last few updates that might be missing
                let recent_count = std::cmp::min(10, updates.len());
                if recent_count > 0 {
                    updates[updates.len() - recent_count..]
                        .iter()
                        .flat_map(|update| update.iter())
                        .copied()
                        .collect()
                } else {
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        }
    }

    /// Apply a new update to the document
    pub fn apply_update(&self, update: Vec<u8>) {
        // Store the update
        if let Ok(mut updates) = self.updates.try_write() {
            updates.push(update.clone());

            // Update the state vector
            // In a full implementation, this would properly compute the new state vector
            // For now, we'll use a simplified approach
            drop(updates);

            if let Ok(mut state_vector) = self.state_vector.try_write() {
                // Simplified: append update length as state
                state_vector.extend_from_slice(&(update.len() as u32).to_le_bytes());
            }
        }

        self.update_activity();
    }
}

/// Manages all active collaboration rooms
pub struct RoomManager {
    rooms: Arc<DashMap<String, Arc<Room>>>,
}

impl RoomManager {
    /// Create a new room manager
    pub fn new() -> Self {
        Self {
            rooms: Arc::new(DashMap::new()),
        }
    }

    /// Get or create a room for a document
    pub fn get_or_create_room(&self, document_id: String) -> Arc<Room> {
        self.rooms
            .entry(document_id.clone())
            .or_insert_with(|| Arc::new(Room::new(document_id)))
            .clone()
    }

    /// Get an existing room
    pub fn get_room(&self, document_id: &str) -> Option<Arc<Room>> {
        self.rooms.get(document_id).map(|entry| entry.clone())
    }

    /// Remove a room (e.g., when empty)
    #[allow(dead_code)]
    pub fn remove_room(&self, document_id: &str) {
        self.rooms.remove(document_id);
    }

    /// Clean up empty rooms
    #[allow(dead_code)]
    pub fn cleanup_empty_rooms(&self) {
        self.rooms.retain(|_, room| !room.is_empty());
    }

    /// Get all active rooms
    #[allow(dead_code)]
    pub fn get_all_rooms(&self) -> Vec<Arc<Room>> {
        self.rooms
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Get room count
    #[allow(dead_code)]
    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    /// Get total user count across all rooms
    #[allow(dead_code)]
    pub fn total_user_count(&self) -> usize {
        self.rooms.iter().map(|entry| entry.user_count()).sum()
    }
}

impl Default for RoomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_room_creation() {
        let room = Room::new("doc-123".to_string());
        assert_eq!(room.user_count(), 0);
        assert!(room.is_empty());

        let info = room.get_info().await;
        assert_eq!(info.document_id, "doc-123");
        assert_eq!(info.active_users, 0);
    }

    #[tokio::test]
    async fn test_room_users() {
        let room = Room::new("doc-123".to_string());

        let presence = UserPresence {
            user_id: "user-1".to_string(),
            user_name: "Alice".to_string(),
            color: "hsl(0, 70%, 60%)".to_string(),
            cursor_position: None,
            last_seen: chrono::Utc::now(),
        };

        room.add_user("conn-1".to_string(), presence.clone());
        assert_eq!(room.user_count(), 1);
        assert!(!room.is_empty());

        let users = room.get_users();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].1.user_name, "Alice");

        room.remove_user("conn-1");
        assert_eq!(room.user_count(), 0);
        assert!(room.is_empty());
    }

    #[test]
    fn test_room_manager() {
        let manager = RoomManager::new();
        assert_eq!(manager.room_count(), 0);

        let room1 = manager.get_or_create_room("doc-1".to_string());
        assert_eq!(manager.room_count(), 1);

        let room2 = manager.get_or_create_room("doc-1".to_string());
        assert_eq!(manager.room_count(), 1);

        // Should be the same room
        assert_eq!(Arc::ptr_eq(&room1, &room2), true);

        let room3 = manager.get_or_create_room("doc-2".to_string());
        assert_eq!(manager.room_count(), 2);
        assert_eq!(Arc::ptr_eq(&room1, &room3), false);
    }

    #[test]
    fn test_cleanup_empty_rooms() {
        let manager = RoomManager::new();

        let room1 = manager.get_or_create_room("doc-1".to_string());
        let _room2 = manager.get_or_create_room("doc-2".to_string());

        let presence = UserPresence {
            user_id: "user-1".to_string(),
            user_name: "Alice".to_string(),
            color: "hsl(0, 70%, 60%)".to_string(),
            cursor_position: None,
            last_seen: chrono::Utc::now(),
        };

        room1.add_user("conn-1".to_string(), presence);

        assert_eq!(manager.room_count(), 2);

        manager.cleanup_empty_rooms();

        assert_eq!(manager.room_count(), 1);
        assert!(manager.get_room("doc-1").is_some());
        assert!(manager.get_room("doc-2").is_none());
    }
}
