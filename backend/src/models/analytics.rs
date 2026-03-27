use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_revenue: Decimal,
    pub total_transactions: i64,
    pub total_items_sold: Decimal,
    pub average_transaction_value: Decimal,
    pub top_products: Vec<TopProduct>,
    pub daily_sales: Vec<DailySale>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TopProduct {
    pub product_id: Uuid,
    pub product_name: String,
    pub total_quantity: Decimal,
    pub total_revenue: Decimal,
    pub times_sold: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DailySale {
    pub date: NaiveDate,
    pub revenue: Decimal,
    pub transaction_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalesTrend {
    pub period: String,
    pub current_value: Decimal,
    pub previous_value: Decimal,
    pub change_percent: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsQueryParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub period: Option<String>, // day, week, month, year
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Sales summary for alerts
#[derive(Debug, Clone)]
pub struct SalesSummary {
    pub total_revenue: Decimal,
    pub total_transactions: i64,
    pub total_items_sold: Decimal,
}

/// Comparison result
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub period: String,
    pub current_revenue: Decimal,
    pub previous_revenue: Decimal,
    pub change_percent: f64,
}

/// Anomaly result
#[derive(Debug, Clone)]
pub struct AnomalyResult {
    pub date: chrono::NaiveDate,
    pub actual_value: Decimal,
    pub expected_value: Decimal,
    pub deviation_percent: f64,
}
