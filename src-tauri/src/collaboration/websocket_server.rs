// WebSocket server for Yjs collaboration
// Features:
// - Room management (one room per document)
// - Message broadcasting
// - User presence tracking
// - Conflict-free sync using Yjs protocol

use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::{error, info, warn};

use super::presence::UserPresence;
use super::room_manager::RoomManager;

/// WebSocket message types for Yjs protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Yjs sync step 1: Request state vector
    SyncStep1 {
        document_id: String,
    },
    /// Yjs sync step 2: Send state vector
    SyncStep2 {
        document_id: String,
        state_vector: Vec<u8>,
    },
    /// Yjs update: Document changes
    Update {
        document_id: String,
        update: Vec<u8>,
    },
    /// Awareness: User presence information
    Awareness {
        document_id: String,
        awareness: Vec<u8>,
    },
    /// Room join request
    Join {
        document_id: String,
        user_id: String,
        user_name: String,
    },
    /// Room leave notification
    Leave {
        document_id: String,
        user_id: String,
    },
    /// Ping/Pong for connection health
    Ping,
    Pong,
}

/// Configuration for the collaboration server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Host address
    pub host: String,
    /// Maximum connections per room
    pub max_connections_per_room: usize,
    /// Heartbeat interval in seconds
    #[allow(dead_code)]
    pub heartbeat_interval: u64,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 1234,
            host: "127.0.0.1".to_string(),
            max_connections_per_room: 50,
            heartbeat_interval: 30,
        }
    }
}

/// Main collaboration server managing WebSocket connections
pub struct CollaborationServer {
    config: ServerConfig,
    room_manager: Arc<RoomManager>,
    connections: Arc<DashMap<String, mpsc::UnboundedSender<Message>>>,
}

impl CollaborationServer {
    /// Create a new collaboration server
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config,
            room_manager: Arc::new(RoomManager::new()),
            connections: Arc::new(DashMap::new()),
        }
    }

    /// Start the WebSocket server
    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error>> {
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;

        info!("Collaboration server listening on {}", addr);

        while let Ok((stream, peer_addr)) = listener.accept().await {
            info!("New connection from: {}", peer_addr);
            let server = Arc::clone(&self);
            tokio::spawn(async move {
                if let Err(e) = server.handle_connection(stream).await {
                    error!("Error handling connection from {}: {}", peer_addr, e);
                }
            });
        }

        Ok(())
    }

    /// Handle a single WebSocket connection
    async fn handle_connection(&self, stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // Create a channel for this connection
        let (tx, mut rx) = mpsc::unbounded_channel();
        let connection_id = uuid::Uuid::new_v4().to_string();

        // Store connection sender
        self.connections.insert(connection_id.clone(), tx);

        // Spawn task to forward messages from channel to WebSocket
        let connection_id_clone = connection_id.clone();
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                if let Err(e) = ws_sender.send(message).await {
                    error!(
                        "Failed to send message to connection {}: {}",
                        connection_id_clone, e
                    );
                    break;
                }
            }
        });

        // Handle incoming messages
        while let Some(message) = ws_receiver.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Err(e) = self.handle_text_message(&connection_id, text).await {
                        error!("Error handling text message: {}", e);
                    }
                }
                Ok(Message::Binary(data)) => {
                    if let Err(e) = self.handle_binary_message(&connection_id, data).await {
                        error!("Error handling binary message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("Connection {} closed", connection_id);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    // Send pong response
                    if let Some(sender) = self.connections.get(&connection_id) {
                        let _ = sender.send(Message::Pong(data));
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Heartbeat received
                }
                Err(e) => {
                    error!("WebSocket error on connection {}: {}", connection_id, e);
                    break;
                }
                _ => {}
            }
        }

        // Clean up connection
        self.connections.remove(&connection_id);
        Ok(())
    }

    /// Handle text message (JSON protocol)
    async fn handle_text_message(
        &self,
        connection_id: &str,
        text: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let msg: WsMessage = serde_json::from_str(&text)?;

        match msg {
            WsMessage::Join {
                document_id,
                user_id,
                user_name,
            } => {
                self.handle_join(connection_id, &document_id, &user_id, &user_name)
                    .await?;
            }
            WsMessage::Leave {
                document_id,
                user_id,
            } => {
                self.handle_leave(&document_id, &user_id).await?;
            }
            WsMessage::Ping => {
                self.send_to_connection(connection_id, WsMessage::Pong)
                    .await?;
            }
            _ => {
                warn!("Unexpected text message type");
            }
        }

        Ok(())
    }

    /// Handle binary message (Yjs protocol)
    async fn handle_binary_message(
        &self,
        _connection_id: &str,
        data: Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Parse Yjs binary protocol
        // First byte indicates message type
        if data.is_empty() {
            return Ok(());
        }

        match data[0] {
            0 => {
                // Sync Step 1
                info!("Received Yjs Sync Step 1");
            }
            1 => {
                // Sync Step 2
                info!("Received Yjs Sync Step 2");
            }
            2 => {
                // Update
                info!("Received Yjs Update");
            }
            _ => {
                warn!("Unknown Yjs message type: {}", data[0]);
            }
        }

        Ok(())
    }

    /// Handle user joining a room
    async fn handle_join(
        &self,
        connection_id: &str,
        document_id: &str,
        user_id: &str,
        user_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "User {} ({}) joining room {}",
            user_name, user_id, document_id
        );

        // Get or create room
        let room = self
            .room_manager
            .get_or_create_room(document_id.to_string());

        // Add user presence
        let presence = UserPresence {
            user_id: user_id.to_string(),
            user_name: user_name.to_string(),
            color: Self::generate_user_color(user_id),
            cursor_position: None,
            last_seen: chrono::Utc::now(),
        };

        room.add_user(connection_id.to_string(), presence);

        // Broadcast to other users in the room
        self.broadcast_to_room(
            document_id,
            Some(connection_id),
            WsMessage::Awareness {
                document_id: document_id.to_string(),
                awareness: vec![], // Will be populated with actual awareness data
            },
        )
        .await?;

        Ok(())
    }

    /// Handle user leaving a room
    async fn handle_leave(
        &self,
        document_id: &str,
        user_id: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("User {} leaving room {}", user_id, document_id);

        if let Some(room) = self.room_manager.get_room(document_id) {
            room.remove_user_by_id(user_id);

            // Broadcast leave event
            self.broadcast_to_room(
                document_id,
                None,
                WsMessage::Leave {
                    document_id: document_id.to_string(),
                    user_id: user_id.to_string(),
                },
            )
            .await?;
        }

        Ok(())
    }

    /// Send message to a specific connection
    async fn send_to_connection(
        &self,
        connection_id: &str,
        msg: WsMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(sender) = self.connections.get(connection_id) {
            let json = serde_json::to_string(&msg)?;
            sender.send(Message::Text(json))?;
        }
        Ok(())
    }

    /// Broadcast message to all users in a room (except sender if specified)
    async fn broadcast_to_room(
        &self,
        document_id: &str,
        exclude_connection: Option<&str>,
        msg: WsMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(room) = self.room_manager.get_room(document_id) {
            let json = serde_json::to_string(&msg)?;

            for (connection_id, _) in room.get_users() {
                if Some(connection_id.as_str()) != exclude_connection {
                    if let Some(sender) = self.connections.get(&connection_id) {
                        let _ = sender.send(Message::Text(json.clone()));
                    }
                }
            }
        }

        Ok(())
    }

    /// Generate a consistent color for a user based on their ID
    fn generate_user_color(user_id: &str) -> String {
        // Simple hash-based color generation
        let hash = user_id
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));

        let hue = (hash % 360) as f32;
        format!("hsl({}, 70%, 60%)", hue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 1234);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.max_connections_per_room, 50);
    }

    #[test]
    fn test_generate_user_color() {
        let color1 = CollaborationServer::generate_user_color("user1");
        let color2 = CollaborationServer::generate_user_color("user1");
        let color3 = CollaborationServer::generate_user_color("user2");

        // Same user should get same color
        assert_eq!(color1, color2);

        // Different users should likely get different colors
        assert_ne!(color1, color3);

        // Color should be valid HSL format
        assert!(color1.starts_with("hsl("));
        assert!(color1.ends_with(")"));
    }
}
