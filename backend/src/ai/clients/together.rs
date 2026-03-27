use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

use super::{CloudLLMClient, AIError};

pub struct TogetherClient {
    client: Client,
    api_key: String,
}

impl TogetherClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
        }
    }
}

#[async_trait]
impl CloudLLMClient for TogetherClient {
    async fn generate(&self, prompt: &str) -> Result<String, AIError> {
        let response = self.client
            .post("https://api.together.xyz/v1/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": "meta-llama/Llama-3.1-8B-Instruct",
                "prompt": prompt,
                "max_tokens": 500,
                "temperature": 0.7
            }))
            .send()
            .await
            .map_err(AIError::Network)?;

        let result: serde_json::Value = response.json().await.map_err(AIError::Network)?;
        
        let text = result["choices"][0]["text"]
            .as_str()
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
