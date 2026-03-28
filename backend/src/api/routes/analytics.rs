use axum::{
    extract::{State, Query, Extension},
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};

use rust_decimal::Decimal;
use crate::api::state::AppState;
use crate::api::middleware::AuthUser;
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
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

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
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

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
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

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
    Extension(auth_user): Extension<AuthUser>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    // Fetch current + previous period and product performance in parallel
    let (current_result, previous_result, perf_result) = tokio::join!(
        state.analytics_service.get_summary(user_id, 7),
        state.analytics_service.get_summary_for_period(user_id, 14, 7),
        state.analytics_service.get_product_performance(user_id, 30),
    );

    let current = current_result.map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let previous = previous_result.map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    let products = perf_result.map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let insights = generate_insights(&current, &previous, &products);

    Ok(Json(json!({
        "success": true,
        "data": insights
    })))
}

/// GET /api/v1/analytics/products — per-product performance for a period
pub async fn product_performance(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    let days = query.days.unwrap_or(30).min(90);

    let products = state
        .analytics_service
        .get_product_performance(user_id, days)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    let top_sellers: Vec<_> = products.iter().filter(|p| p.performance_label == "top_seller").collect();
    let good: Vec<_> = products.iter().filter(|p| p.performance_label == "good").collect();
    let slow: Vec<_> = products.iter().filter(|p| p.performance_label == "slow_mover").collect();
    let no_sales: Vec<_> = products.iter().filter(|p| p.performance_label == "no_sales").collect();

    Ok(Json(json!({
        "success": true,
        "data": {
            "period_days": days,
            "products": products,
            "summary": {
                "top_sellers": top_sellers.len(),
                "good_performers": good.len(),
                "slow_movers": slow.len(),
                "no_sales": no_sales.len(),
                "total_products": products.len(),
            }
        }
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
    products: &[crate::models::analytics::ProductPerformance],
) -> serde_json::Value {
    use rust_decimal::prelude::ToPrimitive;

    let revenue_change_pct = if previous.total_revenue > Decimal::ZERO {
        ((current.total_revenue - previous.total_revenue) / previous.total_revenue)
            .to_f64()
            .unwrap_or(0.0)
            * 100.0
    } else {
        0.0
    };

    let revenue = current.total_revenue.to_f64().unwrap_or(0.0);
    let avg_tx = current.average_transaction_value.to_f64().unwrap_or(0.0);
    let daily_avg = revenue / 7.0;
    let tx_per_day = current.total_transactions as f64 / 7.0;

    // ── Business health score (0-100) ──────────────────────────────────────
    // Weighted from: revenue trend (40), transaction volume (30), avg tx value (30)
    let trend_score = ((revenue_change_pct + 50.0) / 100.0 * 40.0).clamp(0.0, 40.0);
    let volume_score = (tx_per_day / 10.0 * 30.0).clamp(0.0, 30.0); // 10 tx/day = full marks
    let avg_tx_score = (avg_tx / 50.0 * 30.0).clamp(0.0, 30.0);     // $50 avg = full marks
    let health_score = (trend_score + volume_score + avg_tx_score).round() as u32;

    let health_label = match health_score {
        80..=100 => "excellent",
        60..=79  => "good",
        40..=59  => "fair",
        _        => "needs_attention",
    };

    // ── Product classification ──────────────────────────────────────────────
    let top_sellers: Vec<_> = products.iter()
        .filter(|p| p.performance_label == "top_seller")
        .take(3)
        .collect();
    let slow_movers: Vec<_> = products.iter()
        .filter(|p| p.performance_label == "slow_mover")
        .take(5)
        .collect();
    let no_sales: Vec<_> = products.iter()
        .filter(|p| p.performance_label == "no_sales")
        .take(5)
        .collect();

    // ── Profitability snapshot ─────────────────────────────────────────────
    let products_with_margin: Vec<_> = products.iter()
        .filter(|p| p.profit_margin_pct.is_some() && p.times_sold > 0)
        .collect();
    let avg_margin_pct = if !products_with_margin.is_empty() {
        let sum: f64 = products_with_margin.iter()
            .filter_map(|p| p.profit_margin_pct)
            .sum();
        Some(sum / products_with_margin.len() as f64)
    } else {
        None
    };

    // ── Recommendations ────────────────────────────────────────────────────
    let mut recommendations: Vec<serde_json::Value> = vec![];

    if revenue_change_pct > 10.0 {
        recommendations.push(json!({
            "type": "stock",
            "priority": "high",
            "message": format!("Sales are up {:.1}% — ensure top sellers are well-stocked to avoid lost sales.", revenue_change_pct),
        }));
    } else if revenue_change_pct < -10.0 {
        recommendations.push(json!({
            "type": "pricing",
            "priority": "high",
            "message": format!("Revenue dropped {:.1}% vs last week. Consider a promotion or reviewing prices.", revenue_change_pct.abs()),
        }));
    }

    if avg_tx < 20.0 && current.total_transactions > 0 {
        recommendations.push(json!({
            "type": "upsell",
            "priority": "medium",
            "message": format!("Average transaction is ${:.2}. Bundle slow-movers with top sellers to increase order size.", avg_tx),
        }));
    }

    if !top_sellers.is_empty() {
        recommendations.push(json!({
            "type": "stock",
            "priority": "medium",
            "message": format!("{} is your best performer. Keep stock levels high.", top_sellers[0].product_name),
        }));
    }

    if !slow_movers.is_empty() {
        let names: Vec<&str> = slow_movers.iter().map(|p| p.product_name.as_str()).take(3).collect();
        recommendations.push(json!({
            "type": "pricing",
            "priority": "low",
            "message": format!("Slow-moving products: {}. Consider discounting or bundling these.", names.join(", ")),
        }));
    }

    if !no_sales.is_empty() {
        recommendations.push(json!({
            "type": "catalogue",
            "priority": "low",
            "message": format!("{} product(s) had zero sales in 30 days. Review if they should still be listed.", no_sales.len()),
        }));
    }

    // ── Alerts ─────────────────────────────────────────────────────────────
    let mut alerts: Vec<serde_json::Value> = vec![];

    if revenue_change_pct < -20.0 {
        alerts.push(json!({
            "type": "revenue_drop",
            "severity": "critical",
            "message": format!("Revenue is down {:.1}% compared to last week.", revenue_change_pct.abs()),
        }));
    } else if revenue_change_pct < -10.0 {
        alerts.push(json!({
            "type": "revenue_drop",
            "severity": "warning",
            "message": format!("Revenue is down {:.1}% compared to last week.", revenue_change_pct.abs()),
        }));
    }

    // ── Summary sentence ───────────────────────────────────────────────────
    let summary = if current.total_transactions == 0 {
        "No transactions recorded yet. Start logging sales with the voice recorder.".to_string()
    } else if revenue_change_pct > 0.0 {
        format!(
            "Sales are up {:.1}% vs last week — ${:.2} across {} transactions (avg ${:.2}/sale, ${:.2}/day).",
            revenue_change_pct, revenue, current.total_transactions, avg_tx, daily_avg,
        )
    } else {
        format!(
            "Sales are down {:.1}% vs last week — ${:.2} across {} transactions (avg ${:.2}/sale, ${:.2}/day).",
            revenue_change_pct.abs(), revenue, current.total_transactions, avg_tx, daily_avg,
        )
    };

    json!({
        "period": "last 7 days",
        "summary": summary,
        "health_score": health_score,
        "health_label": health_label,
        "revenue": revenue,
        "revenue_change_percent": revenue_change_pct,
        "average_transaction_value": avg_tx,
        "average_daily_revenue": daily_avg,
        "transactions_per_day": tx_per_day,
        "average_profit_margin_pct": avg_margin_pct,
        "top_sellers": top_sellers,
        "slow_movers": slow_movers,
        "no_sales_products": no_sales,
        "recommendations": recommendations,
        "alerts": alerts,
    })
}
