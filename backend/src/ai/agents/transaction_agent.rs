use std::sync::Arc;
use uuid::Uuid;
use rust_decimal::prelude::FromPrimitive;
use tracing::{error, info, warn};

use crate::models::transaction::{ExtractedEntities, MultiProductEntities, ExtractedItem, Transaction, CreateTransactionRequest};
use crate::models::product::CreateProductRequest;
use crate::db::repositories::transaction_repo::TransactionRepository;
use crate::db::repositories::product_repo::{ProductRepository, ProductMatch};

/// Result of transaction processing with match information
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub transaction: Transaction,
    pub product_match: Option<ProductMatch>,
    pub is_new_product: bool,
    pub confidence: f64, // 0.0 - 1.0
}

/// Result of multi-product transaction processing
#[derive(Debug, Clone)]
pub struct MultiTransactionResult {
    pub transactions: Vec<Transaction>,
    pub items: Vec<SingleItemResult>,
    pub total_amount: rust_decimal::Decimal,
}

#[derive(Debug, Clone)]
pub struct SingleItemResult {
    pub extracted_item: ExtractedItem,
    pub product_match: Option<ProductMatch>,
    pub transaction: Option<Transaction>,
    pub is_new_product: bool,
    pub confidence: f64,
}

pub struct TransactionAgent {
    transaction_repo: Arc<TransactionRepository>,
    product_repo: Arc<ProductRepository>,
}

impl TransactionAgent {
    pub fn new(
        transaction_repo: Arc<TransactionRepository>,
        product_repo: Arc<ProductRepository>,
    ) -> Self {
        Self {
            transaction_repo,
            product_repo,
        }
    }

    /// Process a transaction from extracted entities (single product)
    pub async fn process_transaction(
        &self,
        entities: ExtractedEntities,
        user_id: Uuid,
    ) -> anyhow::Result<TransactionResult> {
        info!("Processing transaction for user {}: {:?}", user_id, entities);

        // Find product using fuzzy matching; auto-create if not found
        let (product_id, product_match, is_new_product) =
            if let Some(product_name) = &entities.product {
                match self.product_repo.find_best_match(user_id, product_name, Some(60)).await? {
                    Some(best_match) => {
                        info!(
                            "Found matching product '{}' with score {}",
                            best_match.product.name, best_match.score
                        );
                        (Some(best_match.product.id), Some(best_match), false)
                    }
                    None => {
                        // Auto-create product so it appears in the products catalogue
                        let unit = entities.unit.clone().unwrap_or_else(|| "piece".to_string());
                        let default_price = entities.price
                            .and_then(|p| rust_decimal::Decimal::from_f64(p));
                        match self.product_repo.create(user_id, CreateProductRequest {
                            name: product_name.clone(),
                            description: None,
                            sku: None,
                            default_price,
                            cost_price: None,
                            unit: Some(unit),
                            stock_quantity: None,
                            low_stock_threshold: None,
                        }).await {
                            Ok(product) => {
                                info!("Auto-created product '{}' from voice transaction", product_name);
                                (Some(product.id), None, true)
                            }
                            Err(e) => {
                                warn!("Failed to auto-create product '{}': {}", product_name, e);
                                (None, None, true)
                            }
                        }
                    }
                }
            } else {
                (None, None, false)
            };

        // Calculate confidence based on match score
        let confidence = product_match.as_ref()
            .map(|m| (m.score as f64 / 100.0).min(1.0))
            .unwrap_or(0.0);

        // Create transaction
        let request = CreateTransactionRequest {
            product_id,
            quantity: entities.quantity.map(|q| rust_decimal::Decimal::from_f64(q).unwrap_or_default())
                .unwrap_or_else(|| rust_decimal::Decimal::from(1i32)),
            unit: Some(entities.unit.clone().unwrap_or_else(|| "piece".to_string())),
            price: entities.price.map(|p| rust_decimal::Decimal::from_f64(p).unwrap_or_default())
                .unwrap_or_default(),
            notes: entities.product.clone().map(|name| {
                if is_new_product {
                    format!("New product from voice: {}", name)
                } else {
                    format!("Voice transaction: {}", name)
                }
            }),
        };

        let transaction = self.transaction_repo.create(user_id, request).await?;
        
        info!("Created transaction {} for user {}", transaction.id, user_id);

        Ok(TransactionResult {
            transaction,
            product_match,
            is_new_product,
            confidence,
        })
    }

    /// Process multiple products in a single transaction
    pub async fn process_multi_product_transaction(
        &self,
        entities: MultiProductEntities,
        user_id: Uuid,
    ) -> anyhow::Result<MultiTransactionResult> {
        info!("Processing multi-product transaction for user {}: {} items", user_id, entities.items.len());

        let mut results = Vec::new();
        let mut total_amount = rust_decimal::Decimal::ZERO;

        // Process each item
        for item in &entities.items {
            let result = self.process_single_item(item, user_id).await?;
            
            if let Some(ref tx) = result.transaction {
                total_amount += tx.total;
            }
            
            results.push(result);
        }

        // If total was mentioned, calculate adjustment ratio
        if let Some(mentioned_total) = entities.total_price {
            let mentioned_decimal = rust_decimal::Decimal::from_f64(mentioned_total)
                .unwrap_or_default();
            
            if total_amount > rust_decimal::Decimal::ZERO {
                let _ratio = mentioned_decimal / total_amount;
                // Could adjust individual prices here if needed
            }
        }

        let transactions: Vec<Transaction> = results.iter()
            .filter_map(|r| r.transaction.clone())
            .collect();

        info!(
            "Created {} transactions for multi-product sale, total: ${}",
            transactions.len(),
            total_amount
        );

        Ok(MultiTransactionResult {
            transactions,
            items: results,
            total_amount,
        })
    }

    /// Process a single item from multi-product extraction
    async fn process_single_item(
        &self,
        item: &ExtractedItem,
        user_id: Uuid,
    ) -> anyhow::Result<SingleItemResult> {
        // Find product using fuzzy matching; auto-create if not found
        let (product_id, product_match, is_new_product) =
            match self.product_repo.find_best_match(user_id, &item.product, Some(60)).await? {
                Some(best_match) => {
                    info!(
                        "Found matching product '{}' with score {}",
                        best_match.product.name, best_match.score
                    );
                    (Some(best_match.product.id), Some(best_match), false)
                }
                None => {
                    let unit = item.unit.clone().unwrap_or_else(|| "piece".to_string());
                    let default_price = item.price
                        .and_then(|p| rust_decimal::Decimal::from_f64(p));
                    match self.product_repo.create(user_id, CreateProductRequest {
                        name: item.product.clone(),
                        description: None,
                        sku: None,
                        default_price,
                        cost_price: None,
                        unit: Some(unit),
                        stock_quantity: None,
                        low_stock_threshold: None,
                    }).await {
                        Ok(product) => {
                            info!("Auto-created product '{}' from multi-item voice", item.product);
                            (Some(product.id), None, true)
                        }
                        Err(e) => {
                            warn!("Failed to auto-create product '{}': {}", item.product, e);
                            (None, None, true)
                        }
                    }
                }
            };

        // Calculate confidence
        let confidence = product_match.as_ref()
            .map(|m| (m.score as f64 / 100.0).min(1.0))
            .unwrap_or(0.0);

        // Calculate price
        let quantity = rust_decimal::Decimal::from_f64(item.quantity)
            .unwrap_or_else(|| rust_decimal::Decimal::from(1i32));
        
        let unit_price = item.price.map(|p| rust_decimal::Decimal::from_f64(p).unwrap_or_default())
            .unwrap_or_default();

        // Create transaction
        let request = CreateTransactionRequest {
            product_id,
            quantity,
            unit: Some(item.unit.clone().unwrap_or_else(|| "piece".to_string())),
            price: unit_price,
            notes: Some(format!("Multi-item transaction: {}", item.product)),
        };

        let transaction = match self.transaction_repo.create(user_id, request).await {
            Ok(tx) => {
                info!("Created transaction {} for item {}", tx.id, item.product);
                Some(tx)
            }
            Err(e) => {
                error!("Failed to create transaction for {}: {}", item.product, e);
                None
            }
        };

        Ok(SingleItemResult {
            extracted_item: item.clone(),
            product_match,
            transaction,
            is_new_product,
            confidence,
        })
    }

    /// Find similar products for a given query
    pub async fn find_similar_products(
        &self,
        user_id: Uuid,
        query: &str,
    ) -> anyhow::Result<Vec<ProductMatch>> {
        self.product_repo.find_by_name_fuzzy(user_id, query, Some(40), Some(3)).await
    }
}
