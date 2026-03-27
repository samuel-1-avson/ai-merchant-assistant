use axum::{
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;
use base64::Engine;

use crate::models::transaction::{CreateTransactionRequest, CreateVoiceTransactionRequest};
use crate::api::state::AppState;
use crate::api::middleware::AuthUser;
use crate::utils::errors::ApiError;
use crate::realtime::websocket::broadcast_transaction_update;

#[derive(Debug, serde::Deserialize)]
pub struct ListTransactionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub product_id: Option<Uuid>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

/// List transactions for the authenticated user
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token after auth middleware is implemented
    // For now, using a placeholder - THIS MUST BE REPLACED WITH REAL AUTH
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    let limit = query.limit.unwrap_or(50).min(100); // Max 100 per page
    let offset = query.offset.unwrap_or(0);

    let transactions = state
        .transaction_service
        .list_transactions(user_id, limit, offset)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": transactions,
        "meta": {
            "limit": limit,
            "offset": offset,
            "count": transactions.len()
        }
    })))
}

/// Create a new transaction manually
pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token after auth middleware is implemented
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    // Validate request
    if request.quantity <= rust_decimal::Decimal::ZERO {
        return Err(ApiError::ValidationError("Quantity must be greater than 0".to_string()));
    }
    if request.price < rust_decimal::Decimal::ZERO {
        return Err(ApiError::ValidationError("Price cannot be negative".to_string()));
    }

    let transaction = state
        .transaction_service
        .create_transaction(user_id, request)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Broadcast real-time update via WebSocket
    if let Some(hub) = &state.notification_hub {
        broadcast_transaction_update(hub, user_id, transaction.id);
    }

    Ok(Json(json!({
        "success": true,
        "data": transaction,
        "message": "Transaction created successfully"
    })))
}

/// Create a transaction from voice input
pub async fn create_voice(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateVoiceTransactionRequest>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token after auth middleware is implemented
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    // Decode base64 audio
    let audio_bytes = base64::engine::general_purpose::STANDARD.decode(&request.audio_data)
        .map_err(|_| ApiError::ValidationError("Invalid audio data".to_string()))?;

    // Process through AI orchestrator
    let result = state
        .ai_orchestrator
        .process_voice_transaction(audio_bytes, user_id)
        .await
        .map_err(|e| ApiError::AIProcessingError(e.to_string()))?;

    // Broadcast real-time update via WebSocket
    if let Some(hub) = &state.notification_hub {
        broadcast_transaction_update(hub, user_id, result.transaction.id);
    }

    Ok(Json(json!({
        "success": true,
        "data": {
            "transaction": result.transaction,
            "transcription": result.transcription,
            "extracted_entities": result.extracted_entities,
        },
        "message": "Voice transaction processed successfully"
    })))
}

/// Get a single transaction by ID
pub async fn get_by_id(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token after auth middleware is implemented
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    // Get transaction from repository directly since we need by_id
    use crate::db::repositories::transaction_repo::TransactionRepository;
    let repo = TransactionRepository::new(state.db.pool.clone());
    
    let transaction = repo
        .get_by_id(id, user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    match transaction {
        Some(tx) => Ok(Json(json!({
            "success": true,
            "data": tx
        }))),
        None => Err(ApiError::NotFound("Transaction not found".to_string())),
    }
}
