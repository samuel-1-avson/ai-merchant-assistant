//! Analytics Engine - Core analytics calculations

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{Utc, Duration, Datelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

use crate::models::analytics::{AnalyticsSummary, TopProduct, DailySale, SalesTrend};
use crate::analytics::TrendDirection;

pub struct AnalyticsEngine {
    pool: PgPool,
}

impl AnalyticsEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get analytics summary (runtime query)
    pub async fn get_summary(&self, user_id: Uuid, days: i64) -> anyhow::Result<AnalyticsSummary> {
        let start_date = Utc::now() - Duration::days(days);

        // Get summary using runtime query
        let row = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(total_revenue), 0) as total_revenue,
                COALESCE(SUM(transaction_count), 0) as total_transactions,
                COALESCE(SUM(total_items_sold), 0) as total_items_sold
            FROM mv_daily_sales_summary
            WHERE user_id = $1 AND sale_date >= $2
            "#
        )
        .bind(user_id)
        .bind(start_date.date_naive())
        .fetch_one(&self.pool)
        .await?;

        let total_revenue: Decimal = row.try_get("total_revenue")?;
        let total_transactions: i64 = row.try_get("total_transactions")?;
        let total_items_sold: Decimal = row.try_get("total_items_sold")?;

        let avg_transaction_value = if total_transactions > 0 {
            total_revenue / Decimal::from(total_transactions)
        } else {
            Decimal::ZERO
        };

        // Get top products
        let top_products = self.get_top_products(user_id, 5).await?;

        // Get daily sales
        let daily_sales = self.get_daily_sales(user_id, days).await?;

        Ok(AnalyticsSummary {
            total_revenue,
            total_transactions,
            total_items_sold,
            average_transaction_value: avg_transaction_value,
            top_products,
            daily_sales,
        })
    }

    /// Get top products (runtime query)
    pub async fn get_top_products(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> anyhow::Result<Vec<TopProduct>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                product_id,
                product_name,
                total_quantity,
                total_revenue,
                times_sold
            FROM mv_top_products
            WHERE user_id = $1
            ORDER BY total_revenue DESC
            LIMIT $2
            "#
        )
        .bind(user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let products = rows.into_iter().map(|row| {
            TopProduct {
                product_id: row.try_get("product_id").unwrap_or_default(),
                product_name: row.try_get("product_name").unwrap_or_default(),
                total_quantity: row.try_get("total_quantity").unwrap_or_default(),
                total_revenue: row.try_get("total_revenue").unwrap_or_default(),
                times_sold: row.try_get("times_sold").unwrap_or(0),
            }
        }).collect();

        Ok(products)
    }

    /// Get daily sales (runtime query)
    pub async fn get_daily_sales(
        &self,
        user_id: Uuid,
        days: i64,
    ) -> anyhow::Result<Vec<DailySale>> {
        let start_date = (Utc::now() - Duration::days(days)).date_naive();

        let rows = sqlx::query(
            r#"
            SELECT 
                sale_date,
                total_revenue,
                transaction_count
            FROM mv_daily_sales_summary
            WHERE user_id = $1 AND sale_date >= $2
            ORDER BY sale_date
            "#
        )
        .bind(user_id)
        .bind(start_date)
        .fetch_all(&self.pool)
        .await?;

        let sales = rows.into_iter().map(|row| {
            DailySale {
                date: row.try_get("sale_date").unwrap_or_default(),
                revenue: row.try_get("total_revenue").unwrap_or_default(),
                transaction_count: row.try_get("transaction_count").unwrap_or(0),
            }
        }).collect();

        Ok(sales)
    }

    /// Calculate trend direction
    pub fn calculate_trend(current: Decimal, previous: Decimal) -> TrendDirection {
        if current > previous {
            TrendDirection::Increasing
        } else if current < previous {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate sales trend
    pub async fn get_sales_trend(&self, user_id: Uuid, period: &str) -> anyhow::Result<SalesTrend> {
        let (current, previous) = match period {
            "week" => {
                let current = self.get_summary(user_id, 7).await?.total_revenue;
                let previous = self.get_summary(user_id, 14).await?.total_revenue - current;
                (current, previous)
            }
            "month" => {
                let current = self.get_summary(user_id, 30).await?.total_revenue;
                let previous = self.get_summary(user_id, 60).await?.total_revenue - current;
                (current, previous)
            }
            _ => (Decimal::ZERO, Decimal::ZERO),
        };

        let change_percent = if previous > Decimal::ZERO {
            ((current - previous) / previous).to_f64().unwrap_or(0.0) * 100.0
        } else {
            0.0
        };

        Ok(SalesTrend {
            period: period.to_string(),
            current_value: current,
            previous_value: previous,
            change_percent,
        })
    }
}
