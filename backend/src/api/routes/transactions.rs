use axum::{
    extract::{State, Query, Path, Extension},
    Json,
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
use crate::ai::orchestrator::VoiceProcessingResult;

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
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let transactions = state
        .transaction_service
        .list_transactions(user_id, limit, offset)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "transactions": transactions,
            "meta": {
                "limit": limit,
                "offset": offset,
                "count": transactions.len()
            }
        }
    })))
}

/// Create a new transaction manually
pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

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
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateVoiceTransactionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    let audio_data = base64::engine::general_purpose::STANDARD
        .decode(&request.audio_data)
        .map_err(|e| ApiError::ValidationError(format!("Invalid audio data: {}", e)))?;

    if audio_data.len() > 10 * 1024 * 1024 {
        return Err(ApiError::ValidationError("Audio file too large (max 10MB)".to_string()));
    }

    let result = state
        .ai_orchestrator
        .process_voice_transaction(audio_data, user_id)
        .await
        .map_err(|e| ApiError::AIProcessingError(e.to_string()))?;

    match result {
        VoiceProcessingResult::Immediate(response) => {
            if let Some(hub) = &state.notification_hub {
                broadcast_transaction_update(hub, user_id, response.transaction.id);
            }

            Ok(Json(json!({
                "success": true,
                "data": {
                    "type": "immediate",
                    "transaction": response.transaction,
                    "transcription": response.transcription,
                    "extracted_entities": response.extracted_entities,
                },
                "message": "Transaction created successfully"
            })))
        }
        VoiceProcessingResult::Pending(confirmation) => {
            Ok(Json(json!({
                "success": true,
                "data": {
                    "type": "pending",
                    "pending_confirmation": confirmation,
                },
                "message": "Please confirm this transaction"
            })))
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ConfirmTransactionRequest {
    pub confirmation_id: Uuid,
}

pub async fn confirm(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ConfirmTransactionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    let transaction = state
        .ai_orchestrator
        .confirm_transaction(&request.confirmation_id, user_id)
        .await
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    if let Some(hub) = &state.notification_hub {
        broadcast_transaction_update(hub, user_id, transaction.id);
    }

    Ok(Json(json!({
        "success": true,
        "data": transaction,
        "message": "Transaction confirmed and created"
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct RejectTransactionRequest {
    pub confirmation_id: Uuid,
}

pub async fn reject(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<RejectTransactionRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    state
        .ai_orchestrator
        .reject_transaction(&request.confirmation_id, user_id)
        .await
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "message": "Transaction rejected"
    })))
}

pub async fn pending_confirmations(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    let confirmations = state
        .ai_orchestrator
        .get_pending_confirmations(user_id)
        .await;

    Ok(Json(json!({
        "success": true,
        "data": confirmations,
        "count": confirmations.len()
    })))
}

/// POST /api/v1/transactions/confirmations/:id/confirm
pub async fn confirm_by_id(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(confirmation_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    let transaction = state
        .ai_orchestrator
        .confirm_transaction(&confirmation_id, user_id)
        .await
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    if let Some(hub) = &state.notification_hub {
        broadcast_transaction_update(hub, user_id, transaction.id);
    }

    Ok(Json(json!({
        "success": true,
        "data": transaction,
        "message": "Transaction confirmed and created"
    })))
}

/// POST /api/v1/transactions/confirmations/:id/reject
pub async fn reject_by_id(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Path(confirmation_id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    state
        .ai_orchestrator
        .reject_transaction(&confirmation_id, user_id)
        .await
        .map_err(|e| ApiError::ValidationError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "message": "Transaction rejected"
    })))
}
