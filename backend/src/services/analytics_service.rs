use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate};
use sqlx::{PgPool, Row};
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;

use crate::models::analytics::{AnalyticsSummary, TopProduct, DailySale};

pub struct AnalyticsService {
    pool: PgPool,
}

impl AnalyticsService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get analytics summary for a given period
    pub async fn get_summary(&self, user_id: Uuid, days: i64) -> anyhow::Result<AnalyticsSummary> {
        let end = Utc::now();
        let start = end - Duration::days(days);

        // Get total revenue and transaction count - runtime query
        let row = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(total), 0.0) as total_revenue,
                COUNT(*) as transaction_count,
                COALESCE(SUM(quantity), 0.0) as total_items_sold
            FROM transactions
            WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3
            "#
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let total_revenue = Decimal::from_f64(row.try_get::<f64, _>("total_revenue")?).unwrap_or(Decimal::ZERO);
        let total_transactions: i64 = row.try_get("transaction_count")?;
        let total_items_sold = Decimal::from_f64(row.try_get::<f64, _>("total_items_sold")?).unwrap_or(Decimal::ZERO);

        let average_transaction_value = if total_transactions > 0 {
            total_revenue / Decimal::from(total_transactions)
        } else {
            Decimal::ZERO
        };

        // Get top products
        let top_products = self.get_top_products(user_id, days).await?;

        // Get daily sales
        let daily_sales = self.get_daily_sales(user_id, days).await?;

        Ok(AnalyticsSummary {
            total_revenue,
            total_transactions,
            total_items_sold,
            average_transaction_value,
            top_products,
            daily_sales,
        })
    }

    /// Get summary for a specific period range (for comparison)
    pub async fn get_summary_for_period(
        &self,
        user_id: Uuid,
        days_ago: i64,
        period_length: i64,
    ) -> anyhow::Result<AnalyticsSummary> {
        let end = Utc::now() - Duration::days(days_ago - period_length);
        let start = end - Duration::days(period_length);

        let row = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(total), 0.0) as total_revenue,
                COUNT(*) as transaction_count,
                COALESCE(SUM(quantity), 0.0) as total_items_sold
            FROM transactions
            WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3
            "#
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let total_revenue = Decimal::from_f64(row.try_get::<f64, _>("total_revenue")?).unwrap_or(Decimal::ZERO);
        let total_transactions: i64 = row.try_get("transaction_count")?;
        let total_items_sold = Decimal::from_f64(row.try_get::<f64, _>("total_items_sold")?).unwrap_or(Decimal::ZERO);

        let average_transaction_value = if total_transactions > 0 {
            total_revenue / Decimal::from(total_transactions)
        } else {
            Decimal::ZERO
        };

        Ok(AnalyticsSummary {
            total_revenue,
            total_transactions,
            total_items_sold,
            average_transaction_value,
            top_products: vec![], // Skip for comparison
            daily_sales: vec![],
        })
    }

    /// Get top selling products
    async fn get_top_products(&self, user_id: Uuid, days: i64) -> anyhow::Result<Vec<TopProduct>> {
        let start = Utc::now() - Duration::days(days);

        let rows = sqlx::query(
            r#"
            SELECT 
                p.id as product_id,
                p.name as product_name,
                COUNT(*) as times_sold,
                COALESCE(SUM(t.quantity), 0.0) as total_quantity,
                COALESCE(SUM(t.total), 0.0) as total_revenue
            FROM transactions t
            JOIN products p ON t.product_id = p.id
            WHERE t.user_id = $1 AND t.created_at >= $2
            GROUP BY p.id, p.name
            ORDER BY total_revenue DESC
            LIMIT 5
            "#
        )
        .bind(user_id)
        .bind(start)
        .fetch_all(&self.pool)
        .await?;

        let mut top_products = Vec::new();
        for row in rows {
            top_products.push(TopProduct {
                product_id: row.try_get("product_id")?,
                product_name: row.try_get("product_name")?,
                times_sold: row.try_get::<i64, _>("times_sold")?,
                total_quantity: Decimal::from_f64(row.try_get::<f64, _>("total_quantity")?).unwrap_or(Decimal::ZERO),
                total_revenue: Decimal::from_f64(row.try_get::<f64, _>("total_revenue")?).unwrap_or(Decimal::ZERO),
            });
        }

        Ok(top_products)
    }

    /// Get daily sales breakdown
    async fn get_daily_sales(&self, user_id: Uuid, days: i64) -> anyhow::Result<Vec<DailySale>> {
        let start = Utc::now() - Duration::days(days);

        let rows = sqlx::query(
            r#"
            SELECT 
                DATE(created_at) as sale_date,
                COALESCE(SUM(total), 0.0) as revenue,
                COUNT(*) as transaction_count
            FROM transactions
            WHERE user_id = $1 AND created_at >= $2
            GROUP BY DATE(created_at)
            ORDER BY sale_date ASC
            "#
        )
        .bind(user_id)
        .bind(start)
        .fetch_all(&self.pool)
        .await?;

        let mut daily_sales = Vec::new();
        for row in rows {
            daily_sales.push(DailySale {
                date: row.try_get("sale_date")?,
                revenue: Decimal::from_f64(row.try_get::<f64, _>("revenue")?).unwrap_or(Decimal::ZERO),
                transaction_count: row.try_get::<i64, _>("transaction_count")?,
            });
        }

        Ok(daily_sales)
    }

    /// Get transactions formatted for trend analysis
    pub async fn get_transactions_for_analysis(
        &self,
        user_id: Uuid,
        days: i64,
    ) -> anyhow::Result<Vec<(NaiveDate, f64)>> {
        let start = Utc::now() - Duration::days(days);

        let rows = sqlx::query(
            r#"
            SELECT 
                DATE(created_at) as sale_date,
                COALESCE(SUM(total), 0.0) as daily_total
            FROM transactions
            WHERE user_id = $1 AND created_at >= $2
            GROUP BY DATE(created_at)
            ORDER BY sale_date ASC
            "#
        )
        .bind(user_id)
        .bind(start)
        .fetch_all(&self.pool)
        .await?;

        let mut transactions = Vec::new();
        for row in rows {
            let date: NaiveDate = row.try_get("sale_date")?;
            let total: f64 = row.try_get("daily_total")?;
            transactions.push((date, total));
        }

        Ok(transactions)
    }
}
