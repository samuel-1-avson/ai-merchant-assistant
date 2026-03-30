use axum::{
    extract::{ws::{WebSocket, WebSocketUpgrade}, Query, State},
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{broadcast, mpsc};
use tracing::{info, error, warn};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::alerts::notifier::{NotificationEvent, NotificationHub};

/// WebSocket connection query parameters
#[derive(Debug, Deserialize)]
pub struct WsQuery {
    token: Option<String>,
}

/// WebSocket client message types
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsMessage {
    Ping,
    Subscribe { channel: String },
    Unsubscribe { channel: String },
    MarkAlertRead { alert_id: Uuid },
    VoiceStream { data: String },
}

/// WebSocket server response types
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsResponse {
    Connected { message: String, user_id: Option<String> },
    Pong,
    Error { message: String },
    NewAlert { alert: serde_json::Value },
    TransactionUpdate { user_id: Uuid, transaction_id: Uuid, transaction: Option<serde_json::Value> },
    SystemMessage { message: String },
    VoiceResult { transcription: String, intent: serde_json::Value },
    Notification { title: String, message: String, severity: String },
}

/// Authenticated WebSocket connection info
#[derive(Clone, Debug)]
struct WsConnection {
    user_id: Option<Uuid>,
    subscribed_channels: Vec<String>,
}

/// WebSocket upgrade handler with JWT authentication
pub async fn handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    // Authenticate from query token
    let user_id = if let Some(token) = query.token {
        authenticate_token(&token, &state).ok()
    } else {
        None
    };

    ws.on_upgrade(move |socket| handle_socket(socket, state, user_id))
}

/// Authenticate JWT token and return user_id
fn authenticate_token(token: &str, state: &AppState) -> anyhow::Result<Uuid> {
    let claims = state.jwt_validator.validate(token)?;
    let user_id = Uuid::parse_str(&claims.sub)?;
    Ok(user_id)
}

/// Main WebSocket connection handler
async fn handle_socket(
    socket: WebSocket,
    state: Arc<AppState>,
    user_id: Option<Uuid>,
) {
    let (mut sender, mut receiver) = socket.split();
    
    // Channel for sending messages to the client
    let (tx, mut rx) = mpsc::channel::<String>(100);
    
    // Subscribe to notifications
    let mut notification_rx = if let Some(hub) = &state.notification_hub {
        hub.subscribe()
    } else {
        let (tx, rx) = broadcast::channel(1);
        drop(tx);
        rx
    };

    let connection = WsConnection {
        user_id,
        subscribed_channels: vec!["alerts".to_string(), "transactions".to_string()],
    };

    info!("WebSocket connected: user_id={:?}", user_id);

    // Send connected message
    let connected_msg = WsResponse::Connected {
        message: "Connected to AI Merchant Assistant".to_string(),
        user_id: user_id.map(|u| u.to_string()),
    };
    
    if let Ok(text) = serde_json::to_string(&connected_msg) {
        let _ = sender.send(axum::extract::ws::Message::Text(text)).await;
    }

    // Clone for tasks
    let conn = connection.clone();
    let state_clone = state.clone();
    let tx_for_recv = tx.clone();

    // Task 1: Handle incoming messages from client
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                axum::extract::ws::Message::Text(text) => {
                    info!("Received WebSocket message: {}", text);
                    
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        let response = handle_client_message(ws_msg, &conn, &state_clone).await;
                        if let Ok(json) = serde_json::to_string(&response) {
                            if let Err(e) = tx_for_recv.send(json).await {
                                error!("Failed to send to channel: {}", e);
                                break;
                            }
                        }
                    } else {
                        let error = WsResponse::Error {
                            message: "Invalid message format".to_string(),
                        };
                        if let Ok(json) = serde_json::to_string(&error) {
                            let _ = tx_for_recv.send(json).await;
                        }
                    }
                }
                axum::extract::ws::Message::Binary(data) => {
                    info!("Received binary data: {} bytes", data.len());
                    // Handle voice/audio streaming
                    handle_binary_data(data, &conn, &state_clone, &tx_for_recv).await;
                }
                axum::extract::ws::Message::Close(_) => {
                    info!("WebSocket closed by client");
                    break;
                }
                _ => {}
            }
        }
    });

    // Task 2: Send messages to client (from internal channel)
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(axum::extract::ws::Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task 3: Forward notifications from notification hub
    let conn_clone = connection.clone();
    let tx_clone = tx.clone();
    let mut notify_task = tokio::spawn(async move {
        loop {
            match notification_rx.recv().await {
                Ok(event) => {
                    // Filter events for this user
                    let should_send = match &event {
                        NotificationEvent::NewAlert(alert) => {
                            conn_clone.user_id.map_or(false, |uid| uid == alert.user_id)
                        }
                        NotificationEvent::AlertRead { .. } => true,
                        NotificationEvent::TransactionUpdate { user_id, .. } => {
                            conn_clone.user_id.map_or(false, |uid| &uid == user_id)
                        }
                        NotificationEvent::SystemMessage { user_id, .. } => {
                            conn_clone.user_id.map_or(false, |cuid| cuid == *user_id)
                        }
                    };

                    if should_send {
                        let response = match event {
                            NotificationEvent::NewAlert(alert) => WsResponse::NewAlert {
                                alert: serde_json::to_value(&alert).unwrap_or_default(),
                            },
                            NotificationEvent::TransactionUpdate { user_id, transaction_id } => {
                                WsResponse::TransactionUpdate {
                                    user_id,
                                    transaction_id,
                                    transaction: None,
                                }
                            }
                            NotificationEvent::SystemMessage { message, .. } => {
                                WsResponse::SystemMessage { message }
                            }
                            _ => continue,
                        };

                        if let Ok(json) = serde_json::to_string(&response) {
                            if tx_clone.send(json).await.is_err() {
                                break;
                            }
                        }
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });

    // Wait for any task to complete
    tokio::select! {
        _ = &mut recv_task => {
            send_task.abort();
            notify_task.abort();
        }
        _ = &mut send_task => {
            recv_task.abort();
            notify_task.abort();
        }
        _ = &mut notify_task => {
            recv_task.abort();
            send_task.abort();
        }
    }

    info!("WebSocket connection ended: user_id={:?}", user_id);
}

/// Handle client messages and return response
async fn handle_client_message(
    msg: WsMessage,
    conn: &WsConnection,
    state: &Arc<AppState>,
) -> WsResponse {
    match msg {
        WsMessage::Ping => WsResponse::Pong,
        WsMessage::Subscribe { channel } => {
            info!("Client subscribed to channel: {}", channel);
            WsResponse::SystemMessage {
                message: format!("Subscribed to {}", channel),
            }
        }
        WsMessage::Unsubscribe { channel } => {
            info!("Client unsubscribed from channel: {}", channel);
            WsResponse::SystemMessage {
                message: format!("Unsubscribed from {}", channel),
            }
        }
        WsMessage::MarkAlertRead { alert_id } => {
            if let Some(user_id) = conn.user_id {
                if let Some(hub) = &state.notification_hub {
                    let _ = hub.mark_as_read(&alert_id, &user_id).await;
                }
            }
            WsResponse::SystemMessage {
                message: "Alert marked as read".to_string(),
            }
        }
        WsMessage::VoiceStream { data } => {
            // Decode base64 audio and process
            info!("Received voice stream data: {} bytes", data.len());
            WsResponse::SystemMessage {
                message: "Voice data received".to_string(),
            }
        }
    }
}

/// Handle binary data (audio streaming)
async fn handle_binary_data(
    data: Vec<u8>,
    conn: &WsConnection,
    state: &Arc<AppState>,
    tx: &mpsc::Sender<String>,
) {
    use crate::ai::orchestrator::VoiceProcessingResult;
    
    // Process audio through AI orchestrator if available
    let orchestrator = &state.ai_orchestrator;
    if let Some(user_id) = conn.user_id {
        // Process voice transaction
        match orchestrator.process_voice_transaction(data, user_id).await {
            Ok(VoiceProcessingResult::Immediate(response)) => {
                let ws_response = WsResponse::VoiceResult {
                    transcription: response.transcription,
                    intent: json!({
                        "type": "immediate",
                        "transaction": response.transaction,
                        "extracted_entities": response.extracted_entities,
                    }),
                };
                if let Ok(json) = serde_json::to_string(&ws_response) {
                    let _ = tx.send(json).await;
                }
            }
            Ok(VoiceProcessingResult::Pending(confirmation)) => {
                let ws_response = WsResponse::VoiceResult {
                    transcription: confirmation.original_transcription.clone(),
                    intent: json!({
                        "type": "pending_confirmation",
                        "confirmation_id": confirmation.id,
                        "confirmation": confirmation,
                    }),
                };
                if let Ok(json) = serde_json::to_string(&ws_response) {
                    let _ = tx.send(json).await;
                }
            }
            Ok(VoiceProcessingResult::AwaitingPrice { transaction_id, product_name, transcription }) => {
                let ws_response = WsResponse::VoiceResult {
                    transcription: transcription.clone(),
                    intent: json!({
                        "type": "awaiting_price",
                        "transaction_id": transaction_id,
                        "product_name": product_name,
                        "transcription": transcription,
                    }),
                };
                if let Ok(json) = serde_json::to_string(&ws_response) {
                    let _ = tx.send(json).await;
                }
            }
            Err(e) => {
                let error = WsResponse::Error {
                    message: format!("Voice processing error: {}", e),
                };
                if let Ok(json) = serde_json::to_string(&error) {
                    let _ = tx.send(json).await;
                }
            }
        }
    }
}

/// Broadcast transaction update to connected clients
pub fn broadcast_transaction_update(
    hub: &NotificationHub,
    user_id: Uuid,
    transaction_id: Uuid,
) {
    hub.notify_transaction_update(user_id, transaction_id);
}

/// Broadcast alert to connected clients
pub fn broadcast_alert(hub: &NotificationHub, alert: crate::alerts::Alert) {
    hub.notify_new_alert(alert);
}
