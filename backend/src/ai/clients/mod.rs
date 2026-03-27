use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod huggingface;
pub mod together;
pub mod runpod;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub confidence: f32,
    pub language: String,
}

#[derive(Debug)]
pub enum AIError {
    Network(reqwest::Error),
    Parse(serde_json::Error),
    RateLimited,
    ServiceUnavailable,
    Other(String),
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIError::Network(e) => write!(f, "Network error: {}", e),
            AIError::Parse(e) => write!(f, "Parse error: {}", e),
            AIError::RateLimited => write!(f, "Rate limited"),
            AIError::ServiceUnavailable => write!(f, "Service unavailable"),
            AIError::Other(s) => write!(f, "Error: {}", s),
        }
    }
}

impl std::error::Error for AIError {}

#[async_trait]
pub trait CloudSTTClient: Send + Sync {
    async fn transcribe(&self, audio_bytes: Vec<u8>) -> Result<TranscriptionResult, AIError>;
}

#[async_trait]
pub trait CloudLLMClient: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String, AIError>;
    async fn extract_entities(&self, text: &str) -> Result<crate::models::transaction::ExtractedEntities, AIError>;
}

#[async_trait]
pub trait CloudTTSClient: Send + Sync {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, AIError>;
}

pub struct CloudClientFactory;

impl CloudClientFactory {
    pub fn create_stt_client(provider: &str, api_key: Option<String>) -> Box<dyn CloudSTTClient> {
        match provider {
            "huggingface" => Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default())),
            _ => Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default())),
        }
    }

    pub fn create_llm_client(provider: &str, api_key: Option<String>) -> Box<dyn CloudLLMClient> {
        match provider {
            "huggingface" => Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default())),
            _ => Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default())),
        }
    }

    pub fn create_tts_client(provider: &str, api_key: Option<String>) -> Box<dyn CloudTTSClient> {
        match provider {
            "huggingface" => Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default())),
            _ => Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default())),
        }
    }
}
