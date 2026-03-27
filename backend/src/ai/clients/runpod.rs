use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use base64::Engine;

use super::{CloudSTTClient, CloudLLMClient, TranscriptionResult, AIError};

pub struct RunPodClient {
    client: Client,
    endpoint_url: String,
    api_key: String,
}

impl RunPodClient {
    pub fn new(endpoint_url: String, api_key: String) -> Self {
        Self {
            client: Client::new(),
            endpoint_url,
            api_key,
        }
    }
}

#[async_trait]
impl CloudSTTClient for RunPodClient {
    async fn transcribe(&self, audio_bytes: Vec<u8>) -> Result<TranscriptionResult, AIError> {
        let response = self.client
            .post(&self.endpoint_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "input": {
                    "audio": base64::engine::general_purpose::STANDARD.encode(&audio_bytes),
                    "task": "transcribe"
                }
            }))
            .send()
            .await
            .map_err(AIError::Network)?;

        let result: serde_json::Value = response.json().await.map_err(AIError::Network)?;
        
        Ok(TranscriptionResult {
            text: result["text"].as_str().unwrap_or("").to_string(),
            confidence: result["confidence"].as_f64().unwrap_or(1.0) as f32,
            language: result["language"].as_str().unwrap_or("en").to_string(),
        })
    }
}

#[async_trait]
impl CloudLLMClient for RunPodClient {
    async fn generate(&self, prompt: &str) -> Result<String, AIError> {
        let response = self.client
            .post(&self.endpoint_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "input": {
                    "prompt": prompt,
                    "max_tokens": 500
                }
            }))
            .send()
            .await
            .map_err(AIError::Network)?;

        let result: serde_json::Value = response.json().await.map_err(AIError::Network)?;
        
        let text = result["output"].as_str()
            .or_else(|| result["text"].as_str())
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
