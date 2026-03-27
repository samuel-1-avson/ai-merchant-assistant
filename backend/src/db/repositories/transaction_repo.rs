use sqlx::PgPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::transaction::{Transaction, CreateTransactionRequest};
use rust_decimal::Decimal;

pub struct TransactionRepository {
    pool: PgPool,
}

impl TransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        request: CreateTransactionRequest,
    ) -> anyhow::Result<Transaction> {
        let total = request.quantity * request.price;
        let unit = request.unit.unwrap_or_else(|| "piece".to_string());

        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions (id, user_id, product_id, quantity, unit, price, total, notes, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(request.product_id)
        .bind(request.quantity)
        .bind(unit)
        .bind(request.price)
        .bind(total)
        .bind(request.notes)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(transaction)
    }

    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<Transaction>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions 
            WHERE user_id = $1 
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(transactions)
    }

    pub async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> anyhow::Result<Option<Transaction>> {
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions 
            WHERE id = $1 AND user_id = $2
            "#
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(transaction)
    }

    pub async fn get_by_date_range(
        &self,
        user_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<Transaction>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions 
            WHERE user_id = $1 
            AND created_at BETWEEN $2 AND $3
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        Ok(transactions)
    }
}
