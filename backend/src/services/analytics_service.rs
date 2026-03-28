use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate};
use sqlx::{PgPool, Row};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

use crate::models::analytics::{AnalyticsSummary, TopProduct, DailySale, ProductPerformance};

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
                COALESCE(SUM(total), 0.0)::float8 as total_revenue,
                COUNT(*) as transaction_count,
                COALESCE(SUM(quantity), 0.0)::float8 as total_items_sold
            FROM transactions
            WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3
            "#
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let total_revenue = Decimal::from_f64(row.try_get::<f64, _>("total_revenue").unwrap_or(0.0)).unwrap_or(Decimal::ZERO);
        let total_transactions: i64 = row.try_get("transaction_count")?;
        let total_items_sold = Decimal::from_f64(row.try_get::<f64, _>("total_items_sold").unwrap_or(0.0)).unwrap_or(Decimal::ZERO);

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
                COALESCE(SUM(total), 0.0)::float8 as total_revenue,
                COUNT(*) as transaction_count,
                COALESCE(SUM(quantity), 0.0)::float8 as total_items_sold
            FROM transactions
            WHERE user_id = $1 AND created_at >= $2 AND created_at <= $3
            "#
        )
        .bind(user_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        let total_revenue = Decimal::from_f64(row.try_get::<f64, _>("total_revenue").unwrap_or(0.0)).unwrap_or(Decimal::ZERO);
        let total_transactions: i64 = row.try_get("transaction_count")?;
        let total_items_sold = Decimal::from_f64(row.try_get::<f64, _>("total_items_sold").unwrap_or(0.0)).unwrap_or(Decimal::ZERO);

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
                COALESCE(SUM(t.quantity), 0.0)::float8 as total_quantity,
                COALESCE(SUM(t.total), 0.0)::float8 as total_revenue
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
                total_quantity: Decimal::from_f64(row.try_get::<f64, _>("total_quantity").unwrap_or(0.0)).unwrap_or(Decimal::ZERO),
                total_revenue: Decimal::from_f64(row.try_get::<f64, _>("total_revenue").unwrap_or(0.0)).unwrap_or(Decimal::ZERO),
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
                COALESCE(SUM(total), 0.0)::float8 as revenue,
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
                revenue: Decimal::from_f64(row.try_get::<f64, _>("revenue").unwrap_or(0.0)).unwrap_or(Decimal::ZERO),
                transaction_count: row.try_get::<i64, _>("transaction_count")?,
            });
        }

        Ok(daily_sales)
    }

    /// Get per-product performance for a period.
    ///
    /// Returns ALL active products (LEFT JOIN) so that products with zero sales
    /// also appear, labelled "no_sales".  This enables the AI and the frontend
    /// to surface slow-movers alongside top sellers.
    pub async fn get_product_performance(
        &self,
        user_id: Uuid,
        days: i64,
    ) -> anyhow::Result<Vec<ProductPerformance>> {
        let start = Utc::now() - Duration::days(days);

        let rows = sqlx::query(
            r#"
            SELECT
                p.id            AS product_id,
                p.name          AS product_name,
                p.cost_price,
                COALESCE(SUM(t.total),    0.0)::float8 AS total_revenue,
                COALESCE(SUM(t.quantity), 0.0)::float8 AS total_quantity,
                COUNT(t.id)                             AS times_sold,
                CASE
                    WHEN COUNT(t.id) > 0 THEN COALESCE(AVG(t.price), 0.0)::float8
                    ELSE COALESCE(p.default_price, 0.0)::float8
                END AS average_price
            FROM products p
            LEFT JOIN transactions t
                   ON t.product_id = p.id
                  AND t.user_id    = $1
                  AND t.created_at >= $2
            WHERE p.user_id   = $1
              AND p.is_active = true
            GROUP BY p.id, p.name, p.cost_price, p.default_price
            ORDER BY total_revenue DESC
            "#,
        )
        .bind(user_id)
        .bind(start)
        .fetch_all(&self.pool)
        .await?;

        let mut performances = Vec::new();

        for row in rows {
            let total_revenue = Decimal::from_f64(
                row.try_get::<f64, _>("total_revenue").unwrap_or(0.0),
            )
            .unwrap_or(Decimal::ZERO);

            let total_quantity = Decimal::from_f64(
                row.try_get::<f64, _>("total_quantity").unwrap_or(0.0),
            )
            .unwrap_or(Decimal::ZERO);

            let times_sold: i64 = row.try_get("times_sold").unwrap_or(0);

            let average_price = Decimal::from_f64(
                row.try_get::<f64, _>("average_price").unwrap_or(0.0),
            )
            .unwrap_or(Decimal::ZERO);

            let cost_price: Option<Decimal> = row
                .try_get::<Option<f64>, _>("cost_price")
                .ok()
                .flatten()
                .and_then(Decimal::from_f64);

            // Estimated profit = revenue − (qty × unit_cost)
            let (estimated_profit, profit_margin_pct) = if let Some(cost) = cost_price {
                let cogs = total_quantity * cost;
                let profit = total_revenue - cogs;
                let margin_pct = if total_revenue > Decimal::ZERO {
                    (profit / total_revenue)
                        .to_f64()
                        .map(|v| v * 100.0)
                } else {
                    None
                };
                (Some(profit), margin_pct)
            } else {
                (None, None)
            };

            // Performance label based on share of total revenue is computed
            // after we have all rows; for now assign raw bucket.
            let performance_label = match times_sold {
                0 => "no_sales",
                1..=2 => "slow_mover",
                3..=9 => "good",
                _ => "top_seller",
            }
            .to_string();

            performances.push(ProductPerformance {
                product_id: row.try_get("product_id")?,
                product_name: row.try_get("product_name")?,
                total_revenue,
                total_quantity,
                times_sold,
                average_price,
                cost_price,
                estimated_profit,
                profit_margin_pct,
                performance_label,
            });
        }

        // Re-classify top_seller as the top-25% by revenue (if there are any sales at all)
        let max_rev = performances
            .iter()
            .map(|p| p.total_revenue)
            .max()
            .unwrap_or(Decimal::ZERO);

        if max_rev > Decimal::ZERO {
            let threshold = max_rev * Decimal::from_f64(0.5).unwrap_or(Decimal::ZERO);
            for p in performances.iter_mut() {
                if p.times_sold > 0 && p.total_revenue >= threshold {
                    p.performance_label = "top_seller".to_string();
                }
            }
        }

        Ok(performances)
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
                COALESCE(SUM(total), 0.0)::float8 as daily_total
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
            let total_f64: f64 = row.try_get("daily_total").unwrap_or(0.0);
            transactions.push((date, total_f64));
        }

        Ok(transactions)
    }
}
