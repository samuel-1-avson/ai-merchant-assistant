use axum::{
    extract::{State, Path},
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::utils::errors::ApiError;

pub async fn list(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Extract user_id from JWT token when auth middleware is implemented
    let user_id = Uuid::nil(); // Placeholder - replace with actual user_id from JWT
    
    // Get notification hub from AppState
    let notification_hub = state
        .notification_hub
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Notification hub not available".to_string()))?;
    
    // Fetch real alerts from the database
    let alerts = notification_hub
        .get_unread_notifications(&user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": alerts,
        "message": "Alerts retrieved successfully"
    })))
}

pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Extract user_id from JWT token when auth middleware is implemented
    let user_id = Uuid::nil(); // Placeholder - replace with actual user_id from JWT
    
    // Parse the alert ID
    let alert_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::ValidationError("Invalid alert ID".to_string()))?;
    
    // Get notification hub from AppState
    let notification_hub = state
        .notification_hub
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Notification hub not available".to_string()))?;
    
    // Mark alert as read
    notification_hub
        .mark_as_read(&alert_id, &user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "alert_id": alert_id.to_string(),
            "is_read": true
        },
        "message": format!("Alert {} marked as read", id)
    })))
}

pub async fn check_now(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Extract user_id from JWT token when auth middleware is implemented
    let user_id = Uuid::nil(); // Placeholder - replace with actual user_id from JWT
    
    // Get notification hub from AppState
    let notification_hub = state
        .notification_hub
        .as_ref()
        .ok_or_else(|| ApiError::InternalError("Notification hub not available".to_string()))?;
    
    // Trigger alert check and get generated alerts
    let generated_alerts = notification_hub
        .check_and_generate_alerts(&user_id)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let alert_count = generated_alerts.len();

    Ok(Json(json!({
        "success": true,
        "data": {
            "alerts_generated": alert_count,
            "alerts": generated_alerts
        },
        "message": format!("Alert check completed. {} new alerts generated.", alert_count)
    })))
}
