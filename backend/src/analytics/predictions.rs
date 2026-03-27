use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate};
use rust_decimal::Decimal;
use sqlx::PgPool;

use crate::analytics::{TimeSeriesPoint, TrendDirection};

pub struct PredictionEngine {
    pool: PgPool,
}

impl PredictionEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Forecast demand for a specific product using simple moving average
    pub async fn forecast_product_demand(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        days_to_forecast: i64,
    ) -> anyhow::Result<DemandForecast> {
        // Get historical daily sales for the product
        let historical = sqlx::query!(
            r#"
            SELECT 
                DATE(created_at) as sale_date,
                SUM(quantity) as total_quantity
            FROM transactions
            WHERE user_id = $1 
                AND product_id = $2
                AND created_at >= NOW() - INTERVAL '30 days'
            GROUP BY DATE(created_at)
            ORDER BY sale_date ASC
            "#,
            user_id,
            product_id
        )
        .fetch_all(&self.pool)
        .await?;

        if historical.is_empty() {
            return Ok(DemandForecast {
                product_id,
                historical_average: Decimal::ZERO,
                forecasted_demand: vec![],
                confidence: 0.0,
            });
        }

        // Calculate simple moving average
        let total_quantity: f64 = historical
            .iter()
            .map(|h| h.total_quantity.unwrap_or_default().to_f64().unwrap_or(0.0))
            .sum();
        let avg_daily_demand = total_quantity / historical.len() as f64;

        // Generate forecast
        let last_date = historical
            .last()
            .and_then(|h| h.sale_date)
            .unwrap_or_else(|| Utc::now().date_naive());

        let mut forecasted_demand = Vec::new();
        for i in 1..=days_to_forecast {
            let date = last_date + Duration::days(i);
            // Add some seasonality (weekends might have different patterns)
            let day_of_week = date.weekday().num_days_from_monday() as f64;
            let seasonal_factor = if day_of_week >= 5.0 { 1.2 } else { 1.0 }; // Weekend boost
            
            let value = Decimal::from_f64(avg_daily_demand * seasonal_factor).unwrap_or_default();
            forecasted_demand.push(TimeSeriesPoint { date, value });
        }

        // Calculate confidence based on data variance
        let variance = historical
            .iter()
            .map(|h| {
                let diff = h.total_quantity.unwrap_or_default().to_f64().unwrap_or(0.0) - avg_daily_demand;
                diff.powi(2)
            })
            .sum::<f64>() / historical.len() as f64;
        let std_dev = variance.sqrt();
        let confidence = if std_dev / avg_daily_demand < 0.3 {
            0.9 // Low variance = high confidence
        } else if std_dev / avg_daily_demand < 0.6 {
            0.7 // Medium variance
        } else {
            0.5 // High variance = low confidence
        };

        Ok(DemandForecast {
            product_id,
            historical_average: Decimal::from_f64(avg_daily_demand).unwrap_or_default(),
            forecasted_demand,
            confidence,
        })
    }

    /// Predict revenue for the next period
    pub async fn predict_revenue(
        &self,
        user_id: Uuid,
        days_to_forecast: i64,
    ) -> anyhow::Result<RevenuePrediction> {
        let daily_sales = sqlx::query!(
            r#"
            SELECT 
                sale_date,
                total_revenue
            FROM mv_daily_sales_summary
            WHERE user_id = $1
            ORDER BY sale_date DESC
            LIMIT 30
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        if daily_sales.len() < 7 {
            return Ok(RevenuePrediction {
                predicted_revenue: Decimal::ZERO,
                lower_bound: Decimal::ZERO,
                upper_bound: Decimal::ZERO,
                confidence: 0.0,
            });
        }

        let revenues: Vec<f64> = daily_sales
            .iter()
            .map(|s| s.total_revenue.to_f64().unwrap_or(0.0))
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

        let confidence = if revenues.len() >= 30 {
            0.9
        } else if revenues.len() >= 14 {
            0.75
        } else {
            0.6
        };

        Ok(RevenuePrediction {
            predicted_revenue,
            lower_bound,
            upper_bound,
            confidence,
        })
    }

    /// Detect anomalies in sales patterns
    pub async fn detect_anomalies(
        &self,
        user_id: Uuid,
        lookback_days: i64,
    ) -> anyhow::Result<Vec<Anomaly>> {
        let daily_sales = sqlx::query!(
            r#"
            SELECT 
                sale_date,
                total_revenue,
                transaction_count
            FROM mv_daily_sales_summary
            WHERE user_id = $1 AND sale_date >= $2
            ORDER BY sale_date ASC
            "#,
            user_id,
            (Utc::now() - Duration::days(lookback_days)).date_naive()
        )
        .fetch_all(&self.pool)
        .await?;

        if daily_sales.len() < 7 {
            return Ok(vec![]);
        }

        let revenues: Vec<f64> = daily_sales
            .iter()
            .map(|s| s.total_revenue.to_f64().unwrap_or(0.0))
            .collect();

        let mean = revenues.iter().sum::<f64>() / revenues.len() as f64;
        let std_dev = (revenues
            .iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / revenues.len() as f64)
            .sqrt();

        let mut anomalies = Vec::new();
        for (i, sale) in daily_sales.iter().enumerate() {
            let revenue = sale.total_revenue.to_f64().unwrap_or(0.0);
            let z_score = (revenue - mean) / std_dev;

            if z_score.abs() > 2.0 {
                anomalies.push(Anomaly {
                    date: sale.sale_date.unwrap_or_else(|| Utc::now().date_naive()),
                    anomaly_type: if z_score > 0.0 {
                        AnomalyType::Spike
                    } else {
                        AnomalyType::Drop
                    },
                    severity: if z_score.abs() > 3.0 {
                        AnomalySeverity::High
                    } else {
                        AnomalySeverity::Medium
                    },
                    actual_value: sale.total_revenue,
                    expected_value: Decimal::from_f64(mean).unwrap_or_default(),
                    deviation_percent: (z_score * 100.0).abs(),
                });
            }
        }

        Ok(anomalies)
    }

    /// Get inventory recommendations
    pub async fn get_inventory_recommendations(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<InventoryRecommendation>> {
        let products = sqlx::query!(
            r#"
            SELECT 
                p.id as product_id,
                p.name as product_name,
                p.stock_quantity,
                p.low_stock_threshold,
                COALESCE(mv.total_quantity, 0) as sold_last_30_days
            FROM products p
            LEFT JOIN mv_top_products mv ON p.id = mv.product_id AND p.user_id = mv.user_id
            WHERE p.user_id = $1 AND p.is_active = true
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut recommendations = Vec::new();
        for product in products {
            let daily_demand = product.sold_last_30_days.to_f64().unwrap_or(0.0) / 30.0;
            let days_of_stock = if daily_demand > 0.0 {
                product.stock_quantity as f64 / daily_demand
            } else {
                999.0 // No demand means effectively infinite days
            };

            let recommendation = if product.stock_quantity <= 0 {
                InventoryRecommendation {
                    product_id: product.product_id,
                    product_name: product.product_name,
                    current_stock: product.stock_quantity,
                    recommended_action: RecommendedAction::UrgentRestock,
                    suggested_quantity: (daily_demand * 14.0).ceil() as i32,
                    reason: "Out of stock".to_string(),
                }
            } else if days_of_stock < 7.0 {
                InventoryRecommendation {
                    product_id: product.product_id,
                    product_name: product.product_name,
                    current_stock: product.stock_quantity,
                    recommended_action: RecommendedAction::Restock,
                    suggested_quantity: (daily_demand * 21.0).ceil() as i32,
                    reason: format!("Low stock: {:.0} days remaining", days_of_stock),
                }
            } else if days_of_stock > 90.0 && product.sold_last_30_days.to_f64().unwrap_or(0.0) > 0.0 {
                InventoryRecommendation {
                    product_id: product.product_id,
                    product_name: product.product_name,
                    current_stock: product.stock_quantity,
                    recommended_action: RecommendedAction::ReduceStock,
                    suggested_quantity: 0,
                    reason: format!("Overstocked: {:.0} days of inventory", days_of_stock),
                }
            } else {
                continue; // No recommendation needed
            };

            recommendations.push(recommendation);
        }

        Ok(recommendations)
    }
}

#[derive(Debug, Clone)]
pub struct DemandForecast {
    pub product_id: Uuid,
    pub historical_average: Decimal,
    pub forecasted_demand: Vec<TimeSeriesPoint>,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct RevenuePrediction {
    pub predicted_revenue: Decimal,
    pub lower_bound: Decimal,
    pub upper_bound: Decimal,
    pub confidence: f64,
}

#[derive(Debug, Clone)]
pub struct Anomaly {
    pub date: NaiveDate,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub actual_value: Decimal,
    pub expected_value: Decimal,
    pub deviation_percent: f64,
}

#[derive(Debug, Clone)]
pub enum AnomalyType {
    Spike,
    Drop,
}

#[derive(Debug, Clone)]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone)]
pub struct InventoryRecommendation {
    pub product_id: Uuid,
    pub product_name: String,
    pub current_stock: i32,
    pub recommended_action: RecommendedAction,
    pub suggested_quantity: i32,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub enum RecommendedAction {
    Restock,
    UrgentRestock,
    ReduceStock,
    Monitor,
}
