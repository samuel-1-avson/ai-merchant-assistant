use axum::{
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::transaction::{CreateTransactionRequest, CreateVoiceTransactionRequest};
use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct ListTransactionsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);

    // Mock response
    let transactions = json!([
        {
            "id": "123e4567-e89b-12d3-a456-426614174001",
            "product_name": "Eggs",
            "quantity": 2,
            "unit": "crate",
            "price": 10.00,
            "total": 20.00,
            "created_at": "2024-01-15T10:30:00Z"
        },
        {
            "id": "123e4567-e89b-12d3-a456-426614174002",
            "product_name": "Milk",
            "quantity": 5,
            "unit": "liter",
            "price": 3.50,
            "total": 17.50,
            "created_at": "2024-01-15T11:00:00Z"
        }
    ]);

    Ok(Json(json!({
        "success": true,
        "data": transactions
    })))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateTransactionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Mock response
    let transaction = json!({
        "id": Uuid::new_v4().to_string(),
        "product_id": request.product_id,
        "quantity": request.quantity,
        "unit": request.unit,
        "price": request.price,
        "total": request.quantity * request.price,
        "notes": request.notes,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(json!({
        "success": true,
        "data": transaction
    })))
}

pub async fn create_voice(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateVoiceTransactionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Mock voice transaction processing
    let response = json!({
        "success": true,
        "data": {
            "transaction": {
                "id": Uuid::new_v4().to_string(),
                "product_name": "Eggs",
                "quantity": 2,
                "unit": "crate",
                "price": 10.00,
                "total": 20.00,
                "created_at": chrono::Utc::now().to_rfc3339(),
            },
            "transcription": "Sold 2 crates of eggs for 10 dollars each",
            "extracted_entities": {
                "product": "eggs",
                "quantity": 2,
                "unit": "crate",
                "price": 10.00,
                "currency": "USD"
            }
        }
    });

    Ok(Json(response))
}
