use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;
use base64::Engine as _;
use tracing::{info, warn};

use super::{
    CloudSTTClient, CloudLLMClient, CloudTTSClient, CloudVisionClient,
    TranscriptionResult, VisionResult, AIError,
};

// ── Text / STT / TTS client ────────────────────────────────────────────────

pub struct HuggingFaceClient {
    client: Client,
    api_token: String,
}

// ── Vision client (LLaVA-1.5-7B) ──────────────────────────────────────────

/// Multimodal vision client backed by **llava-hf/llava-1.5-7b-hf**.
///
/// This is a genuine vision-language model that can read text in images,
/// making it suitable for receipt OCR and product image recognition.
pub struct HuggingFaceVisionClient {
    client: Client,
    api_token: String,
}

impl HuggingFaceVisionClient {
    pub fn new(api_token: String) -> Self {
        Self { client: Client::new(), api_token }
    }

    /// Build the LLaVA chat prompt embedding an image as a base64 data-URI.
    fn build_vision_prompt(image_bytes: &[u8], instruction: &str) -> (String, String) {
        let b64 = base64::engine::general_purpose::STANDARD.encode(image_bytes);
        let data_uri = format!("data:image/jpeg;base64,{}", b64);
        // LLaVA uses the <image> token inside the USER turn
        let prompt = format!("USER: <image>\n{} ASSISTANT:", instruction);
        (prompt, data_uri)
    }

    /// Call the LLaVA-1.5-7B endpoint on HuggingFace Inference API.
    async fn call_llava(&self, image_bytes: &[u8], instruction: &str) -> Result<String, AIError> {
        let (prompt, data_uri) = Self::build_vision_prompt(image_bytes, instruction);

        let payload = json!({
            "inputs": prompt,
            "parameters": {
                "images": [data_uri],
                "max_new_tokens": 1000,
                "temperature": 0.1,
                "return_full_text": false
            }
        });

        let response = self.client
            .post("https://router.huggingface.co/hf-inference/models/llava-hf/llava-1.5-7b-hf")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&payload)
            .send()
            .await
            .map_err(AIError::Network)?;

        let status = response.status();

        // HuggingFace returns 503 with {"error": "Loading..."} while the
        // model cold-starts — treat that as ServiceUnavailable for the
        // failover system to handle.
        if status == reqwest::StatusCode::SERVICE_UNAVAILABLE {
            warn!("LLaVA model is loading (503) — will retry or failover");
            return Err(AIError::ServiceUnavailable);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AIError::RateLimited);
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(AIError::Other(format!("HF vision API error {}: {}", status, body)));
        }

        let result: serde_json::Value = response.json().await.map_err(AIError::Network)?;

        // LLaVA returns [{generated_text: "..."}]
        let text = result
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v["generated_text"].as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        if text.is_empty() {
            return Err(AIError::Other("LLaVA returned empty response".to_string()));
        }

        info!("LLaVA extracted {} chars of text from image", text.len());
        Ok(text)
    }
}

#[async_trait]
impl CloudVisionClient for HuggingFaceVisionClient {
    async fn extract_text_from_image(&self, image_bytes: &[u8]) -> Result<VisionResult, AIError> {
        let instruction = "You are an OCR assistant. Extract ALL visible text from this image \
            exactly as it appears, preserving newlines and the original layout. \
            Do not add explanations — output only the raw extracted text.";

        let text = self.call_llava(image_bytes, instruction).await?;

        Ok(VisionResult {
            extracted_text: text,
            confidence: 0.85,
            model_used: "llava-hf/llava-1.5-7b-hf".to_string(),
        })
    }

    async fn analyze_receipt(&self, image_bytes: &[u8]) -> Result<VisionResult, AIError> {
        let instruction = r#"You are a receipt analysis assistant. Read this receipt image and \
respond ONLY with valid JSON in this exact format — no extra text:
{
  "merchant_name": "Store name or null",
  "receipt_date": "YYYY-MM-DD or null",
  "total_amount": 123.45,
  "items": [
    {
      "name": "Product name",
      "quantity": 1.0,
      "unit": "piece",
      "price": 10.00,
      "total": 10.00
    }
  ]
}
Extract every line item. Use null for missing fields. Use "piece" as default unit."#;

        let raw = self.call_llava(image_bytes, instruction).await?;

        // Extract JSON block from the model output (model may add extra text)
        let json_text = extract_json_block(&raw).unwrap_or(raw.clone());

        Ok(VisionResult {
            extracted_text: json_text,
            confidence: 0.85,
            model_used: "llava-hf/llava-1.5-7b-hf".to_string(),
        })
    }
}

impl HuggingFaceClient {
    pub fn new(api_token: String) -> Self {
        Self {
            client: Client::new(),
            api_token,
        }
    }

    /// Call any HuggingFace endpoint and handle common error codes centrally.
    async fn hf_post_json(&self, url: &str, body: serde_json::Value) -> Result<serde_json::Value, AIError> {
        let response = self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body)
            .send()
            .await
            .map_err(AIError::Network)?;

        let status = response.status();
        if status == reqwest::StatusCode::SERVICE_UNAVAILABLE {
            warn!("HF model loading (503) at {}", url);
            return Err(AIError::ServiceUnavailable);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AIError::RateLimited);
        }
        if !status.is_success() {
            let body_text = response.text().await.unwrap_or_default();
            return Err(AIError::Other(format!("HF API {} error {}: {}", url, status, body_text)));
        }

        response.json::<serde_json::Value>().await.map_err(AIError::Network)
    }
}

#[async_trait]
impl CloudSTTClient for HuggingFaceClient {
    async fn transcribe(&self, audio_bytes: Vec<u8>) -> Result<TranscriptionResult, AIError> {
        // Whisper Large V3 Turbo — binary audio body, NOT JSON.
        // Content-Type must be application/octet-stream or the model-specific audio type.
        let response = self.client
            .post("https://router.huggingface.co/hf-inference/models/openai/whisper-large-v3-turbo")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "audio/wav")
            .body(audio_bytes)
            .send()
            .await
            .map_err(AIError::Network)?;

        let status = response.status();
        if status == reqwest::StatusCode::SERVICE_UNAVAILABLE {
            warn!("Whisper model is loading (503)");
            return Err(AIError::ServiceUnavailable);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AIError::RateLimited);
        }
        if !status.is_success() {
            return Err(AIError::ServiceUnavailable);
        }

        let result: serde_json::Value = response.json().await.map_err(AIError::Network)?;
        let text = result["text"].as_str().unwrap_or("").trim().to_string();

        // Whisper returns `avg_logprob` on some endpoints; use it as a proxy
        // for confidence. Default to 0.8 (Whisper is generally reliable).
        let confidence = result["avg_logprob"]
            .as_f64()
            .map(|lp| {
                // avg_logprob is negative; map [-1.0, 0.0] → [0.0, 1.0]
                ((lp + 1.0).max(0.0)).min(1.0) as f32
            })
            .unwrap_or(0.80_f32);

        let language = result["language"].as_str().unwrap_or("en").to_string();

        info!("Whisper transcribed: '{}' (conf={:.2})", text, confidence);

        Ok(TranscriptionResult { text, confidence, language })
    }
}

#[async_trait]
impl CloudLLMClient for HuggingFaceClient {
    async fn generate(&self, prompt: &str) -> Result<String, AIError> {
        // Mistral-7B-Instruct-v0.3 — open-weight, not gated, available on HF free inference.
        // Falls back gracefully when unavailable (see NLUAgent / orchestrator for fallback logic).
        let result = self.hf_post_json(
            "https://router.huggingface.co/hf-inference/models/HuggingFaceH4/zephyr-7b-beta",
            json!({
                "inputs": prompt,
                "parameters": {
                    "max_new_tokens": 512,
                    "temperature": 0.3,
                    "return_full_text": false
                }
            }),
        ).await?;

        let text = result
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v["generated_text"].as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(text)
    }

    async fn extract_entities(&self, text: &str) -> Result<crate::models::transaction::ExtractedEntities, AIError> {
        let prompt = format!(
            r#"Extract entities from this sales transaction text: "{}"

Respond ONLY with valid JSON — no extra commentary:
{{
    "product": "product name or null",
    "quantity": number or null,
    "unit": "piece, kg, crate, litre, etc. or null",
    "price": number or null,
    "currency": "USD, EUR, GBP, etc. or null"
}}

Use null for any field not found in the text."#,
            text
        );

        // Use temperature near-zero for deterministic, structured JSON output
        let result = self.hf_post_json(
            "https://router.huggingface.co/hf-inference/models/HuggingFaceH4/zephyr-7b-beta",
            json!({
                "inputs": prompt,
                "parameters": {
                    "max_new_tokens": 300,
                    "temperature": 0.01,
                    "return_full_text": false
                }
            }),
        ).await?;

        let raw = result
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|v| v["generated_text"].as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        // Robustly extract the JSON block — the LLM may add preamble text
        let json_str = extract_json_block(&raw)
            .ok_or_else(|| AIError::Other(format!("No JSON found in LLM output: {}", raw)))?;

        let entities: crate::models::transaction::ExtractedEntities =
            serde_json::from_str(&json_str).map_err(AIError::Parse)?;

        Ok(entities)
    }
}

#[async_trait]
impl CloudTTSClient for HuggingFaceClient {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, AIError> {
        // MeloTTS English via HuggingFace Inference Providers API
        let response = self.client
            .post("https://router.huggingface.co/hf-inference/models/myshell-ai/MeloTTS-English")
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&json!({"inputs": text}))
            .send()
            .await
            .map_err(AIError::Network)?;

        let status = response.status();
        if status == reqwest::StatusCode::SERVICE_UNAVAILABLE {
            return Err(AIError::ServiceUnavailable);
        }
        if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(AIError::RateLimited);
        }
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            warn!("MeloTTS error {}: {}", status, body);
            return Err(AIError::ServiceUnavailable);
        }

        // Guard against the API returning a JSON error body with 200 status
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let bytes = response.bytes().await.map_err(AIError::Network)?;

        if content_type.contains("application/json") || content_type.contains("text/") {
            let body_str = String::from_utf8_lossy(&bytes);
            warn!("MeloTTS returned non-audio content-type '{}': {}", content_type, &body_str[..body_str.len().min(200)]);
            return Err(AIError::ServiceUnavailable);
        }

        Ok(bytes.to_vec())
    }
}

// ── Shared utility ────────────────────────────────────────────────────────

/// Extract the first complete JSON object or array from a string.
/// This is needed because LLMs often emit preamble/postamble text around JSON.
pub(crate) fn extract_json_block(text: &str) -> Option<String> {
    // Find the first '{' or '['
    let start = text.find(|c| c == '{' || c == '[')?;
    let open_char = text.chars().nth(start)?;
    let close_char = if open_char == '{' { '}' } else { ']' };

    let mut depth: i32 = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in text[start..].char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }
        match ch {
            '\\' if in_string => escape_next = true,
            '"' => in_string = !in_string,
            c if !in_string && c == open_char => depth += 1,
            c if !in_string && c == close_char => {
                depth -= 1;
                if depth == 0 {
                    return Some(text[start..start + i + 1].to_string());
                }
            }
            _ => {}
        }
    }
    None
}
