use axum::{
    extract::{State, Query},
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::utils::errors::ApiError;
use crate::analytics::predictions::SimpleForecaster;

#[derive(Debug, serde::Deserialize)]
pub struct AnalyticsQuery {
    pub period: Option<String>, // day, week, month, year
    pub days: Option<i64>,
}

/// Get analytics summary for the authenticated user
pub async fn summary(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    let days = query.days.unwrap_or(7).min(90); // Max 90 days

    let summary = state
        .analytics_service
        .get_summary(user_id, days)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": summary,
        "meta": {
            "period_days": days,
            "period_label": query.period.unwrap_or_else(|| format!("last_{}_days", days))
        }
    })))
}

/// Get sales trends analysis
pub async fn trends(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    let days = query.days.unwrap_or(30);

    // Get transactions for trend analysis
    let transactions = state
        .analytics_service
        .get_transactions_for_analysis(user_id, days)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Calculate trend using simple linear regression
    let trend = calculate_trend(&transactions);

    // Generate forecast
    let forecaster = SimpleForecaster::new();
    let forecast = forecaster.forecast(&transactions, 7);

    Ok(Json(json!({
        "success": true,
        "data": {
            "direction": trend.direction,
            "slope": trend.slope,
            "r_squared": trend.r_squared,
            "forecast": forecast,
        },
        "meta": {
            "period_days": days,
            "forecast_days": 7
        }
    })))
}

/// Get demand forecast
pub async fn forecast(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    let days = query.days.unwrap_or(30);

    // Get historical data
    let transactions = state
        .analytics_service
        .get_transactions_for_analysis(user_id, days)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Calculate forecast
    let forecaster = SimpleForecaster::new();
    let revenue_forecast = forecaster.forecast_revenue(&transactions, 7);

    Ok(Json(json!({
        "success": true,
        "data": {
            "predicted_revenue": revenue_forecast.predicted,
            "lower_bound": revenue_forecast.lower_bound,
            "upper_bound": revenue_forecast.upper_bound,
            "confidence": revenue_forecast.confidence,
            "period": format!("next {} days", 7),
            "daily_forecast": revenue_forecast.daily_values,
        },
        "meta": {
            "historical_days": days,
            "forecast_days": 7
        }
    })))
}

/// Get AI-powered business insights
pub async fn insights(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    // Get summary for the last 7 days
    let summary = state
        .analytics_service
        .get_summary(user_id, 7)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Get transactions for the last 14 days to compare
    let current_period = state
        .analytics_service
        .get_summary(user_id, 7)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let previous_period = state
        .analytics_service
        .get_summary_for_period(user_id, 14, 7)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    // Generate insights
    let insights = generate_insights(&current_period, &previous_period, &summary.top_products);

    Ok(Json(json!({
        "success": true,
        "data": insights
    })))
}

// Helper structures and functions
#[derive(Debug)]
struct TrendResult {
    direction: String,
    slope: f64,
    r_squared: f64,
}

fn calculate_trend(transactions: &[(chrono::NaiveDate, f64)]) -> TrendResult {
    if transactions.len() < 2 {
        return TrendResult {
            direction: "stable".to_string(),
            slope: 0.0,
            r_squared: 0.0,
        };
    }

    // Simple linear regression
    let n = transactions.len() as f64;
    let sum_x: f64 = (0..transactions.len()).map(|i| i as f64).sum();
    let sum_y: f64 = transactions.iter().map(|(_, y)| y).sum();
    let sum_xy: f64 = transactions.iter().enumerate().map(|(i, (_, y))| i as f64 * y).sum();
    let sum_x2: f64 = (0..transactions.len()).map(|i| (i as f64).powi(2)).sum();

    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
    
    let direction = if slope > 0.01 {
        "increasing"
    } else if slope < -0.01 {
        "decreasing"
    } else {
        "stable"
    }.to_string();

    // Calculate R-squared (simplified)
    let mean_y = sum_y / n;
    let ss_tot: f64 = transactions.iter().map(|(_, y)| (y - mean_y).powi(2)).sum();
    let ss_res: f64 = transactions.iter().enumerate()
        .map(|(i, (_, y))| {
            let predicted = slope * i as f64 + (sum_y - slope * sum_x) / n;
            (y - predicted).powi(2)
        })
        .sum();
    
    let r_squared = if ss_tot > 0.0 {
        1.0 - (ss_res / ss_tot)
    } else {
        0.0
    };

    TrendResult {
        direction,
        slope,
        r_squared: r_squared.max(0.0).min(1.0),
    }
}

fn generate_insights(
    current: &crate::models::analytics::AnalyticsSummary,
    previous: &crate::models::analytics::AnalyticsSummary,
    top_products: &[crate::models::analytics::TopProduct],
) -> serde_json::Value {
    use rust_decimal::prelude::*;

    let revenue_change = if previous.total_revenue > Decimal::ZERO {
        ((current.total_revenue - previous.total_revenue) / previous.total_revenue)
            .to_f64()
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let mut recommendations: Vec<String> = vec![];
    let mut alerts = vec![];

    // Revenue trend insights
    if revenue_change > 0.1 {
        recommendations.push("Your sales are trending upward! Consider increasing inventory for popular items.".to_string());
    } else if revenue_change < -0.1 {
        recommendations.push("Sales have declined. Consider promotional activities or reviewing pricing.".to_string());
        alerts.push(serde_json::json!({
            "alert_type": "sales_decline",
            "severity": "warning",
            "message": format!("Sales down {:.1}% compared to last week", revenue_change.abs() * 100.0),
            "suggestion": "Review pricing or run promotions"
        }));
    }

    // Top product insights
    if let Some(top_product) = top_products.first() {
        recommendations.push(format!(
            "{} is your best-selling product. Ensure adequate stock levels.",
            top_product.product_name
        ));
    }

    // Transaction value insights
    let avg_transaction = if current.total_transactions > 0 {
        current.total_revenue / Decimal::from(current.total_transactions)
    } else {
        Decimal::ZERO
    };

    if avg_transaction < Decimal::from(20) {
        recommendations.push("Your average transaction value is below $20. Try bundling products or upselling.".to_string());
    }

    let summary = if revenue_change > 0.0 {
        format!(
            "Your sales are up {:.1}% compared to last week with {} transactions. Keep up the good work!",
            revenue_change * 100.0,
            current.total_transactions
        )
    } else {
        format!(
            "Your sales have decreased by {:.1}% compared to last week. Review your strategy to boost performance.",
            revenue_change.abs() * 100.0
        )
    };

    serde_json::json!({
        "period": "last 7 days",
        "summary": summary,
        "revenue_change_percent": revenue_change * 100.0,
        "recommendations": recommendations,
        "alerts": alerts,
    })
}
