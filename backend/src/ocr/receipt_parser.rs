use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;

use super::{ParsedReceipt, ReceiptItem};

pub struct ReceiptParser;

impl ReceiptParser {
    /// Parse raw OCR text into structured receipt data
    pub fn parse(raw_text: &str) -> ParsedReceipt {
        let mut receipt = ParsedReceipt {
            merchant_name: None,
            receipt_date: None,
            total_amount: None,
            items: Vec::new(),
            raw_text: raw_text.to_string(),
        };

        // Extract merchant name (usually at the top)
        receipt.merchant_name = Self::extract_merchant(raw_text);
        
        // Extract date
        receipt.receipt_date = Self::extract_date(raw_text);
        
        // Extract total amount
        receipt.total_amount = Self::extract_total(raw_text);
        
        // Extract items
        receipt.items = Self::extract_items(raw_text);

        receipt
    }

    /// Extract merchant/store name
    fn extract_merchant(text: &str) -> Option<String> {
        // Look for patterns like "Store Name", "Merchant", etc.
        let lines: Vec<&str> = text.lines().collect();
        
        // Usually merchant name is in the first few lines
        for line in lines.iter().take(5) {
            let trimmed = line.trim();
            if !trimmed.is_empty() && trimmed.len() > 2 {
                // Filter out common receipt headers
                if !trimmed.to_lowercase().contains("receipt") 
                    && !trimmed.to_lowercase().contains("invoice")
                    && !trimmed.to_lowercase().contains("tel:")
                    && !trimmed.to_lowercase().contains("phone") {
                    return Some(trimmed.to_string());
                }
            }
        }
        None
    }

    /// Extract date from receipt
    fn extract_date(text: &str) -> Option<String> {
        // Common date patterns
        let patterns = [
            r"(\d{1,2}[/-]\d{1,2}[/-]\d{2,4})",  // MM/DD/YYYY or DD/MM/YYYY
            r"(\d{4}[/-]\d{1,2}[/-]\d{1,2})",  // YYYY/MM/DD
            r"(\d{1,2}\s+(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*\s+\d{2,4})", // 15 Jan 2024
        ];

        for pattern in &patterns {
            let re = Regex::new(pattern).ok()?;
            if let Some(caps) = re.captures(text) {
                return Some(caps.get(1)?.as_str().to_string());
            }
        }
        None
    }

    /// Extract total amount
    fn extract_total(text: &str) -> Option<Decimal> {
        // Look for total patterns
        let patterns = [
            r"[Tt]otal[:\s]*[$]?\s*(\d+\.\d{2})",
            r"[Aa]mount\s+[Dd]ue[:\s]*[$]?\s*(\d+\.\d{2})",
            r"[Bb]alance[:\s]*[$]?\s*(\d+\.\d{2})",
            r"[Tt]otal\s+[Aa]mount[:\s]*[$]?\s*(\d+\.\d{2})",
        ];

        for pattern in &patterns {
            let re = Regex::new(pattern).ok()?;
            if let Some(caps) = re.captures(text) {
                let amount_str = caps.get(1)?.as_str();
                return Decimal::from_str(amount_str).ok();
            }
        }
        None
    }

    /// Extract line items
    fn extract_items(text: &str) -> Vec<ReceiptItem> {
        let mut items = Vec::new();
        
        // Pattern: Item name followed by price
        // Examples: "Eggs 5.00", "Milk $3.50", "2x Bread 4.00"
        let item_pattern = Regex::new(
            r"(?i)^\s*(\d*)\s*([a-zA-Z\s]+?)\s+[$]?(\d+\.\d{2})\s*$"
        ).unwrap();

        for line in text.lines() {
            if let Some(caps) = item_pattern.captures(line) {
                let qty_str = caps.get(1).map(|m| m.as_str()).unwrap_or("1");
                let name = caps.get(2).map(|m| m.as_str().trim()).unwrap_or("Unknown");
                let price_str = caps.get(3).map(|m| m.as_str()).unwrap_or("0.00");

                // Skip if it looks like a total line
                if name.to_lowercase().contains("total") 
                    || name.to_lowercase().contains("subtotal")
                    || name.to_lowercase().contains("tax") {
                    continue;
                }

                let quantity = Decimal::from_str(qty_str).unwrap_or_else(|_| Decimal::from(1));
                let price = Decimal::from_str(price_str).unwrap_or_else(|_| Decimal::ZERO);
                let total = quantity * price;

                items.push(ReceiptItem {
                    name: name.to_string(),
                    quantity,
                    unit: "piece".to_string(),
                    price,
                    total,
                });
            }
        }

        items
    }

    /// Smart item matching - match receipt items to known products
    pub fn match_items_to_products(
        items: &[ReceiptItem],
        known_products: &[crate::models::product::Product],
    ) -> Vec<(ReceiptItem, Option<uuid::Uuid>)> {
        items
            .iter()
            .map(|item| {
                // Find best matching product
                let matched = known_products.iter().find(|p| {
                    let item_name_lower = item.name.to_lowercase();
                    let product_name_lower = p.name.to_lowercase();
                    
                    // Exact match
                    item_name_lower == product_name_lower
                    // Contains match
                    || item_name_lower.contains(&product_name_lower)
                    || product_name_lower.contains(&item_name_lower)
                    // Word match
                    || item_name_lower.split_whitespace().any(|word| {
                        product_name_lower.split_whitespace().any(|pword| word == pword)
                    })
                });

                (item.clone(), matched.map(|p| p.id))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_receipt() {
        let text = r#"Grocery Store
123 Main St
Date: 01/15/2024

Eggs $5.00
Milk $3.50
Bread $2.00

Total: $10.50"#;

        let receipt = ReceiptParser::parse(text);
        
        assert_eq!(receipt.merchant_name, Some("Grocery Store".to_string()));
        assert_eq!(receipt.receipt_date, Some("01/15/2024".to_string()));
        assert_eq!(receipt.total_amount, Some(Decimal::from_str("10.50").unwrap()));
        assert_eq!(receipt.items.len(), 3);
    }

    #[test]
    fn test_extract_items() {
        let text = r#"Eggs $5.00
Milk $3.50
2x Bread $4.00
Total $10.50"#;

        let items = ReceiptParser::extract_items(text);
        
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].name, "Eggs");
        assert_eq!(items[0].price, Decimal::from_str("5.00").unwrap());
        assert_eq!(items[1].name, "Milk");
    }
}
