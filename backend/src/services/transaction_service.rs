use std::sync::Arc;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::db::repositories::transaction_repo::TransactionRepository;
use crate::models::transaction::{Transaction, CreateTransactionRequest};

pub struct TransactionService {
    repo: Arc<TransactionRepository>,
}

impl TransactionService {
    pub fn new(repo: Arc<TransactionRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_transaction(
        &self,
        user_id: Uuid,
        request: CreateTransactionRequest,
    ) -> anyhow::Result<Transaction> {
        self.repo.create(user_id, request).await
    }

    pub async fn list_transactions(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<Transaction>> {
        self.repo.list_by_user(user_id, limit, offset).await
    }

    pub async fn get_by_date_range(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<Transaction>> {
        self.repo.get_by_date_range(user_id, start, end).await
    }
}
