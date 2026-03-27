use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, State},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use serde_json::json;
use tokio::sync::broadcast;
use tracing::{info, error};
use uuid::Uuid;

use crate::AppState;
use crate::alerts::notifier::{NotificationEvent, NotificationHub};

#[derive(Debug, serde::Deserialize)]
struct WsMessage {
    message_type: String,
    payload: serde_json::Value,
}

#[derive(Debug, serde::Serialize)]
struct WsResponse {
    message_type: String,
    payload: serde_json::Value,
    timestamp: String,
}

pub async fn handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();
    
    // Subscribe to notifications
    let mut notification_rx = if let Some(hub) = &state.notification_hub {
        hub.subscribe()
    } else {
        // If no notification hub, just create a dummy channel
        let (tx, rx) = broadcast::channel(1);
        drop(tx);
        rx
    };

    // Send welcome message
    let welcome = WsResponse {
        message_type: "connected".to_string(),
        payload: json!({
            "message": "Connected to AI Merchant Assistant",
            "version": "1.0.0"
        }),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    
    if let Ok(text) = serde_json::to_string(&welcome) {
        let _ = sender.send(axum::extract::ws::Message::Text(text)).await;
    }

    // Spawn task to handle incoming WebSocket messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    info!("Received WebSocket message: {}", text);
                    
                    // Parse message
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        match ws_msg.message_type.as_str() {
                            "ping" => {
                                // Respond with pong
                                let pong = WsResponse {
                                    message_type: "pong".to_string(),
                                    payload: json!({}),
                                    timestamp: chrono::Utc::now().to_rfc3339(),
                                };
                                if let Ok(response) = serde_json::to_string(&pong) {
                                    // Would need to send through channel
                                }
                            }
                            "voice_stream" => {
                                // Handle voice streaming
                                info!("Received voice stream data");
                            }
                            "subscribe_alerts" => {
                                // Client wants to subscribe to alerts
                                info!("Client subscribed to alerts");
                            }
                            _ => {
                                info!("Unknown message type: {}", ws_msg.message_type);
                            }
                        }
                    }
                }
                axum::extract::ws::Message::Binary(audio_data) => {
                    info!("Received binary audio data: {} bytes", audio_data.len());
                    
                    // Process audio through AI orchestrator
                    // This would integrate with the AI system
                }
                axum::extract::ws::Message::Close(_) => {
                    info!("WebSocket connection closed by client");
                    break;
                }
                _ => {}
            }
        }
    });

    // Spawn task to forward notifications to client
    let mut send_task = tokio::spawn(async move {
        loop {
            match notification_rx.recv().await {
                Ok(event) => {
                    let response = match event {
                        NotificationEvent::NewAlert(alert) => WsResponse {
                            message_type: "new_alert".to_string(),
                            payload: serde_json::to_value(&alert).unwrap_or_default(),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                        NotificationEvent::TransactionUpdate { user_id, transaction_id } => WsResponse {
                            message_type: "transaction_update".to_string(),
                            payload: json!({
                                "user_id": user_id,
                                "transaction_id": transaction_id,
                            }),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                        NotificationEvent::SystemMessage { user_id, message } => WsResponse {
                            message_type: "system_message".to_string(),
                            payload: json!({
                                "user_id": user_id,
                                "message": message,
                            }),
                            timestamp: chrono::Utc::now().to_rfc3339(),
                        },
                        _ => continue,
                    };

                    if let Ok(text) = serde_json::to_string(&response) {
                        // In a real implementation, we'd need to share the sender
                        // For now, just log
                        info!("Would send notification: {}", text);
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => {
                    // Client is lagging, continue
                    continue;
                }
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut recv_task => {
            send_task.abort();
        }
        _ = &mut send_task => {
            recv_task.abort();
        }
    }

    info!("WebSocket connection ended");
}
