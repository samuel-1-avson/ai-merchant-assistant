use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tracing::{info, warn};

use super::{CloudLLMClient, AIError};

/// Groq LLM client — free tier, very fast (GroqChip hardware).
///
/// Free limits (as of 2026): 30 req/min, 6 000 tokens/min.
/// Models: llama-3.1-8b-instant, llama3-groq-8b-8192-tool-use-preview, etc.
/// API key: https://console.groq.com/keys
pub struct GroqClient {
    client: Client,
    api_key: String,
}

impl GroqClient {
    pub fn new(api_key: String) -> Self {
        Self { client: Client::new(), api_key }
    }
}

#[async_trait]
impl CloudLLMClient for GroqClient {
    async fn generate(&self, prompt: &str) -> Result<String, AIError> {
        // Retry with exponential backoff on rate-limit (429).
        // Free tier: 30 req/min — brief waits recover quickly.
        const MAX_RETRIES: u32 = 3;
        let mut delay = Duration::from_millis(500);

        for attempt in 0..MAX_RETRIES {
            let response = self.client
                .post("https://api.groq.com/openai/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&json!({
                    "model": "llama-3.1-8b-instant",
                    "messages": [{"role": "user", "content": prompt}],
                    "max_tokens": 512,
                    "temperature": 0.7
                }))
                .send()
                .await
                .map_err(AIError::Network)?;

            let status = response.status();

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                if attempt < MAX_RETRIES - 1 {
                    warn!("Groq rate limited (attempt {}), retrying in {:?}", attempt + 1, delay);
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                    continue;
                }
                return Err(AIError::RateLimited);
            }
            if !status.is_success() {
                let body = response.text().await.unwrap_or_default();
                return Err(AIError::Other(format!("Groq API error {}: {}", status, body)));
            }

            let result: serde_json::Value = response.json().await.map_err(AIError::Network)?;

            let text = result["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .trim()
                .to_string();

            if text.is_empty() {
                return Err(AIError::Other("Groq returned empty content".to_string()));
            }

            info!("Groq generated {} chars", text.len());
            return Ok(text);
        }

        Err(AIError::RateLimited)
    }

    async fn extract_entities(&self, text: &str) -> Result<crate::models::transaction::ExtractedEntities, AIError> {
        let prompt = format!(
            r#"You are a merchant sales assistant. Extract sales entities from this text: "{}"

Rules:
- "product": the item name being sold (NOT numbers or prices). If only a price is mentioned with no product, use null.
- "quantity": how many units were sold (default 1 if not stated)
- "unit": the measurement unit (piece, kg, crate, dozen, bottle, etc.) — use null if not mentioned
- "price": the unit price as a NUMBER ONLY (e.g. 45 not "$45"). Use null if no price is mentioned.
- "currency": "USD" by default unless another currency is stated

Examples:
- "sold 3 shirts at $45 each" → {{"product":"shirts","quantity":3,"unit":"piece","price":45,"currency":"USD"}}
- "egg crate, no price yet" → {{"product":"egg crate","quantity":1,"unit":"crate","price":null,"currency":"USD"}}
- "the price was $20" → {{"product":null,"quantity":null,"unit":null,"price":20,"currency":"USD"}}
- "five hoodies at $80" → {{"product":"hoodie","quantity":5,"unit":"piece","price":80,"currency":"USD"}}
- "new order 2 caps" → {{"product":"caps","quantity":2,"unit":"piece","price":null,"currency":"USD"}}

Respond ONLY with valid JSON, no explanation:
{{
    "product": "product name or null",
    "quantity": number or null,
    "unit": "unit or null",
    "price": number or null,
    "currency": "USD"
}}"#,
            text
        );

        let response = self.generate(&prompt).await?;

        let json_str = crate::ai::clients::huggingface::extract_json_block(&response)
            .ok_or_else(|| AIError::Other(format!("No JSON in Groq response: {}", response)))?;

        let entities: crate::models::transaction::ExtractedEntities =
            serde_json::from_str(&json_str).map_err(AIError::Parse)?;

        Ok(entities)
    }
}
