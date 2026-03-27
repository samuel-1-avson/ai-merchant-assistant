//! Alert Engine - Monitors data and generates alerts

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use serde_json::Value;

use crate::models::{
    analytics::{SalesSummary, ComparisonResult, AnomalyResult},
    transaction::Transaction,
};
use crate::analytics::TrendDirection;

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AlertType {
    LowStock,
    SalesAnomaly,
    RevenueDrop,
    RevenueSpike,
    TopProduct,
    DailySummary,
    WeeklySummary,
}

/// Alert structure
#[derive(Debug, Clone)]
pub struct Alert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub metadata: Option<Value>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Alert engine
pub struct AlertEngine {
    pool: PgPool,
}

impl AlertEngine {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Check for low stock alerts
    pub async fn check_low_stock(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        // This would query products and check stock levels
        // For now, return empty
        Ok(vec![])
    }

    /// Generate sales anomaly alerts
    pub async fn check_sales_anomalies(&self, _user_id: Uuid, _anomalies: Vec<AnomalyResult>) -> anyhow::Result<Vec<Alert>> {
        // Convert anomalies to alerts
        Ok(vec![])
    }

    /// Generate revenue comparison alerts
    pub async fn check_revenue_changes(&self, _user_id: Uuid, _comparisons: Vec<ComparisonResult>) -> anyhow::Result<Vec<Alert>> {
        // Convert comparisons to alerts
        Ok(vec![])
    }

    /// Save alert to database (runtime query)
    async fn save_alert(&self, alert: &Alert) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO alerts (id, user_id, alert_type, severity, title, message, metadata, is_read, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#
        )
        .bind(alert.id)
        .bind(alert.user_id)
        .bind(format!("{:?}", alert.alert_type))
        .bind(format!("{:?}", alert.severity))
        .bind(&alert.title)
        .bind(&alert.message)
        .bind(alert.metadata.clone())
        .bind(alert.is_read)
        .bind(alert.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get unread alerts for a user (runtime query)
    pub async fn get_unread_alerts(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let alerts = sqlx::query_as::<_, Alert>(
            r#"
            SELECT 
                id,
                user_id,
                alert_type,
                severity,
                title,
                message,
                metadata,
                is_read,
                read_at,
                created_at
            FROM alerts
            WHERE user_id = $1 AND is_read = false
            ORDER BY created_at DESC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(alerts)
    }

    /// Mark alert as read (runtime query)
    pub async fn mark_as_read(&self, alert_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE alerts 
            SET is_read = true, read_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(alert_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// Implement SQLx FromRow for Alert
impl sqlx::FromRow<'_, sqlx::postgres::PgRow> for Alert {
    fn from_row(row: &sqlx::postgres::PgRow) -> sqlx::Result<Self> {
        use sqlx::Row;
        
        Ok(Alert {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            alert_type: AlertType::DailySummary, // Simplified - would parse from string
            severity: AlertSeverity::Info, // Simplified - would parse from string
            title: row.try_get("title")?,
            message: row.try_get("message")?,
            metadata: row.try_get("metadata")?,
            is_read: row.try_get("is_read")?,
            read_at: row.try_get("read_at")?,
            created_at: row.try_get("created_at")?,
        })
    }
}
