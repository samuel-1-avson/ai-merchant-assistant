use std::sync::Arc;
use uuid::Uuid;

use crate::models::transaction::{ExtractedEntities, Transaction, CreateTransactionRequest};
use crate::db::repositories::transaction_repo::TransactionRepository;
use crate::db::repositories::product_repo::ProductRepository;

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

    pub async fn process_transaction(
        &self,
        entities: ExtractedEntities,
        user_id: Uuid,
    ) -> anyhow::Result<Transaction> {
        // Find or create product
        let product_id = if let Some(product_name) = &entities.product {
            let existing = self.product_repo.find_by_name(user_id, product_name).await?;
            existing.map(|p| p.id)
        } else {
            None
        };

        // Create transaction
        let request = CreateTransactionRequest {
            product_id,
            quantity: entities.quantity.map(|q| rust_decimal::Decimal::from_f64(q).unwrap_or_default())
                .unwrap_or_else(|| rust_decimal::Decimal::from(1)),
            unit: entities.unit,
            price: entities.price.map(|p| rust_decimal::Decimal::from_f64(p).unwrap_or_default())
                .unwrap_or_default(),
            notes: None,
        };

        let transaction = self.transaction_repo.create(user_id, request).await?;
        Ok(transaction)
    }
}
