use thiserror::Error;

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
