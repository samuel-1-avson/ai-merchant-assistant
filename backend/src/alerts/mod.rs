pub mod engine;
pub mod notifier;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub metadata: AlertMetadata,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    LowStock,
    OutOfStock,
    SalesDrop,
    SalesSpike,
    HighDemand,
    PriceAnomaly,
    DailySummary,
    WeeklySummary,
    InventoryRecommendation,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMetadata {
    pub product_id: Option<Uuid>,
    pub transaction_id: Option<Uuid>,
    pub value: Option<f64>,
    pub threshold: Option<f64>,
    pub extra: serde_json::Value,
}

impl Default for AlertMetadata {
    fn default() -> Self {
        Self {
            product_id: None,
            transaction_id: None,
            value: None,
            threshold: None,
            extra: serde_json::json!({}),
        }
    }
}
