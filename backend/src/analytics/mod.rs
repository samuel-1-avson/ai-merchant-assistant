pub mod engine;
pub mod predictions;
pub mod prophet;
pub mod customer;
pub mod pricing;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub date: NaiveDate,
    pub value: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub direction: TrendDirection,
    pub slope: f64,
    pub r_squared: f64,
    pub forecast: Vec<TimeSeriesPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessInsights {
    pub user_id: Uuid,
    pub period: String,
    pub summary: String,
    pub recommendations: Vec<String>,
    pub alerts: Vec<InsightAlert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightAlert {
    pub alert_type: String,
    pub severity: String,
    pub message: String,
    pub suggestion: String,
}
