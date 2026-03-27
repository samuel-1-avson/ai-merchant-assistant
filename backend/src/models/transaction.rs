use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub product_id: Option<Uuid>,
    pub quantity: Decimal,
    pub unit: String,
    pub price: Decimal,
    pub total: Decimal,
    pub notes: Option<String>,
    pub voice_recording_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTransactionRequest {
    pub product_id: Option<Uuid>,
    pub quantity: Decimal,
    pub unit: Option<String>,
    pub price: Decimal,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateVoiceTransactionRequest {
    pub audio_data: String, // Base64 encoded audio
}

#[derive(Debug, Clone, Serialize)]
pub struct VoiceTransactionResponse {
    pub transaction: Transaction,
    pub transcription: String,
    pub extracted_entities: ExtractedEntities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntities {
    pub product: Option<String>,
    pub quantity: Option<f64>,
    pub unit: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
}
