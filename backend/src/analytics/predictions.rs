//! Predictions Engine - Forecasting and demand prediction

use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate, Datelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use sqlx::PgPool;

use crate::analytics::{TimeSeriesPoint, TrendDirection};

pub struct PredictionEngine {
    pool: PgPool,
}

/// Demand forecast result
#[derive(Debug, Clone)]
pub struct DemandForecast {
    pub product_id: Uuid,
    pub historical_average: Decimal,
    pub forecasted_demand: Vec<TimeSeriesPoint>,
    pub confidence: f64,
}

/// Revenue prediction result
#[derive(Debug, Clone)]
pub struct RevenuePrediction {
    pub predicted_revenue: Decimal,
    pub lower_bound: Decimal,
    pub upper_bound: Decimal,
    pub confidence: f64,
}

/// Stock requirement prediction
#[derive(Debug, Clone)]
pub struct StockRequirement {
    pub product_id: Uuid,
    pub product_name: String,
    pub current_stock: i32,
    pub recommended_stock: i32,
    pub days_until_stockout: Option<i64>,
}

impl PredictionEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Predict demand for a product (runtime query)
    pub async fn predict_demand(
        &self,
        product_id: Uuid,
        days_to_forecast: i64,
    ) -> anyhow::Result<DemandForecast> {
        // Get historical sales data (last 90 days)
        let rows = sqlx::query(
            r#"
            SELECT 
                DATE(created_at) as sale_date,
                SUM(quantity) as total_quantity
            FROM transactions
            WHERE product_id = $1 AND created_at >= NOW() - INTERVAL '90 days'
            GROUP BY DATE(created_at)
            ORDER BY sale_date
            "#
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 7 {
            // Not enough data
            return Ok(DemandForecast {
                product_id,
                historical_average: Decimal::ZERO,
                forecasted_demand: vec![],
                confidence: 0.0,
            });
        }

        // Parse historical data
        let historical: Vec<(NaiveDate, Decimal)> = rows
            .into_iter()
            .filter_map(|row| {
                let date: Option<NaiveDate> = row.try_get("sale_date").ok();
                let qty: Option<Decimal> = row.try_get("total_quantity").ok();
                date.zip(qty)
            })
            .collect();

        // Calculate simple moving average
        let total_quantity: f64 = historical
            .iter()
            .map(|(_, qty)| qty.to_f64().unwrap_or(0.0))
            .sum();
        let avg_daily_demand = total_quantity / historical.len() as f64;

        // Generate forecast
        let last_date = historical
            .last()
            .map(|(d, _)| *d)
            .unwrap_or_else(|| Utc::now().date_naive());

        let mut forecasted_demand = Vec::new();
        for i in 1..=days_to_forecast {
            let date = last_date + Duration::days(i);
            let day_of_week = date.weekday().num_days_from_monday() as f64;
            let seasonal_factor = if day_of_week >= 5.0 { 1.2 } else { 1.0 };
            
            let value = Decimal::from_f64(avg_daily_demand * seasonal_factor).unwrap_or_default();
            forecasted_demand.push(TimeSeriesPoint { date, value });
        }

        Ok(DemandForecast {
            product_id,
            historical_average: Decimal::from_f64(avg_daily_demand).unwrap_or_default(),
            forecasted_demand,
            confidence: 0.7,
        })
    }

    /// Predict revenue for the next period (runtime query)
    pub async fn predict_revenue(
        &self,
        user_id: Uuid,
        days_to_forecast: i64,
    ) -> anyhow::Result<RevenuePrediction> {
        let rows = sqlx::query(
            r#"
            SELECT 
                sale_date,
                total_revenue
            FROM mv_daily_sales_summary
            WHERE user_id = $1
            ORDER BY sale_date DESC
            LIMIT 30
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 7 {
            return Ok(RevenuePrediction {
                predicted_revenue: Decimal::ZERO,
                lower_bound: Decimal::ZERO,
                upper_bound: Decimal::ZERO,
                confidence: 0.0,
            });
        }

        let revenues: Vec<f64> = rows
            .iter()
            .filter_map(|r| {
                let rev: Option<Decimal> = r.try_get("total_revenue").ok();
                rev.map(|d| d.to_f64().unwrap_or(0.0))
            })
            .collect();

        let avg_revenue = revenues.iter().sum::<f64>() / revenues.len() as f64;
        let std_dev = (revenues
            .iter()
            .map(|r| (r - avg_revenue).powi(2))
            .sum::<f64>() / revenues.len() as f64)
            .sqrt();

        let predicted_revenue = Decimal::from_f64(avg_revenue * days_to_forecast as f64).unwrap_or_default();
        let lower_bound = Decimal::from_f64((avg_revenue - 1.96 * std_dev) * days_to_forecast as f64)
            .unwrap_or_default()
            .max(Decimal::ZERO);
        let upper_bound = Decimal::from_f64((avg_revenue + 1.96 * std_dev) * days_to_forecast as f64)
            .unwrap_or_default();

        let confidence = if std_dev / avg_revenue < 0.2 { 0.9 } else { 0.7 };

        Ok(RevenuePrediction {
            predicted_revenue,
            lower_bound,
            upper_bound,
            confidence,
        })
    }

    /// Predict stock requirements (runtime query)
    pub async fn predict_stock_requirements(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<StockRequirement>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                p.id as product_id,
                p.name as product_name,
                p.stock_quantity,
                p.low_stock_threshold,
                COALESCE(SUM(t.quantity), 0) as sold_last_30_days
            FROM products p
            LEFT JOIN transactions t ON p.id = t.product_id 
                AND t.created_at >= NOW() - INTERVAL '30 days'
            WHERE p.user_id = $1 AND p.is_active = true
            GROUP BY p.id, p.name, p.stock_quantity, p.low_stock_threshold
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut requirements = Vec::new();

        for row in rows {
            let product_id: Uuid = row.try_get("product_id")?;
            let product_name: String = row.try_get("product_name")?;
            let stock_quantity: i32 = row.try_get("stock_quantity")?;
            let sold_last_30_days: Option<Decimal> = row.try_get("sold_last_30_days")?;

            let daily_demand = sold_last_30_days.map(|d| d.to_f64().unwrap_or(0.0) / 30.0).unwrap_or(0.0);
            
            let recommended_stock = if daily_demand > 0.0 {
                (daily_demand * 60.0) as i32 // 60 days of stock
            } else {
                stock_quantity
            };

            let days_until_stockout = if daily_demand > 0.0 {
                Some((stock_quantity as f64 / daily_demand) as i64)
            } else {
                None
            };

            requirements.push(StockRequirement {
                product_id,
                product_name,
                current_stock: stock_quantity,
                recommended_stock: recommended_stock.max(stock_quantity),
                days_until_stockout,
            });
        }

        Ok(requirements)
    }
}
