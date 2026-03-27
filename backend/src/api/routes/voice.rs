use axum::{
    extract::State,
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct TranscribeRequest {
    pub audio_data: String, // Base64 encoded
}

pub async fn transcribe(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TranscribeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Mock transcription - in production, this would call the AI client
    let response = json!({
        "success": true,
        "data": {
            "text": "Sold 2 crates of eggs for 10 dollars each",
            "confidence": 0.95,
            "language": "en"
        }
    });

    Ok(Json(response))
}
