//! OCR Service - High-level interface for receipt processing
//!
//! This module provides a complete OCR pipeline:
//! 1. Image preprocessing (resize, enhance)
//! 2. Text extraction (via HuggingFace or cloud OCR)
//! 3. Receipt parsing (structure extraction)
//! 4. Product matching (link to existing products)

use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;
use rust_decimal::prelude::FromPrimitive;

use crate::ai::clients::{CloudVisionClient, AIError};
use crate::db::repositories::product_repo::{ProductRepository, ProductMatch};

use super::{ParsedReceipt, ReceiptItem, OCRResult};
use super::receipt_parser::ReceiptParser;

/// OCR Service configuration
#[derive(Debug, Clone)]
pub struct OCRConfig {
    /// Minimum confidence threshold for OCR results (0.0 – 1.0)
    pub min_confidence: f32,
    /// Maximum image size in bytes (10 MB default)
    pub max_image_size: usize,
}

impl Default for OCRConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            max_image_size: 10 * 1024 * 1024,
        }
    }
}

/// Result of OCR processing
#[derive(Debug, Clone, serde::Deserialize)]
pub struct OCRProcessingResult {
    pub success: bool,
    pub receipt: ParsedReceipt,
    pub matched_products: Vec<(ReceiptItem, Option<ProductMatch>)>,
    pub errors: Vec<String>,
    pub processing_time_ms: u64,
    pub model_used: String,
}

/// Receipt OCR service backed by a vision-capable model (LLaVA-1.5-7B).
///
/// Pipeline:
///   1. Validate image size
///   2. `vision_client.analyze_receipt()` → structured JSON from image (one LLM call)
///   3. Parse JSON into `ParsedReceipt`; fall back to regex `ReceiptParser` on failure
///   4. Fuzzy-match every line item against the user's product inventory
pub struct OCRService {
    vision_client: Arc<dyn CloudVisionClient>,
    product_repo: Arc<ProductRepository>,
    config: OCRConfig,
}

impl OCRService {
    /// Create with the default configuration.
    pub fn new(
        vision_client: Arc<dyn CloudVisionClient>,
        product_repo: Arc<ProductRepository>,
    ) -> Self {
        Self { vision_client, product_repo, config: OCRConfig::default() }
    }

    /// Create with a custom configuration.
    pub fn with_config(
        vision_client: Arc<dyn CloudVisionClient>,
        product_repo: Arc<ProductRepository>,
        config: OCRConfig,
    ) -> Self {
        Self { vision_client, product_repo, config }
    }

    /// Process a receipt image end-to-end.
    pub async fn process_receipt(
        &self,
        user_id: Uuid,
        image_bytes: Vec<u8>,
    ) -> anyhow::Result<OCRProcessingResult> {
        let start_time = std::time::Instant::now();
        info!("Processing receipt for user {} ({} bytes)", user_id, image_bytes.len());

        // ── 1. Size guard ────────────────────────────────────────────────
        if image_bytes.len() > self.config.max_image_size {
            return Ok(OCRProcessingResult {
                success: false,
                receipt: empty_receipt(),
                matched_products: vec![],
                errors: vec!["Image too large (max 10 MB)".to_string()],
                processing_time_ms: 0,
                model_used: String::new(),
            });
        }

        // ── 2. Vision model: analyze receipt and return structured JSON ──
        let vision_result = match self.vision_client.analyze_receipt(&image_bytes).await {
            Ok(r) => r,
            Err(e) => {
                error!("Vision OCR failed: {}", e);
                return Ok(OCRProcessingResult {
                    success: false,
                    receipt: empty_receipt(),
                    matched_products: vec![],
                    errors: vec![format!("Vision OCR failed: {}", e)],
                    processing_time_ms: start_time.elapsed().as_millis() as u64,
                    model_used: String::new(),
                });
            }
        };

        info!(
            "Vision model ({}) returned {} chars",
            vision_result.model_used, vision_result.extracted_text.len()
        );

        // ── 3. Parse the JSON (with regex fallback) ──────────────────────
        let mut receipt = match parse_vision_json(&vision_result.extracted_text) {
            Ok(parsed) => {
                info!("Parsed {} receipt items from vision JSON", parsed.items.len());
                parsed
            }
            Err(e) => {
                warn!("Could not parse vision JSON ({}), falling back to regex parser", e);
                ReceiptParser::parse(&vision_result.extracted_text)
            }
        };

        // Preserve the raw text for debugging
        receipt.raw_text = vision_result.extracted_text.clone();

        // ── 4. Fuzzy-match items against known products ──────────────────
        let matched_products = self.match_items_to_products(user_id, &receipt.items).await?;

        let processing_time = start_time.elapsed().as_millis() as u64;
        info!(
            "Receipt done: {} items, {} product matches in {}ms",
            receipt.items.len(),
            matched_products.iter().filter(|(_, p)| p.is_some()).count(),
            processing_time
        );

        Ok(OCRProcessingResult {
            success: !receipt.items.is_empty(),
            receipt,
            matched_products,
            errors: vec![],
            processing_time_ms: processing_time,
            model_used: vision_result.model_used,
        })
    }

    /// Scan a raw product image and return the likely product name + category.
    pub async fn scan_product_image(
        &self,
        image_bytes: &[u8],
    ) -> anyhow::Result<ProductScanResult> {
        let vision_result = self.vision_client.extract_text_from_image(image_bytes).await
            .map_err(|e| anyhow::anyhow!("Vision scan failed: {}", e))?;

        Ok(ProductScanResult {
            product_name: if vision_result.extracted_text.is_empty() {
                None
            } else {
                Some(vision_result.extracted_text.clone())
            },
            confidence: vision_result.confidence as f64,
            model_used: vision_result.model_used,
            raw_text: vision_result.extracted_text,
        })
    }

    /// Fuzzy-match receipt items against the user's product catalogue.
    async fn match_items_to_products(
        &self,
        user_id: Uuid,
        items: &[ReceiptItem],
    ) -> anyhow::Result<Vec<(ReceiptItem, Option<ProductMatch>)>> {
        let mut results = Vec::new();
        for item in items {
            let matches = self.product_repo
                .find_by_name_fuzzy(user_id, &item.name, Some(50), Some(1))
                .await?;
            results.push((item.clone(), matches.into_iter().next()));
        }
        Ok(results)
    }

    /// Create confirmed receipt items as database transactions.
    pub async fn create_transactions_from_receipt(
        &self,
        user_id: Uuid,
        result: &OCRProcessingResult,
        confirmed_items: &[Uuid],
    ) -> anyhow::Result<Vec<crate::models::transaction::Transaction>> {
        use crate::db::repositories::transaction_repo::TransactionRepository;
        use crate::models::transaction::CreateTransactionRequest;

        let transaction_repo = TransactionRepository::new(self.product_repo.pool().clone());
        let mut created_transactions = Vec::new();

        for (item, product_match) in &result.matched_products {
            if let Some(product) = product_match {
                if !confirmed_items.contains(&product.product.id) {
                    continue;
                }
            } else {
                continue;
            }

            let request = CreateTransactionRequest {
                product_id: product_match.as_ref().map(|p| p.product.id),
                quantity: item.quantity,
                unit: Some(item.unit.clone()),
                price: item.price,
                notes: Some(format!("From receipt ({}): {}", result.model_used, item.name)),
            };

            match transaction_repo.create(user_id, request).await {
                Ok(tx) => {
                    info!("Created transaction {} for receipt item '{}'", tx.id, item.name);
                    created_transactions.push(tx);
                }
                Err(e) => {
                    error!("Failed to create transaction for '{}': {}", item.name, e);
                }
            }
        }

        Ok(created_transactions)
    }
}

// ── Product scan result ───────────────────────────────────────────────────

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProductScanResult {
    pub product_name: Option<String>,
    pub confidence: f64,
    pub model_used: String,
    pub raw_text: String,
}

// ── Private helpers ───────────────────────────────────────────────────────

fn empty_receipt() -> ParsedReceipt {
    ParsedReceipt {
        merchant_name: None,
        receipt_date: None,
        total_amount: None,
        items: vec![],
        raw_text: String::new(),
    }
}

/// Parse the structured JSON that the vision model returns for a receipt.
fn parse_vision_json(json_text: &str) -> anyhow::Result<ParsedReceipt> {
    let parsed: serde_json::Value = serde_json::from_str(json_text)
        .map_err(|e| anyhow::anyhow!("Invalid JSON from vision model: {}", e))?;

    let items: Vec<ReceiptItem> = parsed["items"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|item| {
            Some(ReceiptItem {
                name: item["name"].as_str()?.to_string(),
                quantity: item["quantity"]
                    .as_f64()
                    .and_then(rust_decimal::Decimal::from_f64)
                    .unwrap_or_else(|| rust_decimal::Decimal::from(1i32)),
                unit: item["unit"].as_str().unwrap_or("piece").to_string(),
                price: item["price"]
                    .as_f64()
                    .and_then(rust_decimal::Decimal::from_f64)
                    .unwrap_or_default(),
                total: item["total"]
                    .as_f64()
                    .and_then(rust_decimal::Decimal::from_f64)
                    .unwrap_or_default(),
            })
        })
        .collect();

    Ok(ParsedReceipt {
        merchant_name: parsed["merchant_name"].as_str().map(|s| s.to_string()),
        receipt_date: parsed["receipt_date"].as_str().map(|s| s.to_string()),
        total_amount: parsed["total_amount"]
            .as_f64()
            .and_then(rust_decimal::Decimal::from_f64),
        items,
        raw_text: json_text.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ocr_config_default() {
        let config = OCRConfig::default();
        assert_eq!(config.min_confidence, 0.6);
        assert_eq!(config.max_image_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_parse_vision_json_valid() {
        let json = r#"{
            "merchant_name": "Best Mart",
            "receipt_date": "2026-03-27",
            "total_amount": 25.50,
            "items": [
                {"name": "Apples", "quantity": 3.0, "unit": "kg", "price": 5.0, "total": 15.0},
                {"name": "Milk",   "quantity": 2.0, "unit": "litre", "price": 5.25, "total": 10.50}
            ]
        }"#;
        let receipt = parse_vision_json(json).unwrap();
        assert_eq!(receipt.merchant_name.as_deref(), Some("Best Mart"));
        assert_eq!(receipt.items.len(), 2);
        assert_eq!(receipt.items[0].name, "Apples");
    }
}
