use axum::{
    extract::{State, Multipart},
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::utils::errors::ApiError;
use crate::ocr::OCRProcessingResult;

/// Process receipt image
pub async fn process_receipt(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    // Find image field in multipart
    let mut image_bytes: Option<Vec<u8>> = None;
    
    while let Some(field) = multipart.next_field().await
        .map_err(|e| ApiError::ValidationError(format!("Invalid multipart: {}", e)))? 
    {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "image" || name == "file" {
            let bytes = field.bytes().await
                .map_err(|e| ApiError::ValidationError(format!("Failed to read file: {}", e)))?;
            image_bytes = Some(bytes.to_vec());
            break;
        }
    }

    let image_bytes = image_bytes.ok_or_else(|| 
        ApiError::ValidationError("No image file provided".to_string())
    )?;

    // Validate image size (max 10MB)
    if image_bytes.len() > 10 * 1024 * 1024 {
        return Err(ApiError::ValidationError("Image too large (max 10MB)".to_string()));
    }

    // Process receipt
    let result = state
        .ocr_service
        .process_receipt(user_id, image_bytes)
        .await
        .map_err(|e| ApiError::AIProcessingError(e.to_string()))?;

    Ok(Json(json!({
        "success": result.success,
        "data": {
            "receipt": result.receipt,
            "matched_products": result.matched_products.iter().map(|(item, product)| {
                json!({
                    "item": item,
                    "matched_product": product,
                    "product_id": product.as_ref().map(|p| p.product.id)
                })
            }).collect::<Vec<_>>(),
            "processing_time_ms": result.processing_time_ms,
        },
        "errors": result.errors,
        "message": if result.success {
            "Receipt processed successfully"
        } else {
            "Failed to process receipt"
        }
    })))
}

/// Create transactions from processed receipt
#[derive(Debug, serde::Deserialize)]
pub struct CreateFromReceiptRequest {
    pub receipt_data: OCRProcessingResult,
    pub confirmed_item_ids: Vec<Uuid>,
}

pub async fn create_transactions(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateFromReceiptRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    let transactions = state
        .ocr_service
        .create_transactions_from_receipt(user_id, &request.receipt_data, &request.confirmed_item_ids)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "transactions_created": transactions.len(),
            "transactions": transactions,
        },
        "message": format!("Created {} transactions from receipt", transactions.len())
    })))
}

/// Scan product image (for future use)
pub async fn scan_product(
    State(_state): State<Arc<AppState>>,
    _multipart: Multipart,
) -> Result<Json<Value>, ApiError> {
    // Placeholder for product image recognition
    Ok(Json(json!({
        "success": false,
        "message": "Product scanning not yet implemented"
    })))
}

/// Test OCR with sample receipt (for development)
pub async fn test_ocr(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    // Sample receipt text for testing parser
    let sample_text = r#"Grocery Store
123 Main Street
Date: 2024-01-15

Eggs (dozen) $5.99
Milk 1gal $3.49
Bread $2.99
Apples (2lb) $4.50

Subtotal: $16.97
Tax: $1.19
Total: $18.16"#;

    use crate::ocr::receipt_parser::ReceiptParser;
    
    let receipt = ReceiptParser::parse(sample_text);

    // Match items to products
    let matched = state
        .ocr_service
        .process_receipt(user_id, vec![])
        .await
        .map_err(|e| ApiError::AIProcessingError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "parsed_receipt": receipt,
            "sample_text": sample_text,
        },
        "message": "OCR test completed"
    })))
}
