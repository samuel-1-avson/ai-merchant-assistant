use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub mod huggingface;
pub mod together;
pub mod runpod;
pub mod groq;
pub mod provider;

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

/// Result from a vision-capable model analyzing an image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionResult {
    /// Raw text or structured JSON extracted from the image
    pub extracted_text: String,
    /// Rough confidence score (0.0 – 1.0); may be estimated
    pub confidence: f32,
    /// Which model produced this result (for logging/metrics)
    pub model_used: String,
}

/// Vision client for multimodal image understanding.
///
/// Primary model : **llava-hf/llava-1.5-7b-hf** (HuggingFace free inference)
/// Handles receipt OCR and product image recognition.
#[async_trait]
pub trait CloudVisionClient: Send + Sync {
    /// Extract all visible text from an image.
    async fn extract_text_from_image(&self, image_bytes: &[u8]) -> Result<VisionResult, AIError>;

    /// Analyse a receipt image and return structured JSON describing
    /// merchant name, date, line items (name/qty/price), and total.
    async fn analyze_receipt(&self, image_bytes: &[u8]) -> Result<VisionResult, AIError>;
}

pub use provider::{
    AIClientBuilder, FailoverSTTClient, FailoverLLMClient, FailoverTTSClient,
    FailoverVisionClient, ProviderConfig, ProviderHealth,
};

/// Legacy factory - maintained for backwards compatibility
/// Use AIClientBuilder for new code with failover support
pub struct CloudClientFactory;

impl CloudClientFactory {
    /// Creates STT client - deprecated, use AIClientBuilder
    #[deprecated(since = "1.0.0", note = "Use AIClientBuilder for failover support")]
    pub fn create_stt_client(_provider: &str, api_key: Option<String>) -> Box<dyn CloudSTTClient> {
        Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default()))
    }

    /// Creates LLM client - deprecated, use AIClientBuilder
    #[deprecated(since = "1.0.0", note = "Use AIClientBuilder for failover support")]
    pub fn create_llm_client(_provider: &str, api_key: Option<String>) -> Box<dyn CloudLLMClient> {
        Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default()))
    }

    /// Creates TTS client - deprecated, use AIClientBuilder
    #[deprecated(since = "1.0.0", note = "Use AIClientBuilder for failover support")]
    pub fn create_tts_client(_provider: &str, api_key: Option<String>) -> Box<dyn CloudTTSClient> {
        Box::new(huggingface::HuggingFaceClient::new(api_key.unwrap_or_default()))
    }
}
