use axum::{
    extract::{State, Multipart},
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::api::state::AppState;
use crate::utils::errors::ApiError;

pub async fn process_receipt(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    // TODO: Extract user_id from JWT token when auth middleware is implemented
    
    // Extract image from multipart form
    let mut image_data = Vec::new();
    
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::ValidationError(format!("Failed to read form field: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            image_data = field
                .bytes()
                .await
                .map_err(|e| ApiError::ValidationError(format!("Failed to read image data: {}", e)))?
                .to_vec();
            break;
        }
    }

    if image_data.is_empty() {
        return Err(ApiError::ValidationError("No image provided".to_string()));
    }

    // Process receipt using the AI orchestrator
    let receipt = state
        .ai_orchestrator
        .process_receipt_image(&image_data)
        .await
        .map_err(|e| ApiError::AIServiceError(format!("Failed to process receipt: {}", e)))?;

    Ok(Json(json!({
        "success": true,
        "data": receipt,
        "message": "Receipt processed successfully"
    })))
}

pub async fn scan_product(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    // TODO: Extract user_id from JWT token when auth middleware is implemented
    
    // Extract image from multipart form
    let mut image_data = Vec::new();
    
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::ValidationError(format!("Failed to read form field: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            image_data = field
                .bytes()
                .await
                .map_err(|e| ApiError::ValidationError(format!("Failed to read image data: {}", e)))?
                .to_vec();
            break;
        }
    }

    if image_data.is_empty() {
        return Err(ApiError::ValidationError("No image provided".to_string()));
    }

    // Process product scan using the AI orchestrator
    let product_info = state
        .ai_orchestrator
        .scan_product_image(&image_data)
        .await
        .map_err(|e| ApiError::AIServiceError(format!("Failed to scan product: {}", e)))?;

    Ok(Json(json!({
        "success": true,
        "data": product_info,
        "message": "Product scanned successfully"
    })))
}
