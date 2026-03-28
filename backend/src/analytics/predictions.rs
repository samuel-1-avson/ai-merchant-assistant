//! Prediction Engine — demand forecasting with linear regression + EWMA.
//!
//! Improvements over the original simple-moving-average implementation:
//!
//!  1. **Linear regression** — fits a trend line to historical data and
//!     extrapolates it forward. The slope tells us whether sales are growing
//!     or shrinking.
//!  2. **Exponential Weighted Moving Average (EWMA, α=0.3)** — gives more
//!     weight to recent data so a sudden sales spike is reflected quickly.
//!  3. **Day-of-week seasonal multipliers** — seven independent factors
//!     learned from history, not a hard-coded weekend 1.2x.
//!  4. **Real R² confidence** — derived from regression residuals.
//!  5. **SQL fix** — queries `daily_sales` view (fixed from broken `mv_daily_sales_summary`).
//!  6. **Minimum data guard raised** to 14 days.

use uuid::Uuid;
use chrono::{Utc, Duration, NaiveDate, Datelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use sqlx::{PgPool, Row};

use crate::analytics::{TimeSeriesPoint, TrendDirection};

// ── Structs ───────────────────────────────────────────────────────────────

pub struct PredictionEngine {
    pool: PgPool,
}

#[derive(Debug, Clone)]
pub struct DemandForecast {
    pub product_id: Uuid,
    pub historical_average: Decimal,
    pub forecasted_demand: Vec<TimeSeriesPoint>,
    /// R² from the linear regression (0.0 – 1.0)
    pub confidence: f64,
    pub trend: TrendDirection,
    pub trend_slope: f64,
}

#[derive(Debug, Clone)]
pub struct RevenuePrediction {
    pub predicted_revenue: Decimal,
    pub lower_bound: Decimal,
    pub upper_bound: Decimal,
    pub confidence: f64,
    pub trend: TrendDirection,
}

#[derive(Debug, Clone)]
pub struct StockRequirement {
    pub product_id: Uuid,
    pub product_name: String,
    pub current_stock: i32,
    pub recommended_stock: i32,
    pub days_until_stockout: Option<i64>,
}

// ── PredictionEngine ──────────────────────────────────────────────────────

impl PredictionEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Predict demand for a product using linear regression + EWMA + DoW seasonality.
    pub async fn predict_demand(
        &self,
        product_id: Uuid,
        days_to_forecast: i64,
    ) -> anyhow::Result<DemandForecast> {
        let rows = sqlx::query(
            r#"
            SELECT
                DATE(created_at)       AS sale_date,
                SUM(quantity)::float8  AS total_quantity
            FROM transactions
            WHERE product_id = $1
              AND created_at >= NOW() - INTERVAL '90 days'
            GROUP BY DATE(created_at)
            ORDER BY sale_date
            "#,
        )
        .bind(product_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 14 {
            return Ok(DemandForecast {
                product_id,
                historical_average: Decimal::ZERO,
                forecasted_demand: vec![],
                confidence: 0.0,
                trend: TrendDirection::Stable,
                trend_slope: 0.0,
            });
        }

        let historical: Vec<(NaiveDate, f64)> = rows
            .into_iter()
            .filter_map(|r| {
                let date: Option<NaiveDate> = r.try_get("sale_date").ok();
                let qty: Option<f64>        = r.try_get("total_quantity").ok();
                date.zip(qty)
            })
            .collect();

        let values: Vec<f64> = historical.iter().map(|(_, v)| *v).collect();
        let n = values.len();

        let (slope, intercept, r_squared) = linear_regression(&values);
        let ewma_base = ewma(&values, 0.3);
        let dow_factors = compute_dow_factors(&historical);
        let avg_daily = values.iter().sum::<f64>() / n as f64;

        let trend = classify_trend(slope);
        let last_date = historical.last().map(|(d, _)| *d).unwrap_or_else(|| Utc::now().date_naive());

        let mut forecasted_demand = Vec::new();
        for i in 1..=days_to_forecast {
            let date = last_date + Duration::days(i);
            let trend_val = intercept + slope * (n as f64 + i as f64);
            let blended = 0.6 * ewma_base + 0.4 * trend_val;
            let dow = date.weekday().num_days_from_monday() as usize;
            let seasonal = (blended * dow_factors[dow]).max(0.0);
            let value = Decimal::from_f64(seasonal).unwrap_or_default();
            forecasted_demand.push(TimeSeriesPoint { date, value });
        }

        Ok(DemandForecast {
            product_id,
            historical_average: Decimal::from_f64(avg_daily).unwrap_or_default(),
            forecasted_demand,
            confidence: r_squared.min(1.0).max(0.0),
            trend,
            trend_slope: slope,
        })
    }

    /// Predict total revenue. Queries `daily_sales` view (fixed SQL).
    pub async fn predict_revenue(
        &self,
        user_id: Uuid,
        days_to_forecast: i64,
    ) -> anyhow::Result<RevenuePrediction> {
        // Fixed: was querying non-existent mv_daily_sales_summary
        let rows = sqlx::query(
            r#"
            SELECT
                date::date       AS sale_date,
                revenue::float8  AS total_revenue
            FROM daily_sales
            WHERE user_id = $1
            ORDER BY date DESC
            LIMIT 60
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 14 {
            return Ok(RevenuePrediction {
                predicted_revenue: Decimal::ZERO,
                lower_bound: Decimal::ZERO,
                upper_bound: Decimal::ZERO,
                confidence: 0.0,
                trend: TrendDirection::Stable,
            });
        }

        let revenues: Vec<f64> = rows
            .iter()
            .rev() // chronological
            .filter_map(|r| r.try_get::<f64, _>("total_revenue").ok())
            .collect();

        let n = revenues.len();
        let (slope, intercept, r_squared) = linear_regression(&revenues);
        let ewma_rev = ewma(&revenues, 0.3);

        let mut daily_total = 0.0f64;
        for i in 1..=days_to_forecast {
            let trend_val = intercept + slope * (n as f64 + i as f64);
            let blended = (0.6 * ewma_rev + 0.4 * trend_val).max(0.0);
            daily_total += blended;
        }

        let avg = revenues.iter().sum::<f64>() / n as f64;
        let std_dev = (revenues.iter().map(|r| (r - avg).powi(2)).sum::<f64>() / n as f64).sqrt();
        let margin = 1.96 * std_dev * (days_to_forecast as f64).sqrt();

        Ok(RevenuePrediction {
            predicted_revenue: Decimal::from_f64(daily_total).unwrap_or_default(),
            lower_bound: Decimal::from_f64((daily_total - margin).max(0.0)).unwrap_or_default(),
            upper_bound: Decimal::from_f64(daily_total + margin).unwrap_or_default(),
            confidence: r_squared.min(1.0).max(0.0),
            trend: classify_trend(slope),
        })
    }

    /// Predict stock requirements for all active products.
    pub async fn predict_stock_requirements(
        &self,
        user_id: Uuid,
    ) -> anyhow::Result<Vec<StockRequirement>> {
        let rows = sqlx::query(
            r#"
            SELECT
                p.id                                          AS product_id,
                p.name                                        AS product_name,
                p.stock_quantity,
                p.low_stock_threshold,
                COALESCE(SUM(t.quantity)::float8, 0)         AS sold_last_30_days
            FROM products p
            LEFT JOIN transactions t
                   ON p.id = t.product_id
                  AND t.created_at >= NOW() - INTERVAL '30 days'
            WHERE p.user_id = $1 AND p.is_active = true
            GROUP BY p.id, p.name, p.stock_quantity, p.low_stock_threshold
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut requirements = Vec::new();
        for row in rows {
            let product_id: Uuid    = row.try_get("product_id")?;
            let product_name: String = row.try_get("product_name")?;
            let stock_quantity: i32 = row.try_get("stock_quantity")?;
            let sold: f64           = row.try_get("sold_last_30_days").unwrap_or(0.0);

            let daily_demand = sold / 30.0;
            let recommended_stock = if daily_demand > 0.0 {
                (daily_demand * 60.0).ceil() as i32
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

// ── SimpleForecaster (used by analytics route layer) ─────────────────────

pub struct SimpleForecaster;

#[derive(Debug, Clone)]
pub struct RevenueForecast {
    pub predicted: f64,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub confidence: f64,
    pub trend: TrendDirection,
    pub trend_slope: f64,
    pub daily_values: Vec<serde_json::Value>,
}

impl SimpleForecaster {
    pub fn new() -> Self { Self }

    pub fn forecast(&self, historical: &[(NaiveDate, f64)], days: i64) -> Vec<serde_json::Value> {
        if historical.is_empty() { return vec![]; }
        let values: Vec<f64> = historical.iter().map(|(_, v)| *v).collect();
        let (slope, intercept, _) = linear_regression(&values);
        let ewma_base = ewma(&values, 0.3);
        let dow_factors = compute_dow_factors(historical);
        let n = values.len();
        let last_date = historical.last().map(|(d, _)| *d).unwrap_or_else(|| Utc::now().date_naive());

        (1..=days).map(|i| {
            let date = last_date + Duration::days(i);
            let trend_val = intercept + slope * (n as f64 + i as f64);
            let blended = 0.6 * ewma_base + 0.4 * trend_val;
            let dow = date.weekday().num_days_from_monday() as usize;
            let v = (blended * dow_factors[dow]).max(0.0);
            serde_json::json!({ "date": date.to_string(), "value": v })
        }).collect()
    }

    pub fn forecast_revenue(&self, historical: &[(NaiveDate, f64)], days: i64) -> RevenueForecast {
        if historical.is_empty() {
            return RevenueForecast {
                predicted: 0.0, lower_bound: 0.0, upper_bound: 0.0,
                confidence: 0.0, trend: TrendDirection::Stable, trend_slope: 0.0,
                daily_values: vec![],
            };
        }
        let values: Vec<f64> = historical.iter().map(|(_, v)| *v).collect();
        let n = values.len();
        let (slope, intercept, r_squared) = linear_regression(&values);
        let ewma_base = ewma(&values, 0.3);
        let dow_factors = compute_dow_factors(historical);
        let last_date = historical.last().map(|(d, _)| *d).unwrap_or_else(|| Utc::now().date_naive());

        let mut total = 0.0f64;
        let mut daily_values = Vec::new();
        for i in 1..=days {
            let date = last_date + Duration::days(i);
            let trend_val = intercept + slope * (n as f64 + i as f64);
            let blended = 0.6 * ewma_base + 0.4 * trend_val;
            let dow = date.weekday().num_days_from_monday() as usize;
            let v = (blended * dow_factors[dow]).max(0.0);
            total += v;
            daily_values.push(serde_json::json!({ "date": date.to_string(), "value": v }));
        }

        let avg = values.iter().sum::<f64>() / n as f64;
        let std_dev = (values.iter().map(|v| (v - avg).powi(2)).sum::<f64>() / n as f64).sqrt();
        let margin = 1.96 * std_dev * (days as f64).sqrt();

        RevenueForecast {
            predicted: total,
            lower_bound: (total - margin).max(0.0),
            upper_bound: total + margin,
            confidence: r_squared.min(1.0).max(0.0),
            trend: classify_trend(slope),
            trend_slope: slope,
            daily_values,
        }
    }
}

impl Default for SimpleForecaster {
    fn default() -> Self { Self::new() }
}

// ── Statistical helpers ───────────────────────────────────────────────────

fn linear_regression(values: &[f64]) -> (f64, f64, f64) {
    let n = values.len() as f64;
    if n < 2.0 { return (0.0, values.first().copied().unwrap_or(0.0), 0.0); }

    let x_mean = (n - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / n;

    let ss_xx: f64 = (0..values.len()).map(|i| { let x = i as f64 - x_mean; x * x }).sum();
    let ss_xy: f64 = values.iter().enumerate().map(|(i, &y)| (i as f64 - x_mean) * (y - y_mean)).sum();

    let slope     = if ss_xx == 0.0 { 0.0 } else { ss_xy / ss_xx };
    let intercept = y_mean - slope * x_mean;

    let ss_res: f64 = values.iter().enumerate().map(|(i, &y)| (y - (intercept + slope * i as f64)).powi(2)).sum();
    let ss_tot: f64 = values.iter().map(|&y| (y - y_mean).powi(2)).sum();
    let r_squared   = if ss_tot == 0.0 { 1.0 } else { (1.0 - ss_res / ss_tot).max(0.0) };

    (slope, intercept, r_squared)
}

fn ewma(values: &[f64], alpha: f64) -> f64 {
    if values.is_empty() { return 0.0; }
    let mut result = values[0];
    for &v in &values[1..] { result = alpha * v + (1.0 - alpha) * result; }
    result
}

fn compute_dow_factors(historical: &[(NaiveDate, f64)]) -> [f64; 7] {
    let mut dow_sum   = [0.0f64; 7];
    let mut dow_count = [0u32;   7];
    for (date, qty) in historical {
        let dow = date.weekday().num_days_from_monday() as usize;
        dow_sum[dow]   += qty;
        dow_count[dow] += 1;
    }
    let overall_avg = historical.iter().map(|(_, q)| q).sum::<f64>() / historical.len() as f64;
    let mut factors = [1.0f64; 7];
    for i in 0..7 {
        if dow_count[i] > 0 && overall_avg > 0.0 {
            factors[i] = (dow_sum[i] / dow_count[i] as f64) / overall_avg;
        }
    }
    factors
}

fn classify_trend(slope: f64) -> TrendDirection {
    if slope > 0.05 { TrendDirection::Increasing }
    else if slope < -0.05 { TrendDirection::Decreasing }
    else { TrendDirection::Stable }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_regression_flat() {
        let (slope, intercept, r2) = linear_regression(&[5.0, 5.0, 5.0, 5.0, 5.0]);
        assert!((slope - 0.0).abs() < 1e-9);
        assert!((intercept - 5.0).abs() < 1e-9);
        assert!((r2 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn linear_regression_increasing() {
        let values: Vec<f64> = (0..10).map(|x| x as f64).collect();
        let (slope, _, r2) = linear_regression(&values);
        assert!((slope - 1.0).abs() < 1e-9);
        assert!((r2 - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ewma_single_value() {
        assert_eq!(ewma(&[42.0], 0.3), 42.0);
    }

    #[test]
    fn dow_factors_uniform() {
        let base = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        let hist: Vec<(NaiveDate, f64)> = (0..14).map(|i| (base + Duration::days(i), 10.0)).collect();
        for f in compute_dow_factors(&hist) {
            assert!((f - 1.0).abs() < 1e-9, "Expected 1.0, got {}", f);
        }
    }

    #[test]
    fn simple_forecaster_basic() {
        let base = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let hist: Vec<(NaiveDate, f64)> = (0..20).map(|i| (base + Duration::days(i), 5.0)).collect();
        let fc = SimpleForecaster::new();
        let result = fc.forecast_revenue(&hist, 7);
        assert!(result.predicted > 0.0);
        assert_eq!(result.daily_values.len(), 7);
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    }
}
