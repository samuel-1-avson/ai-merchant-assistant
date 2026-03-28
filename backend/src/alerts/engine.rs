//! Alert Engine - Monitors data and generates alerts
//!
//! This module provides comprehensive alert detection for:
//! - Low stock warnings
//! - Sales anomalies (unusual drop/spike)
//! - Revenue changes (compared to previous periods)
//! - Daily/weekly summaries

use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration, Datelike};
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use serde_json::{json, Value};
use tracing::{info, warn, error};

use crate::models::analytics::{ComparisonResult, AnomalyResult};

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "lowercase")]
#[derive(serde::Serialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertSeverity::Info => write!(f, "info"),
            AlertSeverity::Warning => write!(f, "warning"),
            AlertSeverity::Critical => write!(f, "critical"),
        }
    }
}

/// Alert type
#[derive(Debug, Clone, Copy, PartialEq, sqlx::Type)]
#[sqlx(type_name = "text")]
#[sqlx(rename_all = "snake_case")]
#[derive(serde::Serialize)]
pub enum AlertType {
    LowStock,
    SalesAnomaly,
    RevenueDrop,
    RevenueSpike,
    TopProduct,
    DailySummary,
    WeeklySummary,
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::LowStock => write!(f, "low_stock"),
            AlertType::SalesAnomaly => write!(f, "sales_anomaly"),
            AlertType::RevenueDrop => write!(f, "revenue_drop"),
            AlertType::RevenueSpike => write!(f, "revenue_spike"),
            AlertType::TopProduct => write!(f, "top_product"),
            AlertType::DailySummary => write!(f, "daily_summary"),
            AlertType::WeeklySummary => write!(f, "weekly_summary"),
        }
    }
}

/// Alert structure
#[derive(Debug, Clone, serde::Serialize)]
pub struct Alert {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(rename = "type")]
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub metadata: Option<Value>,
    pub is_read: bool,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Alert configuration
#[derive(Debug, Clone)]
pub struct AlertConfig {
    /// Low stock warning threshold (default: 10)
    pub low_stock_threshold: i32,
    /// Critical stock threshold (default: 0)
    pub critical_stock_threshold: i32,
    /// Revenue drop alert threshold % (default: 20%)
    pub revenue_drop_threshold: f64,
    /// Revenue spike alert threshold % (default: 50%)
    pub revenue_spike_threshold: f64,
    /// Sales anomaly threshold % (default: 30%)
    pub sales_anomaly_threshold: f64,
    /// Minimum transactions for anomaly detection (default: 5)
    pub min_transactions_for_anomaly: i64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            low_stock_threshold: 10,
            critical_stock_threshold: 0,
            revenue_drop_threshold: 20.0,
            revenue_spike_threshold: 50.0,
            sales_anomaly_threshold: 30.0,
            min_transactions_for_anomaly: 5,
        }
    }
}

/// Alert engine
pub struct AlertEngine {
    pool: PgPool,
    config: AlertConfig,
}

impl AlertEngine {
    pub fn new(pool: PgPool) -> Self {
        Self::with_config(pool, AlertConfig::default())
    }

    pub fn with_config(pool: PgPool, config: AlertConfig) -> Self {
        Self { pool, config }
    }

    /// Run all alert checks for a user
    pub async fn check_all(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let mut all_alerts = Vec::new();

        // Check low stock
        match self.check_low_stock(user_id).await {
            Ok(alerts) => all_alerts.extend(alerts),
            Err(e) => warn!("Low stock check failed: {}", e),
        }

        // Check revenue changes
        match self.check_revenue_changes(user_id).await {
            Ok(alerts) => all_alerts.extend(alerts),
            Err(e) => warn!("Revenue change check failed: {}", e),
        }

        // Check sales anomalies
        match self.check_sales_anomalies(user_id).await {
            Ok(alerts) => all_alerts.extend(alerts),
            Err(e) => warn!("Sales anomaly check failed: {}", e),
        }

        // Save all alerts
        for alert in &all_alerts {
            if let Err(e) = self.save_alert(alert).await {
                error!("Failed to save alert: {}", e);
            }
        }

        info!("Generated {} alerts for user {}", all_alerts.len(), user_id);
        Ok(all_alerts)
    }

    /// Check for low stock alerts
    pub async fn check_low_stock(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let rows = sqlx::query(
            r#"
            SELECT 
                id,
                name,
                stock_quantity,
                low_stock_threshold,
                unit
            FROM products
            WHERE user_id = $1
            AND is_active = true
            AND stock_quantity <= low_stock_threshold
            ORDER BY stock_quantity ASC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let mut alerts = Vec::new();

        for row in rows {
            let product_id: Uuid = row.try_get("id")?;
            let product_name: String = row.try_get("name")?;
            let stock_quantity: i32 = row.try_get("stock_quantity")?;
            let low_threshold: i32 = row.try_get("low_stock_threshold")?;
            let unit: String = row.try_get("unit")?;

            let (severity, title, message) = if stock_quantity == 0 {
                (
                    AlertSeverity::Critical,
                    format!("Out of Stock: {}", product_name),
                    format!(
                        "{} is completely out of stock. Please restock immediately to avoid losing sales.",
                        product_name
                    ),
                )
            } else if stock_quantity <= self.config.critical_stock_threshold {
                (
                    AlertSeverity::Critical,
                    format!("Critical Stock: {}", product_name),
                    format!(
                        "{} has only {} {} remaining. Critical threshold reached!",
                        product_name, stock_quantity, unit
                    ),
                )
            } else {
                (
                    AlertSeverity::Warning,
                    format!("Low Stock: {}", product_name),
                    format!(
                        "{} has {} {} remaining (below threshold of {})",
                        product_name, stock_quantity, unit, low_threshold
                    ),
                )
            };

            // Check if alert already exists and is unread
            if !self.alert_exists(user_id, AlertType::LowStock, &product_id.to_string()).await? {
                alerts.push(Alert {
                    id: Uuid::new_v4(),
                    user_id,
                    alert_type: AlertType::LowStock,
                    severity,
                    title,
                    message,
                    metadata: Some(json!({
                        "product_id": product_id,
                        "product_name": product_name,
                        "current_stock": stock_quantity,
                        "threshold": low_threshold,
                        "unit": unit
                    })),
                    is_read: false,
                    read_at: None,
                    created_at: Utc::now(),
                });
            }
        }

        Ok(alerts)
    }

    /// Check for sales anomalies (unusual patterns)
    pub async fn check_sales_anomalies(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let mut alerts = Vec::new();

        // Get daily sales for the past 14 days
        let rows = sqlx::query(
            r#"
            SELECT 
                sale_date,
                total_revenue,
                transaction_count
            FROM mv_daily_sales_summary
            WHERE user_id = $1
            AND sale_date >= CURRENT_DATE - INTERVAL '14 days'
            ORDER BY sale_date ASC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.len() < 7 {
            // Not enough data for anomaly detection
            return Ok(alerts);
        }

        // Parse historical data
        let revenues: Vec<f64> = rows
            .iter()
            .filter_map(|r| r.try_get::<f64, _>("total_revenue").ok())
            .collect();

        if revenues.len() < 7 {
            return Ok(alerts);
        }

        // Calculate statistics
        let avg_revenue = revenues.iter().sum::<f64>() / revenues.len() as f64;
        let variance = revenues
            .iter()
            .map(|r| (r - avg_revenue).powi(2))
            .sum::<f64>() / revenues.len() as f64;
        let std_dev = variance.sqrt();

        // Check most recent day
        if let Some(row) = rows.last() {
            let date: chrono::NaiveDate = row.try_get("sale_date")?;
            let revenue: f64 = row.try_get("total_revenue")?;
            let transaction_count: i64 = row.try_get("transaction_count")?;

            // Skip if not enough transactions
            if transaction_count < self.config.min_transactions_for_anomaly {
                return Ok(alerts);
            }

            let deviation = ((revenue - avg_revenue) / avg_revenue) * 100.0;
            let z_score = (revenue - avg_revenue) / std_dev;

            // Detect significant drop
            if deviation <= -self.config.sales_anomaly_threshold && z_score < -1.5 {
                alerts.push(Alert {
                    id: Uuid::new_v4(),
                    user_id,
                    alert_type: AlertType::SalesAnomaly,
                    severity: AlertSeverity::Warning,
                    title: "Sales Drop Detected".to_string(),
                    message: format!(
                        "Yesterday's sales (${:.2}) were {:.1}% below your 14-day average (${:.2}).",
                        revenue, -deviation, avg_revenue
                    ),
                    metadata: Some(json!({
                        "date": date.to_string(),
                        "actual_revenue": revenue,
                        "expected_revenue": avg_revenue,
                        "deviation_percent": deviation,
                        "z_score": z_score,
                        "transaction_count": transaction_count
                    })),
                    is_read: false,
                    read_at: None,
                    created_at: Utc::now(),
                });
            }

            // Detect significant spike
            if deviation >= self.config.sales_anomaly_threshold && z_score > 1.5 {
                alerts.push(Alert {
                    id: Uuid::new_v4(),
                    user_id,
                    alert_type: AlertType::SalesAnomaly,
                    severity: AlertSeverity::Info,
                    title: "Sales Spike Detected".to_string(),
                    message: format!(
                        "Great news! Yesterday's sales (${:.2}) were {:.1}% above your 14-day average (${:.2}).",
                        revenue, deviation, avg_revenue
                    ),
                    metadata: Some(json!({
                        "date": date.to_string(),
                        "actual_revenue": revenue,
                        "expected_revenue": avg_revenue,
                        "deviation_percent": deviation,
                        "z_score": z_score,
                        "transaction_count": transaction_count
                    })),
                    is_read: false,
                    read_at: None,
                    created_at: Utc::now(),
                });
            }
        }

        Ok(alerts)
    }

    /// Check for revenue changes compared to previous period
    pub async fn check_revenue_changes(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let mut alerts = Vec::new();

        // Compare this week vs last week
        let rows = sqlx::query(
            r#"
            WITH weekly_data AS (
                SELECT 
                    CASE 
                        WHEN sale_date >= CURRENT_DATE - INTERVAL '7 days' THEN 'current'
                        WHEN sale_date >= CURRENT_DATE - INTERVAL '14 days' THEN 'previous'
                    END as week_period,
                    SUM(total_revenue) as weekly_revenue,
                    SUM(transaction_count) as weekly_transactions
                FROM mv_daily_sales_summary
                WHERE user_id = $1
                AND sale_date >= CURRENT_DATE - INTERVAL '14 days'
                GROUP BY week_period
            )
            SELECT 
                MAX(CASE WHEN week_period = 'current' THEN weekly_revenue END) as current_revenue,
                MAX(CASE WHEN week_period = 'previous' THEN weekly_revenue END) as previous_revenue,
                MAX(CASE WHEN week_period = 'current' THEN weekly_transactions END) as current_transactions,
                MAX(CASE WHEN week_period = 'previous' THEN weekly_transactions END) as previous_transactions
            FROM weekly_data
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        let current_revenue: Option<f64> = rows.try_get("current_revenue")?;
        let previous_revenue: Option<f64> = rows.try_get("previous_revenue")?;
        let current_tx: Option<i64> = rows.try_get("current_transactions")?;

        if let (Some(current), Some(previous)) = (current_revenue, previous_revenue) {
            if previous > 0.0 {
                let change_percent = ((current - previous) / previous) * 100.0;

                // Revenue drop alert
                if change_percent <= -self.config.revenue_drop_threshold {
                    alerts.push(Alert {
                        id: Uuid::new_v4(),
                        user_id,
                        alert_type: AlertType::RevenueDrop,
                        severity: AlertSeverity::Warning,
                        title: "Weekly Revenue Drop".to_string(),
                        message: format!(
                            "This week's revenue (${:.2}) is down {:.1}% compared to last week (${:.2}).",
                            current, -change_percent, previous
                        ),
                        metadata: Some(json!({
                            "period": "weekly",
                            "current_revenue": current,
                            "previous_revenue": previous,
                            "change_percent": change_percent,
                            "current_transactions": current_tx
                        })),
                        is_read: false,
                        read_at: None,
                        created_at: Utc::now(),
                    });
                }

                // Revenue spike alert (positive)
                if change_percent >= self.config.revenue_spike_threshold {
                    alerts.push(Alert {
                        id: Uuid::new_v4(),
                        user_id,
                        alert_type: AlertType::RevenueSpike,
                        severity: AlertSeverity::Info,
                        title: "Weekly Revenue Increase".to_string(),
                        message: format!(
                            "Excellent! This week's revenue (${:.2}) is up {:.1}% compared to last week (${:.2}).",
                            current, change_percent, previous
                        ),
                        metadata: Some(json!({
                            "period": "weekly",
                            "current_revenue": current,
                            "previous_revenue": previous,
                            "change_percent": change_percent,
                            "current_transactions": current_tx
                        })),
                        is_read: false,
                        read_at: None,
                        created_at: Utc::now(),
                    });
                }
            }
        }

        Ok(alerts)
    }

    /// Generate daily summary alert
    pub async fn generate_daily_summary(&self, user_id: Uuid) -> anyhow::Result<Option<Alert>> {
        let row = sqlx::query(
            r#"
            SELECT 
                SUM(total_revenue) as daily_revenue,
                SUM(transaction_count) as daily_transactions,
                SUM(total_items_sold) as daily_items
            FROM mv_daily_sales_summary
            WHERE user_id = $1
            AND sale_date = CURRENT_DATE - INTERVAL '1 day'
            GROUP BY user_id
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let revenue: f64 = row.try_get::<f64, _>("daily_revenue").unwrap_or(0.0);
            let transactions: i64 = row.try_get::<i64, _>("daily_transactions").unwrap_or(0);
            let items: f64 = row.try_get::<f64, _>("daily_items").unwrap_or(0.0);

            if transactions > 0 {
                return Ok(Some(Alert {
                    id: Uuid::new_v4(),
                    user_id,
                    alert_type: AlertType::DailySummary,
                    severity: AlertSeverity::Info,
                    title: "Daily Sales Summary".to_string(),
                    message: format!(
                        "Yesterday you made {} transactions totaling ${:.2} ({} items sold).",
                        transactions, revenue, items
                    ),
                    metadata: Some(json!({
                        "date": (Utc::now() - Duration::days(1)).format("%Y-%m-%d").to_string(),
                        "revenue": revenue,
                        "transactions": transactions,
                        "items_sold": items
                    })),
                    is_read: false,
                    read_at: None,
                    created_at: Utc::now(),
                }));
            }
        }

        Ok(None)
    }

    /// Generate weekly summary alert
    pub async fn generate_weekly_summary(&self, user_id: Uuid) -> anyhow::Result<Option<Alert>> {
        let row = sqlx::query(
            r#"
            SELECT 
                SUM(total_revenue) as weekly_revenue,
                SUM(transaction_count) as weekly_transactions,
                SUM(total_items_sold) as weekly_items
            FROM mv_daily_sales_summary
            WHERE user_id = $1
            AND sale_date >= CURRENT_DATE - INTERVAL '7 days'
            GROUP BY user_id
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let revenue: f64 = row.try_get::<f64, _>("weekly_revenue").unwrap_or(0.0);
            let transactions: i64 = row.try_get::<i64, _>("weekly_transactions").unwrap_or(0);
            let items: f64 = row.try_get::<f64, _>("weekly_items").unwrap_or(0.0);

            if transactions > 0 {
                return Ok(Some(Alert {
                    id: Uuid::new_v4(),
                    user_id,
                    alert_type: AlertType::WeeklySummary,
                    severity: AlertSeverity::Info,
                    title: "Weekly Sales Summary".to_string(),
                    message: format!(
                        "This week you made {} transactions totaling ${:.2} ({} items sold).",
                        transactions, revenue, items
                    ),
                    metadata: Some(json!({
                        "week_start": (Utc::now() - Duration::days(7)).format("%Y-%m-%d").to_string(),
                        "week_end": Utc::now().format("%Y-%m-%d").to_string(),
                        "revenue": revenue,
                        "transactions": transactions,
                        "items_sold": items
                    })),
                    is_read: false,
                    read_at: None,
                    created_at: Utc::now(),
                }));
            }
        }

        Ok(None)
    }

    /// Check if a similar alert already exists and is unread
    async fn alert_exists(&self, user_id: Uuid, alert_type: AlertType, entity_id: &str) -> anyhow::Result<bool> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM alerts
            WHERE user_id = $1
            AND alert_type = $2
            AND metadata->>'product_id' = $3
            AND is_read = false
            AND created_at > NOW() - INTERVAL '24 hours'
            "#
        )
        .bind(user_id)
        .bind(alert_type.to_string())
        .bind(entity_id)
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.try_get("count")?;
        Ok(count > 0)
    }

    /// Save alert to database
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
        .bind(alert.alert_type.to_string())
        .bind(alert.severity.to_string())
        .bind(&alert.title)
        .bind(&alert.message)
        .bind(alert.metadata.clone())
        .bind(alert.is_read)
        .bind(alert.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all alerts for a user
    pub async fn get_alerts(&self, user_id: Uuid, unread_only: bool) -> anyhow::Result<Vec<Alert>> {
        let query = if unread_only {
            r#"
            SELECT 
                id, user_id, alert_type, severity, title, message, metadata,
                is_read, read_at, created_at
            FROM alerts
            WHERE user_id = $1 AND is_read = false
            ORDER BY created_at DESC
            "#
        } else {
            r#"
            SELECT 
                id, user_id, alert_type, severity, title, message, metadata,
                is_read, read_at, created_at
            FROM alerts
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT 50
            "#
        };

        let rows = sqlx::query(query)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;

        let alerts = rows.into_iter().map(|row| {
            Alert {
                id: row.try_get("id").unwrap_or_default(),
                user_id: row.try_get("user_id").unwrap_or_default(),
                alert_type: AlertType::LowStock, // Simplified parsing
                severity: AlertSeverity::Info,     // Simplified parsing
                title: row.try_get("title").unwrap_or_default(),
                message: row.try_get("message").unwrap_or_default(),
                metadata: row.try_get("metadata").ok(),
                is_read: row.try_get("is_read").unwrap_or(false),
                read_at: row.try_get("read_at").ok(),
                created_at: row.try_get("created_at").unwrap_or_else(|_| Utc::now()),
            }
        }).collect();

        Ok(alerts)
    }

    /// Mark alert as read
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

    /// Mark all alerts as read for a user
    pub async fn mark_all_as_read(&self, user_id: Uuid) -> anyhow::Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE alerts 
            SET is_read = true, read_at = NOW()
            WHERE user_id = $1 AND is_read = false
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Get alert counts by severity
    pub async fn get_alert_counts(&self, user_id: Uuid) -> anyhow::Result<AlertCounts> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE is_read = false) as total_unread,
                COUNT(*) FILTER (WHERE is_read = false AND severity = 'critical') as critical,
                COUNT(*) FILTER (WHERE is_read = false AND severity = 'warning') as warning,
                COUNT(*) FILTER (WHERE is_read = false AND severity = 'info') as info
            FROM alerts
            WHERE user_id = $1
            AND created_at > NOW() - INTERVAL '7 days'
            "#
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(AlertCounts {
            total_unread: row.try_get::<i64, _>("total_unread").unwrap_or(0),
            critical: row.try_get::<i64, _>("critical").unwrap_or(0),
            warning: row.try_get::<i64, _>("warning").unwrap_or(0),
            info: row.try_get::<i64, _>("info").unwrap_or(0),
        })
    }
}

/// Alert count summary
#[derive(Debug, Clone)]
pub struct AlertCounts {
    pub total_unread: i64,
    pub critical: i64,
    pub warning: i64,
    pub info: i64,
}
