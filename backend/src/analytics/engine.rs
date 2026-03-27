use std::sync::Arc;
use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use sqlx::PgPool;

use crate::models::analytics::{AnalyticsSummary, TopProduct, DailySale};
use crate::analytics::{TrendAnalysis, TrendDirection, TimeSeriesPoint};

pub struct AnalyticsEngine {
    pool: PgPool,
}

impl AnalyticsEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_summary(&self, user_id: Uuid, days: i64) -> anyhow::Result<AnalyticsSummary> {
        let start_date = Utc::now() - Duration::days(days);

        // Get summary from materialized view
        let summary = sqlx::query!(
            r#"
            SELECT 
                COALESCE(SUM(total_revenue), 0) as total_revenue,
                COALESCE(SUM(transaction_count), 0) as total_transactions,
                COALESCE(SUM(total_items_sold), 0) as total_items_sold
            FROM mv_daily_sales_summary
            WHERE user_id = $1 AND sale_date >= $2
            "#,
            user_id,
            start_date.date_naive()
        )
        .fetch_one(&self.pool)
        .await?;

        let total_revenue = summary.total_revenue.unwrap_or_default();
        let total_transactions = summary.total_transactions.unwrap_or(0);
        let total_items_sold = summary.total_items_sold.unwrap_or_default();

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

    pub async fn get_top_products(
        &self,
        user_id: Uuid,
        limit: i64,
    ) -> anyhow::Result<Vec<TopProduct>> {
        let products = sqlx::query_as!(
            TopProduct,
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
            "#,
            user_id,
            limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(products)
    }

    pub async fn get_daily_sales(
        &self,
        user_id: Uuid,
        days: i64,
    ) -> anyhow::Result<Vec<DailySale>> {
        let start_date = (Utc::now() - Duration::days(days)).date_naive();

        let sales = sqlx::query_as!(
            DailySale,
            r#"
            SELECT 
                sale_date as date,
                total_revenue as revenue,
                transaction_count
            FROM mv_daily_sales_summary
            WHERE user_id = $1 AND sale_date >= $2
            ORDER BY sale_date ASC
            "#,
            user_id,
            start_date
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(sales)
    }

    pub async fn analyze_trends(
        &self,
        user_id: Uuid,
        days: i64,
    ) -> anyhow::Result<TrendAnalysis> {
        let daily_sales = self.get_daily_sales(user_id, days).await?;

        if daily_sales.len() < 2 {
            return Ok(TrendAnalysis {
                direction: TrendDirection::Stable,
                slope: 0.0,
                r_squared: 0.0,
                forecast: vec![],
            });
        }

        // Simple linear regression
        let n = daily_sales.len() as f64;
        let sum_x: f64 = (0..daily_sales.len()).map(|i| i as f64).sum();
        let sum_y: f64 = daily_sales.iter().map(|s| s.revenue.to_f64().unwrap_or(0.0)).sum();
        let sum_xy: f64 = daily_sales
            .iter()
            .enumerate()
            .map(|(i, s)| i as f64 * s.revenue.to_f64().unwrap_or(0.0))
            .sum();
        let sum_x2: f64 = (0..daily_sales.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate R-squared
        let y_mean = sum_y / n;
        let ss_tot: f64 = daily_sales
            .iter()
            .map(|s| (s.revenue.to_f64().unwrap_or(0.0) - y_mean).powi(2))
            .sum();
        let ss_res: f64 = daily_sales
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let y_pred = slope * i as f64 + intercept;
                (s.revenue.to_f64().unwrap_or(0.0) - y_pred).powi(2)
            })
            .sum();
        let r_squared = 1.0 - (ss_res / ss_tot);

        let direction = if slope > 0.01 {
            TrendDirection::Increasing
        } else if slope < -0.01 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        };

        // Generate forecast for next 7 days
        let last_date = daily_sales.last().map(|s| s.date).unwrap_or_else(|| Utc::now().date_naive());
        let mut forecast = Vec::new();
        for i in 1..=7 {
            let date = last_date + Duration::days(i);
            let x = daily_sales.len() as f64 + i as f64;
            let value = Decimal::from_f64(slope * x + intercept).unwrap_or_default();
            forecast.push(TimeSeriesPoint { date, value });
        }

        Ok(TrendAnalysis {
            direction,
            slope,
            r_squared,
            forecast,
        })
    }

    pub async fn compare_periods(
        &self,
        user_id: Uuid,
        current_days: i64,
        previous_days: i64,
    ) -> anyhow::Result<PeriodComparison> {
        let current = self.get_summary(user_id, current_days).await?;
        let previous = self.get_summary(user_id, previous_days).await?;

        let revenue_change = if previous.total_revenue > Decimal::ZERO {
            ((current.total_revenue - previous.total_revenue) / previous.total_revenue * Decimal::from(100))
                .to_f64()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let transaction_change = if previous.total_transactions > 0 {
            ((current.total_transactions - previous.total_transactions) as f64 / previous.total_transactions as f64 * 100.0)
        } else {
            0.0
        };

        Ok(PeriodComparison {
            current_revenue: current.total_revenue,
            previous_revenue: previous.total_revenue,
            revenue_change_percent: revenue_change,
            current_transactions: current.total_transactions,
            previous_transactions: previous.total_transactions,
            transaction_change_percent: transaction_change,
        })
    }

    pub async fn refresh_materialized_views(&self) -> anyhow::Result<()> {
        sqlx::query!("SELECT refresh_analytics_views()")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PeriodComparison {
    pub current_revenue: Decimal,
    pub previous_revenue: Decimal,
    pub revenue_change_percent: f64,
    pub current_transactions: i64,
    pub previous_transactions: i64,
    pub transaction_change_percent: f64,
}
