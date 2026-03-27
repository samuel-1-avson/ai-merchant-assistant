use axum::{
    extract::{State, Multipart},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::AppState;
use crate::ocr::receipt_parser::ReceiptParser;

pub async fn process_receipt(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Extract image from multipart form
    let mut image_data = Vec::new();
    
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();
        if name == "image" {
            image_data = field.bytes().await.unwrap().to_vec();
            break;
        }
    }

    if image_data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "No image provided"
            }))
        ));
    }

    // Process with OCR (mock implementation)
    // In production, this would call the EasyOCR service
    let raw_text = "Grocery Store\nDate: 01/15/2024\n\nEggs $5.00\nMilk $3.50\nTotal: $8.50";
    
    // Parse receipt
    let receipt = ReceiptParser::parse(raw_text);

    Ok(Json(json!({
        "success": true,
        "data": receipt
    })))
}

pub async fn scan_product(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Similar to process_receipt but for single product identification
    Ok(Json(json!({
        "success": true,
        "data": {
            "product_name": "Organic Eggs",
            "price": 5.99,
            "confidence": 0.95
        }
    })))
}
