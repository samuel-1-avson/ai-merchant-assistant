use std::sync::Arc;
use tracing::warn;

use crate::ai::clients::CloudLLMClient;
use crate::models::transaction::{ExtractedEntities, MultiProductEntities, ExtractedItem};

pub struct NLUAgent {
    client: Arc<dyn CloudLLMClient>,
}

impl NLUAgent {
    pub fn new(client: Arc<dyn CloudLLMClient>) -> Self {
        Self { client }
    }

    /// Extract single product entities.
    ///
    /// Tries the LLM first; if the LLM is unavailable falls back to
    /// the built-in rule-based extractor so the pipeline never hard-fails.
    pub async fn extract_entities(&self, text: &str) -> anyhow::Result<ExtractedEntities> {
        match self.client.extract_entities(text).await {
            Ok(entities) => Ok(entities),
            Err(e) => {
                warn!("LLM entity extraction failed ({}), using rule-based fallback", e);
                Ok(Self::rule_based_extract(text))
            }
        }
    }

    /// Classify the user's intent.
    ///
    /// Falls back to keyword matching when the LLM is unavailable.
    pub async fn classify_intent(&self, text: &str) -> anyhow::Result<String> {
        let prompt = format!(
            r#"Classify the intent of this text: "{}"

Possible intents: record_sale, query_analytics, update_inventory, set_alert, general_conversation

Respond with only the intent name."#,
            text
        );

        match self.client.generate(&prompt).await {
            Ok(response) => Ok(response.trim().to_lowercase()),
            Err(e) => {
                warn!("LLM intent classification failed ({}), using keyword fallback", e);
                Ok(Self::keyword_classify_intent(text))
            }
        }
    }

    /// Extract multiple products from transaction text.
    ///
    /// Falls back to splitting on conjunctions and calling rule_based_extract
    /// per segment when the LLM is unavailable.
    pub async fn extract_multi_product_entities(&self, text: &str) -> anyhow::Result<MultiProductEntities> {
        let prompt = format!(
            r#"Extract all products and quantities from this sales transaction text.

Text: "{}"

Respond ONLY with valid JSON in this exact format:
{{
    "items": [
        {{
            "product": "product name",
            "quantity": number,
            "unit": "unit (piece, kg, liter, etc.) or null",
            "price": price per unit or null
        }}
    ],
    "total_price": total amount mentioned or null,
    "currency": "USD, EUR, etc. or null",
    "transaction_date": "date mentioned or null",
    "notes": "any additional notes or null"
}}

Extract EVERY product mentioned. If total is given, calculate individual prices if possible."#,
            text
        );

        match self.client.generate(&prompt).await {
            Ok(response) => {
                if let Ok(entities) = serde_json::from_str::<MultiProductEntities>(&response) {
                    return Ok(entities);
                }
                warn!("LLM multi-product parse failed, using rule-based fallback");
                Ok(Self::rule_based_extract_multi(text))
            }
            Err(e) => {
                warn!("LLM multi-product extraction failed ({}), using rule-based fallback", e);
                Ok(Self::rule_based_extract_multi(text))
            }
        }
    }

    /// Send a free-form prompt to the LLM and return the raw response.
    ///
    /// Used by the assistant chat endpoint. Does NOT fall back — callers must
    /// handle the error and provide their own fallback.
    pub async fn generate_text(&self, prompt: &str) -> Result<String, crate::ai::clients::AIError> {
        self.client.generate(prompt).await
    }

    /// Check if text contains multiple products
    pub fn contains_multiple_products(text: &str) -> bool {
        let indicators = [" and ", ",", " plus ", " with ", "&"];
        let lower = text.to_lowercase();

        let count = indicators.iter()
            .filter(|&&indicator| lower.contains(indicator))
            .count();

        let quantity_pattern = regex::Regex::new(r"\d+\s*(?:x|×|pieces?|units?|pcs?)").unwrap();
        let quantity_matches: Vec<_> = quantity_pattern.find_iter(&lower).collect();

        count > 0 || quantity_matches.len() > 1
    }

    // ── Rule-based fallbacks ───────────────────────────────────────────

    /// Keyword-based intent classifier — no LLM required.
    fn keyword_classify_intent(text: &str) -> String {
        let lower = text.to_lowercase();

        // Sale indicators (most common for a merchant assistant)
        if lower.contains("sold") || lower.contains("sale") || lower.contains("sell")
            || lower.contains("bought") || lower.contains("purchased")
            || lower.contains("customer") || lower.contains("transaction")
            || lower.contains("paid") || lower.contains("charge")
        {
            return "record_sale".to_string();
        }

        // Analytics / reporting
        if lower.contains("report") || lower.contains("revenue") || lower.contains("profit")
            || lower.contains("analytics") || lower.contains("summary")
            || lower.contains("how much") || lower.contains("total sales")
            || lower.contains("how many") || lower.contains("trend")
        {
            return "query_analytics".to_string();
        }

        // Inventory
        if lower.contains("stock") || lower.contains("inventory")
            || lower.contains("restock") || lower.contains("quantity")
        {
            return "update_inventory".to_string();
        }

        // Alert
        if lower.contains("alert") || lower.contains("notify") || lower.contains("remind") {
            return "set_alert".to_string();
        }

        // Default for unknown speech in a merchant context — assume sale
        "record_sale".to_string()
    }

    /// Regex/rule-based single-product entity extractor.
    ///
    /// Handles common merchant transaction patterns:
    /// • "sold 3 kg rice at $10"
    /// • "5 apples for 2 dollars each"
    /// • "customer bought 2 bottles of water 1.50 each"
    /// • "shirt $45"
    pub fn rule_based_extract(text: &str) -> ExtractedEntities {
        let lower = text.to_lowercase();

        // ── Extract price ────────────────────────────────────────────
        // Matches: $45, £2.50, 10 dollars, 1,234.56 USD, etc.
        let price_re = regex::Regex::new(
            r"(?i)(?:[$£€])\s*(\d{1,6}(?:,\d{3})*(?:\.\d{1,2})?)|(\d{1,6}(?:,\d{3})*(?:\.\d{1,2})?)\s*(?:dollars?|usd|gbp|eur(?:os?)?|pounds?)\b"
        ).unwrap();

        let price: Option<f64> = price_re.captures_iter(&lower)
            .find_map(|cap| {
                let s = cap.get(1).or(cap.get(2))?.as_str().replace(',', "");
                s.parse().ok()
            });

        // ── Extract quantity + unit ───────────────────────────────────
        // e.g. "3 kg", "5 pieces", "2 dozen", bare "10"
        let qty_re = regex::Regex::new(
            r"(?i)(\d+(?:\.\d+)?)\s*(kg|kgs|kilogram|kilograms|g|gram|grams|liter|litre|litres|liters|ml|milliliter|piece|pieces|pcs|pc|dozen|crate|crates|bottle|bottles|bag|bags|box|boxes|unit|units|pack|packs|pair|pairs|metre|meter|metres|meters|yard|yards|pound|pounds|lbs?|oz|ounce|ounces|bunch|bunches|roll|rolls|tin|tins|can|cans|sachet|sachets|cup|cups|tablet|tablets|capsule|capsules)?"
        ).unwrap();

        let (quantity, unit) = qty_re.captures_iter(&lower)
            .filter_map(|cap| {
                let qty: f64 = cap.get(1)?.as_str().parse().ok()?;
                // Skip pure year/date-looking numbers
                if qty >= 1900.0 && qty <= 2100.0 { return None; }
                // Skip numbers that look like the price we already captured
                if let Some(p) = price {
                    if (qty - p).abs() < 0.01 { return None; }
                }
                let unit = cap.get(2).map(|m| m.as_str().to_string());
                Some((qty, unit))
            })
            .next()
            .unwrap_or((1.0, None));

        // ── Extract product name ──────────────────────────────────────
        // Remove sale verbs, quantity phrases, price phrases, and filler words
        let strip_words: &[&str] = &[
            "sold", "sell", "buy", "bought", "purchased", "customer", "transaction",
            "sale", "at", "for", "the", "a", "an", "of", "and", "with", "per", "each",
            "dollar", "dollars", "usd", "gbp", "eur", "euro", "euros", "pound", "pounds",
            "apiece", "total", "price", "worth",
        ];

        // Also strip the matched quantity/unit string and price string from text
        let price_stripped = price_re.replace_all(&lower, " ");
        let qty_stripped = qty_re.replace_all(&price_stripped, " ");

        let product: String = qty_stripped
            .split_whitespace()
            .filter(|w| {
                let w_clean = w.trim_matches(|c: char| !c.is_alphabetic());
                !w_clean.is_empty()
                    && !strip_words.contains(&w_clean)
                    && w_clean.len() > 1
            })
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string();

        let product = if product.is_empty() {
            // Last resort: take the longest word that isn't a stop word
            lower.split_whitespace()
                .filter(|w| {
                    let w_clean = w.trim_matches(|c: char| !c.is_alphabetic());
                    !strip_words.contains(&w_clean) && w_clean.len() > 2
                })
                .max_by_key(|w| w.len())
                .unwrap_or("item")
                .to_string()
        } else {
            product
        };

        let currency = if lower.contains('£') || lower.contains("gbp") || lower.contains("pound") {
            Some("GBP".to_string())
        } else if lower.contains('€') || lower.contains("eur") {
            Some("EUR".to_string())
        } else {
            Some("USD".to_string())
        };

        ExtractedEntities {
            product: Some(product),
            quantity: Some(quantity),
            unit,
            price,
            currency,
        }
    }

    /// Rule-based multi-product extractor.
    ///
    /// Splits on conjunctions/commas and applies `rule_based_extract` to each segment.
    fn rule_based_extract_multi(text: &str) -> MultiProductEntities {
        // Split on "and", "," , "&" to get individual items
        let segment_re = regex::Regex::new(r"(?i)\s*(?:,|and|&|plus)\s*").unwrap();
        let segments: Vec<&str> = segment_re.split(text).collect();

        let items: Vec<ExtractedItem> = segments.iter()
            .filter(|s| !s.trim().is_empty())
            .map(|segment| {
                let e = Self::rule_based_extract(segment);
                ExtractedItem {
                    product: e.product.unwrap_or_else(|| "item".to_string()),
                    quantity: e.quantity.unwrap_or(1.0),
                    unit: e.unit,
                    price: e.price,
                }
            })
            .collect();

        // Sum total from items that have prices
        let total_price: Option<f64> = {
            let sum: f64 = items.iter()
                .filter_map(|i| i.price.map(|p| p * i.quantity))
                .sum();
            if sum > 0.0 { Some(sum) } else { None }
        };

        MultiProductEntities {
            items,
            total_price,
            currency: Some("USD".to_string()),
            transaction_date: None,
            notes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_multiple_products() {
        assert!(NLUAgent::contains_multiple_products("sold 3 apples and 2 bananas"));
        assert!(NLUAgent::contains_multiple_products("bought milk, eggs, and bread"));
        assert!(NLUAgent::contains_multiple_products("5x apples plus 3x oranges"));
        assert!(!NLUAgent::contains_multiple_products("sold 5 apples"));
    }

    #[test]
    fn test_rule_based_extract_with_price() {
        let e = NLUAgent::rule_based_extract("sold 3 kg rice at $10");
        assert_eq!(e.product.as_deref(), Some("rice"));
        assert_eq!(e.quantity, Some(3.0));
        assert_eq!(e.unit.as_deref(), Some("kg"));
        assert_eq!(e.price, Some(10.0));
    }

    #[test]
    fn test_rule_based_extract_no_unit() {
        let e = NLUAgent::rule_based_extract("customer bought 5 apples for 2 dollars each");
        assert_eq!(e.product.as_deref(), Some("apples"));
        assert_eq!(e.quantity, Some(5.0));
        assert_eq!(e.price, Some(2.0));
    }

    #[test]
    fn test_rule_based_extract_product_only() {
        let e = NLUAgent::rule_based_extract("sold tomatoes");
        assert_eq!(e.product.as_deref(), Some("tomatoes"));
        assert_eq!(e.quantity, Some(1.0));
    }

    #[test]
    fn test_keyword_intent_sale() {
        assert_eq!(NLUAgent::keyword_classify_intent("sold 3 bottles"), "record_sale");
        assert_eq!(NLUAgent::keyword_classify_intent("customer bought rice"), "record_sale");
    }

    #[test]
    fn test_keyword_intent_analytics() {
        assert_eq!(NLUAgent::keyword_classify_intent("show me my revenue report"), "query_analytics");
        assert_eq!(NLUAgent::keyword_classify_intent("how much did I earn today"), "query_analytics");
    }
}
