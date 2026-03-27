use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::{CloudSTTClient, CloudLLMClient, CloudTTSClient, TranscriptionResult, AIError};

pub struct HuggingFaceClient {
    client: Client,
    api_token: String,
}

impl HuggingFaceClient {
    pub fn new(api_token: String) -> Self {
        Self {
            client: Client::new(),
            api_token,
        }
    }
}

#[async_trait]
impl CloudSTTClient for HuggingFaceClient {
    async fn transcribe(&self, audio_bytes: Vec<u8>) -> Result<TranscriptionResult, AIError> {
        // Using Whisper Large V3 Turbo via HuggingFace Inference API
        let response = self.client
            .post("https://api-inference.huggingface.co/models/openai/whisper-large-v3-turbo")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .body(audio_bytes)
            .send()
            .await
            .map_err(AIError::Network)?;

        let result: serde_json::Value = response.json().await.map_err(AIError::Network)?;
        
        let text = result["text"].as_str().unwrap_or("").to_string();
        
        Ok(TranscriptionResult {
            text,
            confidence: result["confidence"].as_f64().unwrap_or(1.0) as f32,
            language: result["language"].as_str().unwrap_or("en").to_string(),
        })
    }
}

#[async_trait]
impl CloudLLMClient for HuggingFaceClient {
    async fn generate(&self, prompt: &str) -> Result<String, AIError> {
        // Using Llama 3.1 8B via HuggingFace Inference API
        let response = self.client
            .post("https://api-inference.huggingface.co/models/meta-llama/Llama-3.1-8B-Instruct")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&json!({
                "inputs": prompt,
                "parameters": {
                    "max_new_tokens": 500,
                    "temperature": 0.7,
                    "return_full_text": false
                }
            }))
            .send()
            .await
            .map_err(AIError::Network)?;

        let result: Vec<serde_json::Value> = response.json().await.map_err(AIError::Network)?;
        
        let text = result.get(0)
            .and_then(|v| v["generated_text"].as_str())
            .unwrap_or("")
            .to_string();
            
        Ok(text)
    }

    async fn extract_entities(&self, text: &str) -> Result<crate::models::transaction::ExtractedEntities, AIError> {
        let prompt = format!(
            r#"Extract entities from this sales transaction text: "{}"

Respond ONLY with valid JSON in this exact format:
{{
    "product": "product name or null",
    "quantity": number or null,
    "unit": "unit (piece, kg, crate, etc.) or null",
    "price": number or null,
    "currency": "USD, EUR, etc. or null"
}}

If any field is not found, use null."#,
            text
        );

        let response = self.generate(&prompt).await?;
        let entities: crate::models::transaction::ExtractedEntities = 
            serde_json::from_str(&response).map_err(AIError::Parse)?;
        
        Ok(entities)
    }
}

#[async_trait]
impl CloudTTSClient for HuggingFaceClient {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, AIError> {
        // MeloTTS via HuggingFace
        let response = self.client
            .post("https://api-inference.huggingface.co/models/myshell-ai/MeloTTS-English")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&json!({"inputs": text}))
            .send()
            .await
            .map_err(AIError::Network)?;

        let bytes = response.bytes().await.map_err(AIError::Network)?;
        Ok(bytes.to_vec())
    }
}
