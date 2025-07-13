/// WebSocket Module for Real-time Updates
///
/// This module provides WebSocket functionality for real-time updates
/// to the web interface, including file progress, governance updates,
/// and system notifications.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::api_server::ApiState;

/// WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// File upload progress
    FileUploadProgress {
        file_key: String,
        progress: f64,
        status: String,
    },
    /// File download progress
    FileDownloadProgress {
        file_key: String,
        progress: f64,
        status: String,
    },
    /// System status update
    SystemStatus {
        status: String,
        message: String,
    },
    /// Cache statistics update
    CacheStats {
        hit_ratio: f64,
        cache_size: u64,
    },
    /// Governance update
    GovernanceUpdate {
        event_type: String,
        data: serde_json::Value,
    },
    /// Network health update
    NetworkHealth {
        total_operators: usize,
        online_operators: usize,
        online_percentage: f64,
        can_reach_consensus: bool,
    },
    /// Operator status change
    OperatorStatusChange {
        operator_id: String,
        status: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Admin action executed
    AdminActionExecuted {
        action_id: String,
        action_type: String,
        executor: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Heartbeat for connection keep-alive
    Heartbeat {
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// WebSocket connection information
#[derive(Debug, Clone)]
pub struct WebSocketConnection {
    pub id: Uuid,
    pub connected_at: Instant,
    pub last_ping: Instant,
    pub subscriptions: Vec<String>,
}

/// WebSocket manager for handling connections and broadcasting
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<Uuid, WebSocketConnection>>>,
    broadcast_sender: broadcast::Sender<WebSocketMessage>,
}

impl WebSocketManager {
    /// Create a new WebSocket manager
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            broadcast_sender: tx,
        }
    }

    /// Get subscriber for receiving messages
    pub fn subscribe(&self) -> broadcast::Receiver<WebSocketMessage> {
        self.broadcast_sender.subscribe()
    }

    /// Broadcast a message to all connected clients
    pub async fn broadcast(&self, message: WebSocketMessage) {
        match self.broadcast_sender.send(message.clone()) {
            Ok(_) => debug!("Broadcasted message: {:?}", message),
            Err(e) => warn!("Failed to broadcast message: {}", e),
        }
    }

    /// Add a new connection
    pub async fn add_connection(&self, connection: WebSocketConnection) {
        let mut connections = self.connections.write().await;
        connections.insert(connection.id, connection);
        info!("WebSocket connection added. Total connections: {}", connections.len());
    }

    /// Remove a connection
    pub async fn remove_connection(&self, connection_id: &Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);
        info!("WebSocket connection removed. Total connections: {}", connections.len());
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Send heartbeat to all connections
    pub async fn send_heartbeat(&self) {
        let heartbeat = WebSocketMessage::Heartbeat {
            timestamp: chrono::Utc::now(),
        };
        self.broadcast(heartbeat).await;
    }

    /// Cleanup inactive connections
    pub async fn cleanup_inactive_connections(&self) {
        let mut connections = self.connections.write().await;
        let now = Instant::now();
        let timeout = Duration::from_secs(300); // 5 minutes

        connections.retain(|_, conn| {
            now.duration_since(conn.last_ping) < timeout
        });
    }
}

/// WebSocket handler for new connections
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle individual WebSocket connection
async fn handle_websocket(socket: WebSocket, state: ApiState) {
    let connection_id = Uuid::new_v4();
    let connection = WebSocketConnection {
        id: connection_id,
        connected_at: Instant::now(),
        last_ping: Instant::now(),
        subscriptions: vec![],
    };

    // Use the shared WebSocket manager from API state
    let ws_manager = state.websocket_manager.clone();
    ws_manager.add_connection(connection).await;

    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_receiver = ws_manager.subscribe();

    // Send welcome message
    let welcome_message = WebSocketMessage::SystemStatus {
        status: "connected".to_string(),
        message: "WebSocket connection established".to_string(),
    };
    
    if let Err(e) = sender.send(Message::Text(
        serde_json::to_string(&welcome_message).unwrap_or_default()
    )).await {
        error!("Failed to send welcome message: {}", e);
        return;
    }

    // Start heartbeat task
    let ws_manager_clone = ws_manager.clone();
    let heartbeat_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            ws_manager_clone.send_heartbeat().await;
        }
    });

    // Start cleanup task
    let ws_manager_clone = ws_manager.clone();
    let cleanup_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            ws_manager_clone.cleanup_inactive_connections().await;
        }
    });

    // Handle messages from client
    let client_handler = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    debug!("Received WebSocket message: {}", text);
                    // Handle client messages (subscription changes, etc.)
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by client");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    if let Err(e) = sender.send(Message::Pong(data)).await {
                        error!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Update last ping time
                    debug!("Received pong from client");
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }
    });

    // Handle broadcast messages
    let broadcast_handler = tokio::spawn(async move {
        while let Ok(message) = broadcast_receiver.recv().await {
            let message_text = match serde_json::to_string(&message) {
                Ok(text) => text,
                Err(e) => {
                    error!("Failed to serialize message: {}", e);
                    continue;
                }
            };

            if let Err(e) = sender.send(Message::Text(message_text)).await {
                error!("Failed to send broadcast message: {}", e);
                break;
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        _ = client_handler => debug!("Client handler completed"),
        _ = broadcast_handler => debug!("Broadcast handler completed"),
        _ = heartbeat_task => debug!("Heartbeat task completed"),
        _ = cleanup_task => debug!("Cleanup task completed"),
    }

    // Cleanup
    ws_manager.remove_connection(&connection_id).await;
    info!("WebSocket connection {} closed", connection_id);
}

/// Utility functions for sending specific message types
impl WebSocketManager {
    /// Send file upload progress update
    pub async fn send_file_upload_progress(&self, file_key: String, progress: f64, status: String) {
        let message = WebSocketMessage::FileUploadProgress {
            file_key,
            progress,
            status,
        };
        self.broadcast(message).await;
    }

    /// Send file download progress update
    pub async fn send_file_download_progress(&self, file_key: String, progress: f64, status: String) {
        let message = WebSocketMessage::FileDownloadProgress {
            file_key,
            progress,
            status,
        };
        self.broadcast(message).await;
    }

    /// Send system status update
    pub async fn send_system_status(&self, status: String, message: String) {
        let msg = WebSocketMessage::SystemStatus { status, message };
        self.broadcast(msg).await;
    }

    /// Send cache statistics update
    pub async fn send_cache_stats(&self, hit_ratio: f64, cache_size: u64) {
        let message = WebSocketMessage::CacheStats {
            hit_ratio,
            cache_size,
        };
        self.broadcast(message).await;
    }

    /// Send governance update
    pub async fn send_governance_update(&self, event_type: String, data: serde_json::Value) {
        let message = WebSocketMessage::GovernanceUpdate {
            event_type,
            data,
        };
        self.broadcast(message).await;
    }

    /// Send network health update
    pub async fn send_network_health(&self, total_operators: usize, online_operators: usize, online_percentage: f64, can_reach_consensus: bool) {
        let message = WebSocketMessage::NetworkHealth {
            total_operators,
            online_operators,
            online_percentage,
            can_reach_consensus,
        };
        self.broadcast(message).await;
    }

    /// Send operator status change
    pub async fn send_operator_status_change(&self, operator_id: String, status: String) {
        let message = WebSocketMessage::OperatorStatusChange {
            operator_id,
            status,
            timestamp: chrono::Utc::now(),
        };
        self.broadcast(message).await;
    }

    /// Send admin action executed notification
    pub async fn send_admin_action_executed(&self, action_id: String, action_type: String, executor: String) {
        let message = WebSocketMessage::AdminActionExecuted {
            action_id,
            action_type,
            executor,
            timestamp: chrono::Utc::now(),
        };
        self.broadcast(message).await;
    }
}
