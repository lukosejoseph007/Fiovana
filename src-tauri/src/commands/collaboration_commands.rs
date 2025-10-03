// Tauri commands for collaboration server management

use crate::collaboration::{CollaborationServer, ServerConfig};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

/// State for collaboration server
pub type CollaborationServerState = Arc<RwLock<Option<Arc<CollaborationServer>>>>;

/// Request to start collaboration server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartServerRequest {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub max_connections_per_room: Option<usize>,
}

/// Response with server status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub host: String,
    pub active_rooms: usize,
    pub total_users: usize,
}

/// Start the collaboration server
#[tauri::command]
pub async fn start_collaboration_server(
    request: StartServerRequest,
    state: tauri::State<'_, CollaborationServerState>,
) -> Result<ServerStatus, String> {
    info!("Starting collaboration server with config: {:?}", request);

    let mut server_lock = state.write().await;

    // Check if server is already running
    if server_lock.is_some() {
        return Err("Collaboration server is already running".to_string());
    }

    // Create server config
    let mut config = ServerConfig::default();
    if let Some(port) = request.port {
        config.port = port;
    }
    if let Some(host) = request.host {
        config.host = host;
    }
    if let Some(max_connections) = request.max_connections_per_room {
        config.max_connections_per_room = max_connections;
    }

    // Create and start server
    let server = Arc::new(CollaborationServer::new(config.clone()));
    let server_clone: Arc<CollaborationServer> = Arc::clone(&server);

    // Start server in background
    tokio::spawn(async move {
        if let Err(e) = server_clone.start().await {
            error!("Collaboration server error: {}", e);
        }
    });

    // Store server reference
    *server_lock = Some(server);

    Ok(ServerStatus {
        running: true,
        port: config.port,
        host: config.host,
        active_rooms: 0,
        total_users: 0,
    })
}

/// Stop the collaboration server
#[tauri::command]
pub async fn stop_collaboration_server(
    state: tauri::State<'_, CollaborationServerState>,
) -> Result<String, String> {
    info!("Stopping collaboration server");

    let mut server_lock = state.write().await;

    if server_lock.is_none() {
        return Err("Collaboration server is not running".to_string());
    }

    // Drop the server (this will close all connections)
    *server_lock = None;

    Ok("Collaboration server stopped successfully".to_string())
}

/// Get collaboration server status
#[tauri::command]
pub async fn get_collaboration_server_status(
    state: tauri::State<'_, CollaborationServerState>,
) -> Result<ServerStatus, String> {
    let server_lock = state.read().await;

    if let Some(_server) = server_lock.as_ref() {
        // In a full implementation, we would query the server for stats
        Ok(ServerStatus {
            running: true,
            port: 1234, // Default port
            host: "127.0.0.1".to_string(),
            active_rooms: 0, // Would query from room manager
            total_users: 0,  // Would query from room manager
        })
    } else {
        Ok(ServerStatus {
            running: false,
            port: 0,
            host: "".to_string(),
            active_rooms: 0,
            total_users: 0,
        })
    }
}

/// Get available collaboration server info (for frontend configuration)
#[tauri::command]
pub async fn get_collaboration_server_info() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "default_port": 1234,
        "default_host": "127.0.0.1",
        "websocket_url": "ws://127.0.0.1:1234",
        "supports_webrtc_fallback": true,
        "max_document_size_mb": 10,
        "max_concurrent_users": 50,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_collaboration_server_info() {
        let info = get_collaboration_server_info().await.unwrap();

        assert!(info.get("default_port").is_some());
        assert!(info.get("websocket_url").is_some());
    }
}
