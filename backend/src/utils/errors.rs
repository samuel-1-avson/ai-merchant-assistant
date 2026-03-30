use thiserror::Error;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Authentication error: {0}")]
    Auth(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("AI service error: {0}")]
    AIService(String),
    
    #[error("Not found")]
    NotFound,
    
    #[error("Internal error")]
    Internal,
}

/// API Error type for HTTP responses
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Authentication error: {0}")]
    Unauthorized(String),
    
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("AI processing error: {0}")]
    AIProcessingError(String),
    
    #[error("AI service error: {0}")]
    AIServiceError(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Internal server error: {0}")]
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::DatabaseError(msg) => {
                tracing::error!("Database error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error occurred".to_string())
            }
            ApiError::Unauthorized(msg) | ApiError::AuthenticationError(msg) => {
                (StatusCode::UNAUTHORIZED, msg)
            }
            ApiError::AIServiceError(msg) => {
                tracing::error!("AI service error: {}", msg);
                (StatusCode::SERVICE_UNAVAILABLE, "AI service unavailable".to_string())
            }
            ApiError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, msg)
            }
            ApiError::AIProcessingError(msg) => {
                tracing::error!("AI processing error: {}", msg);
                (StatusCode::SERVICE_UNAVAILABLE, "AI service temporarily unavailable".to_string())
            }
            ApiError::NotFound(msg) => {
                (StatusCode::NOT_FOUND, msg)
            }
            ApiError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };

        let body = Json(json!({
            "success": false,
            "error": error_message,
            "code": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        ApiError::DatabaseError(err.to_string())
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        tracing::error!("Application error: {:?}", err);
        ApiError::InternalError(err.to_string())
    }
}
