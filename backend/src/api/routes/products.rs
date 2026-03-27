use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::product::CreateProductRequest;
use crate::AppState;

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Mock response
    let products = json!([
        {
            "id": "123e4567-e89b-12d3-a456-426614174010",
            "name": "Eggs",
            "description": "Fresh farm eggs",
            "default_price": 10.00,
            "unit": "crate",
            "stock_quantity": 50,
            "low_stock_threshold": 10,
        },
        {
            "id": "123e4567-e89b-12d3-a456-426614174011",
            "name": "Milk",
            "description": "Whole milk",
            "default_price": 3.50,
            "unit": "liter",
            "stock_quantity": 100,
            "low_stock_threshold": 20,
        }
    ]);

    Ok(Json(json!({
        "success": true,
        "data": products
    })))
}

pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let product = json!({
        "id": Uuid::new_v4().to_string(),
        "name": request.name,
        "description": request.description,
        "sku": request.sku,
        "default_price": request.default_price,
        "cost_price": request.cost_price,
        "unit": request.unit.unwrap_or_else(|| "piece".to_string()),
        "stock_quantity": request.stock_quantity.unwrap_or(0),
        "low_stock_threshold": request.low_stock_threshold.unwrap_or(10),
        "is_active": true,
        "created_at": chrono::Utc::now().to_rfc3339(),
    });

    Ok(Json(json!({
        "success": true,
        "data": product
    })))
}
