use axum::{
    extract::{State, Path, Query, Extension},
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::api::middleware::AuthUser;
use crate::utils::errors::ApiError;

#[derive(Debug, serde::Deserialize)]
pub struct ListAlertsQuery {
    pub unread_only: Option<bool>,
}

/// List alerts for the authenticated user
pub async fn list(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListAlertsQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;
    
    let unread_only = query.unread_only.unwrap_or(false);
    
    // Fetch alerts from the alert engine
    let alerts = state
        .alert_engine
        .get_alerts(user_id, unread_only)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Get alert counts
    let counts = state
        .alert_engine
        .get_alert_counts(user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "alerts": alerts,
            "counts": {
                "total_unread": counts.total_unread,
                "critical": counts.critical,
                "warning": counts.warning,
                "info": counts.info
            }
        },
        "message": "Alerts retrieved successfully"
    })))
}

/// Mark an alert as read
pub async fn mark_read(
    State(state): State<Arc<AppState>>,
    Extension(_auth_user): Extension<AuthUser>,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    
    let alert_id = Uuid::parse_str(&id)
        .map_err(|_| ApiError::ValidationError("Invalid alert ID".to_string()))?;
    
    // Mark alert as read
    state
        .alert_engine
        .mark_as_read(alert_id)
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

/// Mark all alerts as read
pub async fn mark_all_read(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;
    
    // Mark all alerts as read
    let count = state
        .alert_engine
        .mark_all_as_read(user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "marked_as_read": count
        },
        "message": format!("{} alerts marked as read", count)
    })))
}

/// Trigger alert check manually
pub async fn check_now(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;
    
    // Run all alert checks
    let generated_alerts = state
        .alert_engine
        .check_all(user_id)
        .await
        .map_err(|e| ApiError::InternalError(e.to_string()))?;

    let alert_count = generated_alerts.len();

    Ok(Json(json!({
        "success": true,
        "data": {
            "alerts_generated": alert_count,
            "alerts": generated_alerts.iter().map(|a| json!({
                "id": a.id,
                "type": format!("{:?}", a.alert_type),
                "severity": format!("{:?}", a.severity),
                "title": &a.title,
                "message": &a.message
            })).collect::<Vec<_>>()
        },
        "message": format!("Alert check completed. {} new alerts generated.", alert_count)
    })))
}

/// Get alert counts summary
pub async fn counts(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;
    
    let counts = state
        .alert_engine
        .get_alert_counts(user_id)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "total_unread": counts.total_unread,
            "critical": counts.critical,
            "warning": counts.warning,
            "info": counts.info
        },
        "message": "Alert counts retrieved"
    })))
}
