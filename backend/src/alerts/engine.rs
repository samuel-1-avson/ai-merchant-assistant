use std::sync::Arc;
use uuid::Uuid;
use chrono::Utc;
use sqlx::PgPool;
use rust_decimal::Decimal;
use tracing::{info, error};

use crate::alerts::{Alert, AlertType, AlertSeverity, AlertMetadata};
use crate::analytics::predictions::{PredictionEngine, AnomalyType, AnomalySeverity, RecommendedAction};
use crate::analytics::engine::AnalyticsEngine;

pub struct AlertEngine {
    pool: PgPool,
    prediction_engine: Arc<PredictionEngine>,
    analytics_engine: Arc<AnalyticsEngine>,
}

impl AlertEngine {
    pub fn new(pool: PgPool) -> Self {
        let prediction_engine = Arc::new(PredictionEngine::new(pool.clone()));
        let analytics_engine = Arc::new(AnalyticsEngine::new(pool.clone()));
        
        Self {
            pool,
            prediction_engine,
            analytics_engine,
        }
    }

    /// Check all alert conditions for a user
    pub async fn check_all_alerts(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let mut alerts = Vec::new();

        // Check inventory alerts
        match self.check_inventory_alerts(user_id).await {
            Ok(inventory_alerts) => alerts.extend(inventory_alerts),
            Err(e) => error!("Error checking inventory alerts: {}", e),
        }

        // Check sales anomaly alerts
        match self.check_sales_anomalies(user_id).await {
            Ok(anomaly_alerts) => alerts.extend(anomaly_alerts),
            Err(e) => error!("Error checking sales anomalies: {}", e),
        }

        // Check sales drop alerts
        match self.check_sales_drop(user_id).await {
            Ok(drop_alert) => {
                if let Some(alert) = drop_alert {
                    alerts.push(alert);
                }
            }
            Err(e) => error!("Error checking sales drop: {}", e),
        }

        // Check high demand alerts
        match self.check_high_demand(user_id).await {
            Ok(demand_alerts) => alerts.extend(demand_alerts),
            Err(e) => error!("Error checking high demand: {}", e),
        }

        // Save alerts to database
        for alert in &alerts {
            if let Err(e) = self.save_alert(alert).await {
                error!("Error saving alert: {}", e);
            }
        }

        info!("Generated {} alerts for user {}", alerts.len(), user_id);
        Ok(alerts)
    }

    /// Check for inventory-related alerts
    async fn check_inventory_alerts(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let recommendations = self.prediction_engine.get_inventory_recommendations(user_id).await?;
        let mut alerts = Vec::new();

        for rec in recommendations {
            let (alert_type, severity, title) = match rec.recommended_action {
                RecommendedAction::UrgentRestock => (
                    AlertType::OutOfStock,
                    AlertSeverity::Critical,
                    format!("Out of Stock: {}", rec.product_name),
                ),
                RecommendedAction::Restock => (
                    AlertType::LowStock,
                    AlertSeverity::Warning,
                    format!("Low Stock: {}", rec.product_name),
                ),
                RecommendedAction::ReduceStock => (
                    AlertType::InventoryRecommendation,
                    AlertSeverity::Info,
                    format!("Overstocked: {}", rec.product_name),
                ),
                _ => continue,
            };

            alerts.push(Alert {
                id: Uuid::new_v4(),
                user_id,
                alert_type,
                severity,
                title,
                message: rec.reason,
                metadata: AlertMetadata {
                    product_id: Some(rec.product_id),
                    value: Some(rec.current_stock as f64),
                    threshold: Some(rec.suggested_quantity as f64),
                    extra: serde_json::json!({
                        "suggested_quantity": rec.suggested_quantity,
                        "product_name": rec.product_name,
                    }),
                },
                is_read: false,
                read_at: None,
                created_at: Utc::now(),
            });
        }

        Ok(alerts)
    }

    /// Check for sales anomalies
    async fn check_sales_anomalies(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let anomalies = self.prediction_engine.detect_anomalies(user_id, 30).await?;
        let mut alerts = Vec::new();

        for anomaly in anomalies {
            let alert_type = match anomaly.anomaly_type {
                AnomalyType::Spike => AlertType::SalesSpike,
                AnomalyType::Drop => AlertType::SalesDrop,
            };

            let severity = match anomaly.severity {
                AnomalySeverity::High => AlertSeverity::Critical,
                AnomalySeverity::Medium => AlertSeverity::Warning,
                _ => AlertSeverity::Info,
            };

            let title = match anomaly.anomaly_type {
                AnomalyType::Spike => format!("Sales Spike Detected on {}", anomaly.date),
                AnomalyType::Drop => format!("Sales Drop Detected on {}", anomaly.date),
            };

            alerts.push(Alert {
                id: Uuid::new_v4(),
                user_id,
                alert_type,
                severity,
                title,
                message: format!(
                    "Revenue was ${:.2} (expected ${:.2}, deviation: {:.1}%)",
                    anomaly.actual_value,
                    anomaly.expected_value,
                    anomaly.deviation_percent
                ),
                metadata: AlertMetadata {
                    value: Some(anomaly.actual_value.to_f64().unwrap_or(0.0)),
                    threshold: Some(anomaly.expected_value.to_f64().unwrap_or(0.0)),
                    extra: serde_json::json!({
                        "deviation_percent": anomaly.deviation_percent,
                        "date": anomaly.date,
                    }),
                    ..Default::default()
                },
                is_read: false,
                read_at: None,
                created_at: Utc::now(),
            });
        }

        Ok(alerts)
    }

    /// Check for significant sales drop compared to previous period
    async fn check_sales_drop(&self, user_id: Uuid) -> anyhow::Result<Option<Alert>> {
        let comparison = self.analytics_engine.compare_periods(user_id, 7, 14).await?;

        // Alert if revenue dropped by more than 30%
        if comparison.revenue_change_percent < -30.0 {
            return Ok(Some(Alert {
                id: Uuid::new_v4(),
                user_id,
                alert_type: AlertType::SalesDrop,
                severity: AlertSeverity::Warning,
                title: "Significant Sales Drop Detected".to_string(),
                message: format!(
                    "Your revenue dropped by {:.1}% compared to last week. Current: ${:.2}, Previous: ${:.2}",
                    comparison.revenue_change_percent.abs(),
                    comparison.current_revenue,
                    comparison.previous_revenue
                ),
                metadata: AlertMetadata {
                    value: Some(comparison.current_revenue.to_f64().unwrap_or(0.0)),
                    threshold: Some(comparison.previous_revenue.to_f64().unwrap_or(0.0)),
                    extra: serde_json::json!({
                        "change_percent": comparison.revenue_change_percent,
                    }),
                    ..Default::default()
                },
                is_read: false,
                read_at: None,
                created_at: Utc::now(),
            }));
        }

        Ok(None)
    }

    /// Check for trending/high demand products
    async fn check_high_demand(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let top_products = self.analytics_engine.get_top_products(user_id, 10).await?;
        let mut alerts = Vec::new();

        for product in top_products.iter().take(3) {
            alerts.push(Alert {
                id: Uuid::new_v4(),
                user_id,
                alert_type: AlertType::HighDemand,
                severity: AlertSeverity::Info,
                title: format!("Trending: {}", product.product_name),
                message: format!(
                    "{} is trending with {} sales and ${:.2} in revenue",
                    product.product_name,
                    product.times_sold,
                    product.total_revenue
                ),
                metadata: AlertMetadata {
                    product_id: Some(product.product_id),
                    value: Some(product.total_revenue.to_f64().unwrap_or(0.0)),
                    extra: serde_json::json!({
                        "times_sold": product.times_sold,
                        "total_quantity": product.total_quantity.to_f64().unwrap_or(0.0),
                    }),
                    ..Default::default()
                },
                is_read: false,
                read_at: None,
                created_at: Utc::now(),
            });
        }

        Ok(alerts)
    }

    /// Generate daily summary alert
    pub async fn generate_daily_summary(&self, user_id: Uuid) -> anyhow::Result<Alert> {
        let summary = self.analytics_engine.get_summary(user_id, 1).await?;

        Ok(Alert {
            id: Uuid::new_v4(),
            user_id,
            alert_type: AlertType::DailySummary,
            severity: AlertSeverity::Info,
            title: "Daily Business Summary".to_string(),
            message: format!(
                "Today: {} transactions, ${:.2} revenue, {:.0} items sold",
                summary.total_transactions,
                summary.total_revenue,
                summary.total_items_sold
            ),
            metadata: AlertMetadata {
                value: Some(summary.total_revenue.to_f64().unwrap_or(0.0)),
                extra: serde_json::json!({
                    "transactions": summary.total_transactions,
                    "items_sold": summary.total_items_sold.to_f64().unwrap_or(0.0),
                    "avg_transaction": summary.average_transaction_value.to_f64().unwrap_or(0.0),
                }),
                ..Default::default()
            },
            is_read: false,
            read_at: None,
            created_at: Utc::now(),
        })
    }

    /// Save alert to database
    async fn save_alert(&self, alert: &Alert) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO alerts (id, user_id, alert_type, severity, title, message, metadata, is_read, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT (id) DO NOTHING
            "#,
            alert.id,
            alert.user_id,
            format!("{:?}", alert.alert_type),
            format!("{:?}", alert.severity),
            alert.title,
            alert.message,
            serde_json::to_value(&alert.metadata)?,
            alert.is_read,
            alert.created_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get unread alerts for a user
    pub async fn get_unread_alerts(&self, user_id: Uuid) -> anyhow::Result<Vec<Alert>> {
        let alerts = sqlx::query_as!(
            Alert,
            r#"
            SELECT 
                id,
                user_id,
                alert_type as "alert_type: _",
                severity as "severity: _",
                title,
                message,
                metadata as "metadata: _",
                is_read,
                read_at,
                created_at
            FROM alerts
            WHERE user_id = $1 AND is_read = false
            ORDER BY created_at DESC
            "#,
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(alerts)
    }

    /// Mark alert as read
    pub async fn mark_as_read(&self, alert_id: Uuid) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            UPDATE alerts 
            SET is_read = true, read_at = NOW()
            WHERE id = $1
            "#,
            alert_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
