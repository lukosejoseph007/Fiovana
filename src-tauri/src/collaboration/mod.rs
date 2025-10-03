// Collaboration module for real-time document editing
// Implements WebSocket server with Yjs protocol support

pub mod operational_transforms;
pub mod presence;
pub mod room_manager;
pub mod websocket_server;

pub use websocket_server::{CollaborationServer, ServerConfig};
