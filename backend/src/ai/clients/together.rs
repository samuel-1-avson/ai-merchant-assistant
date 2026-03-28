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
        // Use the chat completions endpoint so instruction-tuned models
        // (Llama 3.1, Mistral Instruct, etc.) receive properly formatted input.
        let response = self.client
            .post("https://api.together.xyz/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&json!({
                "model": "meta-llama/Meta-Llama-3.1-8B-Instruct-Turbo",
                "messages": [
                    {"role": "user", "content": prompt}
                ],
                "max_tokens": 512,
                "temperature": 0.7
            }))
            .send()
            .await
            .map_err(AIError::Network)?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AIError::Other(format!("Together API error {}: {}", status, body)));
        }

        let result: serde_json::Value = response.json().await.map_err(AIError::Network)?;

        // Chat completions format: choices[0].message.content
        let text = result["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        if text.is_empty() {
            return Err(AIError::Other("Together returned empty content".to_string()));
        }

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

        // Extract JSON block — model may emit surrounding text
        let json_str = crate::ai::clients::huggingface::extract_json_block(&response)
            .ok_or_else(|| AIError::Other(format!("No JSON in Together response: {}", response)))?;

        let entities: crate::models::transaction::ExtractedEntities =
            serde_json::from_str(&json_str).map_err(AIError::Parse)?;

        Ok(entities)
    }
}
