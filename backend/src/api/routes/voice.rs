use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};
use base64::Engine;

use crate::api::state::AppState;
use crate::utils::errors::ApiError;

#[derive(Debug, serde::Deserialize)]
pub struct TranscribeRequest {
    pub audio_data: String, // Base64 encoded
}

#[derive(Debug, serde::Deserialize)]
pub struct ProductSuggestionQuery {
    pub q: String,
}

/// Transcribe audio to text using AI
pub async fn transcribe(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TranscribeRequest>,
) -> Result<Json<Value>, ApiError> {
    // Decode base64 audio
    let audio_bytes = base64::engine::general_purpose::STANDARD.decode(&request.audio_data)
        .map_err(|_| ApiError::ValidationError("Invalid audio data format".to_string()))?;

    // Use AI orchestrator to transcribe
    let transcription = state
        .ai_orchestrator
        .transcribe_audio(audio_bytes)
        .await
        .map_err(|e| ApiError::AIProcessingError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": {
            "text": transcription.text,
            "confidence": transcription.confidence,
            "language": transcription.language,
        }
    })))
}

/// Synthesize text to speech
#[derive(Debug, serde::Deserialize)]
pub struct SynthesizeRequest {
    pub text: String,
}

pub async fn synthesize(
    State(state): State<Arc<AppState>>,
    Json(request): Json<SynthesizeRequest>,
) -> Result<Json<Value>, ApiError> {
    if request.text.trim().is_empty() {
        return Err(ApiError::ValidationError("Text is required".to_string()));
    }

    // Use AI orchestrator to synthesize speech
    let audio_bytes = state
        .ai_orchestrator
        .generate_response(&request.text)
        .await
        .map_err(|e| ApiError::AIProcessingError(e.to_string()))?;

    // Encode audio as base64 for JSON response
    let audio_base64 = base64::engine::general_purpose::STANDARD.encode(&audio_bytes);

    Ok(Json(json!({
        "success": true,
        "data": {
            "audio": audio_base64,
            "format": "wav",
        }
    })))
}
