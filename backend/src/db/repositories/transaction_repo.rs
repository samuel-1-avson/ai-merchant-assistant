//! Transaction Repository

use sqlx::{PgPool, Row};

// Row trait is already imported above
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

use crate::models::transaction::{Transaction, CreateTransactionRequest};
// Simple UserActivity struct for analytics
#[derive(Debug, Clone)]
pub struct UserActivity {
    pub date: chrono::NaiveDate,
    pub transaction_count: i64,
    pub total_amount: rust_decimal::Decimal,
}

pub struct TransactionRepository {
    pool: PgPool,
}

impl TransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get transaction by ID (runtime query)
    pub async fn get_by_id(&self, id: Uuid, user_id: Uuid) -> anyhow::Result<Option<Transaction>> {
        let row = sqlx::query(
            r#"
            SELECT t.id, t.user_id, t.product_id,
                   t.quantity::float8 as quantity, t.unit,
                   t.price::float8 as price, t.total::float8 as total,
                   t.notes, t.voice_recording_url, t.created_at, t.updated_at,
                   p.name AS product_name
            FROM transactions t
            LEFT JOIN products p ON t.product_id = p.id
            WHERE t.id = $1 AND t.user_id = $2
            "#
        )
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_transaction(&row)?)),
            None => Ok(None),
        }
    }

    /// List transactions with pagination and filters (runtime query)
    pub async fn list(
        &self,
        user_id: Uuid,
        product_id: Option<Uuid>,
        start_date: Option<chrono::DateTime<chrono::Utc>>,
        end_date: Option<chrono::DateTime<chrono::Utc>>,
        limit: i64,
        offset: i64,
    ) -> anyhow::Result<Vec<Transaction>> {
        let mut query = String::from(
            "SELECT t.id, t.user_id, t.product_id, \
             t.quantity::float8 as quantity, t.unit, \
             t.price::float8 as price, t.total::float8 as total, \
             t.notes, t.voice_recording_url, t.created_at, t.updated_at, p.name AS product_name \
             FROM transactions t LEFT JOIN products p ON t.product_id = p.id \
             WHERE t.user_id = $1"
        );
        let mut param_idx = 2;

        if product_id.is_some() {
            query.push_str(&format!(" AND t.product_id = ${}", param_idx));
            param_idx += 1;
        }
        if start_date.is_some() {
            query.push_str(&format!(" AND t.created_at >= ${}", param_idx));
            param_idx += 1;
        }
        if end_date.is_some() {
            query.push_str(&format!(" AND t.created_at <= ${}", param_idx));
            param_idx += 1;
        }
        query.push_str(&format!(" ORDER BY t.created_at DESC LIMIT ${} OFFSET ${}", param_idx, param_idx + 1));

        let mut q = sqlx::query(&query).bind(user_id);
        if let Some(pid) = product_id {
            q = q.bind(pid);
        }
        if let Some(sd) = start_date {
            q = q.bind(sd);
        }
        if let Some(ed) = end_date {
            q = q.bind(ed);
        }
        q = q.bind(limit).bind(offset);

        let rows = q.fetch_all(&self.pool).await?;
        rows.into_iter().map(|r| self.row_to_transaction(&r)).collect()
    }

    /// Create transaction (runtime query)
    pub async fn create(&self, user_id: Uuid, tx: CreateTransactionRequest) -> anyhow::Result<Transaction> {
        let id = Uuid::new_v4();
        let total = tx.quantity * tx.price;

        let row = sqlx::query(
            r#"
            INSERT INTO transactions (id, user_id, product_id, quantity, unit, price, total, notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, product_id,
                      quantity::float8 as quantity, unit,
                      price::float8 as price, total::float8 as total,
                      notes, voice_recording_url, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(tx.product_id)
        .bind(tx.quantity.to_f64().unwrap_or(0.0))
        .bind(&tx.unit)
        .bind(tx.price.to_f64().unwrap_or(0.0))
        .bind(total.to_f64().unwrap_or(0.0))
        .bind(&tx.notes)
        .fetch_one(&self.pool)
        .await?;

        self.row_to_transaction(&row)
    }

    /// Update transaction (runtime query)
    pub async fn update(&self, id: Uuid, user_id: Uuid, tx: CreateTransactionRequest) -> anyhow::Result<Option<Transaction>> {
        let total = tx.quantity * tx.price;

        let row = sqlx::query(
            r#"
            UPDATE transactions
            SET product_id = $1, quantity = $2, unit = $3, price = $4,
                total = $5, notes = $6, updated_at = NOW()
            WHERE id = $7 AND user_id = $8
            RETURNING id, user_id, product_id,
                      quantity::float8 as quantity, unit,
                      price::float8 as price, total::float8 as total,
                      notes, voice_recording_url, created_at, updated_at
            "#
        )
        .bind(tx.product_id)
        .bind(tx.quantity.to_f64().unwrap_or(0.0))
        .bind(tx.unit.as_deref().unwrap_or("piece"))
        .bind(tx.price.to_f64().unwrap_or(0.0))
        .bind(total.to_f64().unwrap_or(0.0))
        .bind(&tx.notes)
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(self.row_to_transaction(&r)?)),
            None => Ok(None),
        }
    }

    /// Update only the price (and recalculate total) for a transaction
    pub async fn update_price(&self, id: Uuid, user_id: Uuid, price: Decimal) -> anyhow::Result<Option<Transaction>> {
        let row = sqlx::query(
            r#"
            UPDATE transactions
            SET price = $1, total = quantity * $1, updated_at = NOW()
            WHERE id = $2 AND user_id = $3
            RETURNING id, user_id, product_id,
                      quantity::float8 as quantity, unit,
                      price::float8 as price, total::float8 as total,
                      notes, voice_recording_url, created_at, updated_at
            "#
        )
        .bind(price.to_f64().unwrap_or(0.0))
        .bind(id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(self.row_to_transaction(&r)?)),
            None => Ok(None),
        }
    }

    /// Delete transaction (runtime query)
    pub async fn delete(&self, id: Uuid, user_id: Uuid) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM transactions WHERE id = $1 AND user_id = $2")
            .bind(id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Get user activity (runtime query)
    pub async fn get_user_activity(&self, user_id: Uuid, days: i64) -> anyhow::Result<Vec<UserActivity>> {
        let start_date = chrono::Utc::now() - chrono::Duration::days(days);

        let rows = sqlx::query(
            r#"
            SELECT
                DATE(created_at) as date,
                COUNT(*) as transaction_count,
                SUM(total)::float8 as total_amount
            FROM transactions
            WHERE user_id = $1 AND created_at >= $2
            GROUP BY DATE(created_at)
            ORDER BY date DESC
            "#
        )
        .bind(user_id)
        .bind(start_date)
        .fetch_all(&self.pool)
        .await?;

        let activity = rows.into_iter().map(|row| {
            UserActivity {
                date: row.try_get("date").unwrap_or_default(),
                transaction_count: row.try_get("transaction_count").unwrap_or(0),
                total_amount: Decimal::from_f64(row.try_get::<f64, _>("total_amount").unwrap_or(0.0)).unwrap_or_default(),
            }
        }).collect();

        Ok(activity)
    }

    /// List transactions by user with pagination
    pub async fn list_by_user(&self, user_id: Uuid, limit: i64, offset: i64) -> anyhow::Result<Vec<Transaction>> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.user_id, t.product_id,
                   t.quantity::float8 as quantity, t.unit,
                   t.price::float8 as price, t.total::float8 as total,
                   t.notes, t.voice_recording_url, t.created_at, t.updated_at,
                   p.name AS product_name
            FROM transactions t
            LEFT JOIN products p ON t.product_id = p.id
            WHERE t.user_id = $1
            ORDER BY t.created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| self.row_to_transaction(&r)).collect()
    }

    /// Get transactions by date range
    pub async fn get_by_date_range(
        &self,
        user_id: Uuid,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<Vec<Transaction>> {
        let rows = sqlx::query(
            r#"
            SELECT t.id, t.user_id, t.product_id,
                   t.quantity::float8 as quantity, t.unit,
                   t.price::float8 as price, t.total::float8 as total,
                   t.notes, t.voice_recording_url, t.created_at, t.updated_at,
                   p.name AS product_name
            FROM transactions t
            LEFT JOIN products p ON t.product_id = p.id
            WHERE t.user_id = $1 AND t.created_at >= $2 AND t.created_at <= $3
            ORDER BY t.created_at DESC
            "#
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| self.row_to_transaction(&r)).collect()
    }

    /// Helper to convert row to Transaction.
    ///
    /// `product_name` is populated from the products JOIN when available.
    /// For voice transactions without a catalog link, it is extracted from
    /// the notes field (e.g. "New product from voice: shirts").
    fn row_to_transaction(&self, row: &sqlx::postgres::PgRow) -> anyhow::Result<Transaction> {
        use sqlx::Row;
        let notes: Option<String> = row.try_get("notes").ok();

        // Try the joined products.name column first; fall back to notes-based extraction.
        let product_name: Option<String> = row
            .try_get::<Option<String>, _>("product_name")
            .ok()
            .flatten()
            .or_else(|| {
                notes.as_ref().and_then(|n| {
                    for prefix in &[
                        "New product from voice: ",
                        "Voice transaction: ",
                        "Multi-item transaction: ",
                    ] {
                        if let Some(name) = n.strip_prefix(prefix) {
                            return Some(name.to_string());
                        }
                    }
                    None
                })
            });

        Ok(Transaction {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            product_id: row.try_get("product_id")?,
            product_name,
            quantity: Decimal::from_f64(row.try_get::<f64, _>("quantity").unwrap_or(0.0)).unwrap_or_default(),
            unit: row.try_get::<Option<String>, _>("unit").ok().flatten().unwrap_or_else(|| "piece".to_string()),
            price: Decimal::from_f64(row.try_get::<f64, _>("price").unwrap_or(0.0)).unwrap_or_default(),
            total: Decimal::from_f64(row.try_get::<f64, _>("total").unwrap_or(0.0)).unwrap_or_default(),
            notes,
            voice_recording_url: row.try_get("voice_recording_url").ok(),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
