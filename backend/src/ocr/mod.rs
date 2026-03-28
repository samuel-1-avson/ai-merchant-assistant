pub mod easyocr;
pub mod receipt_parser;
pub mod service;

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;

pub use service::{OCRService, OCRConfig, OCRProcessingResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptItem {
    pub name: String,
    pub quantity: Decimal,
    pub unit: String,
    pub price: Decimal,
    pub total: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedReceipt {
    pub merchant_name: Option<String>,
    pub receipt_date: Option<String>,
    pub total_amount: Option<Decimal>,
    pub items: Vec<ReceiptItem>,
    pub raw_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OCRResult {
    pub text: String,
    pub confidence: f32,
    pub bbox: Vec<(i32, i32)>, // Bounding box coordinates
}
