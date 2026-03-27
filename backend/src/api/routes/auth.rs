use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::models::user::{CreateUserRequest, LoginRequest, AuthResponse};
use crate::AppState;

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // For now, return a mock response
    // In production, integrate with Supabase Auth
    let response = json!({
        "success": true,
        "message": "User registered successfully",
        "user": {
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "email": request.email,
            "full_name": request.full_name,
            "business_name": request.business_name,
        },
        "token": "mock_jwt_token"
    });

    Ok(Json(response))
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // For now, return a mock response
    let response = json!({
        "success": true,
        "message": "Login successful",
        "user": {
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "email": request.email,
        },
        "token": "mock_jwt_token"
    });

    Ok(Json(response))
}
