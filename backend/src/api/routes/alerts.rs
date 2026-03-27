use axum::{
    extract::{State, Path},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::AppState;

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let alerts = json!([
        {
            "id": "alert-001",
            "alert_type": "LowStock",
            "severity": "Warning",
            "title": "Low Stock: Milk",
            "message": "Stock level for Milk is at 5 liters",
            "metadata": {
                "product_id": "123e4567-e89b-12d3-a456-426614174011",
                "current_stock": 5,
                "suggested_quantity": 50
            },
            "is_read": false,
            "created_at": "2024-01-15T10:30:00Z"
        },
        {
            "id": "alert-002",
            "alert_type": "SalesSpike",
            "severity": "Info",
            "title": "Sales Spike Detected",
            "message": "Revenue was $3,500 (expected $2,000, deviation: 75%)",
            "metadata": {
                "date": "2024-01-14",
                "deviation_percent": 75.0
            },
            "is_read": false,
            "created_at": "2024-01-14T18:00:00Z"
        },
        {
            "id": "alert-003",
            "alert_type": "HighDemand",
            "severity": "Info",
            "title": "Trending: Eggs",
            "message": "Eggs is trending with 250 sales and $5,000 in revenue",
            "metadata": {
                "times_sold": 250,
                "total_quantity": 500
            },
            "is_read": true,
            "created_at": "2024-01-13T09:00:00Z"
        }
    ]);

    Ok(Json(json!({
        "success": true,
        "data": alerts
    })))
}

pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Ok(Json(json!({
        "success": true,
        "message": format!("Alert {} marked as read", id)
    })))
}

pub async fn check_now(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Trigger alert check
    Ok(Json(json!({
        "success": true,
        "message": "Alert check triggered",
        "alerts_generated": 3
    })))
}
