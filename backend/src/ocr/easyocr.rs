use reqwest::Client;
use serde_json::json;

use super::OCRResult;
use crate::config::AppConfig;

pub struct EasyOCRClient {
    client: Client,
    api_url: String,
    api_key: Option<String>,
}

impl EasyOCRClient {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            client: Client::new(),
            api_url: config.easyocr_url.clone().unwrap_or_else(|| 
                "https://api-inference.huggingface.co/models/jinhybr/OCR-DocTR".to_string()
            ),
            api_key: config.huggingface_api_token.clone(),
        }
    }

    /// Process receipt image and extract text
    pub async fn process_image(&self, image_bytes: Vec<u8>) -> anyhow::Result<Vec<OCRResult>> {
        // For local EasyOCR deployment
        let response = self.client
            .post(&self.api_url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key.as_deref().unwrap_or("")))
            .body(image_bytes)
            .send()
            .await?;

        let results: Vec<OCRResult> = response.json().await?;
        Ok(results)
    }

    /// Process receipt via HuggingFace Inference API (Free tier)
    pub async fn process_with_huggingface(&self, image_bytes: Vec<u8>) -> anyhow::Result<String> {
        let response = self.client
            .post("https://api-inference.huggingface.co/models/jinhybr/OCR-DocTR")
            .header("Authorization", format!("Bearer {}", self.api_key.as_deref().unwrap_or("")))
            .header("Content-Type", "application/json")
            .body(image_bytes)
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result["text"].as_str().unwrap_or("").to_string())
    }
}

/// Local EasyOCR service using Python bridge
pub struct LocalEasyOCR;

impl LocalEasyOCR {
    /// Extract text from image file
    pub fn extract_text(image_path: &str) -> anyhow::Result<Vec<String>> {
        // This would call a Python script using pyo3
        // For now, return mock data
        Ok(vec![
            "Store: Grocery Mart".to_string(),
            "Date: 2024-01-15".to_string(),
            "Eggs $5.00".to_string(),
            "Milk $3.50".to_string(),
            "Total: $8.50".to_string(),
        ])
    }
}
