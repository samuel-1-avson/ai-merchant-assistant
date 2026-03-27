use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::models::user::{CreateUserRequest, LoginRequest};
use crate::api::state::AppState;
use crate::utils::errors::ApiError;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Extract user_id from JWT token when auth middleware is implemented
    
    // Create user using the UserService
    let user = state
        .user_service
        .create_user(&request)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "id": user.id,
            "email": user.email,
            "full_name": user.full_name,
            "business_name": user.business_name,
            "created_at": user.created_at,
            "updated_at": user.updated_at,
        },
        "message": "User registered successfully"
    })))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Extract user_id from JWT token when auth middleware is implemented
    
    // Authenticate user using the UserService
    let auth_response = state
        .user_service
        .authenticate(&request)
        .await
        .map_err(|e| ApiError::AuthenticationError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "user": {
                "id": auth_response.user.id,
                "email": auth_response.user.email,
                "full_name": auth_response.user.full_name,
                "business_name": auth_response.user.business_name,
                "created_at": auth_response.user.created_at,
                "updated_at": auth_response.user.updated_at,
            },
            "token": auth_response.token,
        },
        "message": "Login successful"
    })))
}
