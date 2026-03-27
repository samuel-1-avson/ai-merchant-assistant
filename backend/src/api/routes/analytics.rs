use axum::{
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::AppState;

#[derive(Debug, serde::Deserialize)]
pub struct AnalyticsQuery {
    pub period: Option<String>, // day, week, month, year
    pub days: Option<i64>,
}

pub async fn summary(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let days = query.days.unwrap_or(7);

    // Mock summary - in production, use AnalyticsEngine
    let summary = json!({
        "total_revenue": 12500.00,
        "total_transactions": 450,
        "total_items_sold": 1280,
        "average_transaction_value": 27.78,
        "period": query.period.unwrap_or_else(|| "week".to_string()),
        "top_products": [
            {
                "product_id": "123e4567-e89b-12d3-a456-426614174010",
                "product_name": "Eggs",
                "total_quantity": 500,
                "total_revenue": 5000.00,
                "times_sold": 250
            },
            {
                "product_id": "123e4567-e89b-12d3-a456-426614174011",
                "product_name": "Milk",
                "total_quantity": 1000,
                "total_revenue": 3500.00,
                "times_sold": 200
            }
        ],
        "daily_sales": [
            { "date": "2024-01-10", "revenue": 2000.00, "transaction_count": 80 },
            { "date": "2024-01-11", "revenue": 3500.00, "transaction_count": 120 },
            { "date": "2024-01-12", "revenue": 3000.00, "transaction_count": 100 },
            { "date": "2024-01-13", "revenue": 2500.00, "transaction_count": 90 },
            { "date": "2024-01-14", "revenue": 1500.00, "transaction_count": 60 },
        ]
    });

    Ok(Json(json!({
        "success": true,
        "data": summary
    })))
}

pub async fn trends(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let trends = json!({
        "direction": "increasing",
        "slope": 0.15,
        "r_squared": 0.85,
        "forecast": [
            { "date": "2024-01-15", "value": 1800.00 },
            { "date": "2024-01-16", "value": 1950.00 },
            { "date": "2024-01-17", "value": 2100.00 },
            { "date": "2024-01-18", "value": 2250.00 },
            { "date": "2024-01-19", "value": 2400.00 },
            { "date": "2024-01-20", "value": 2550.00 },
            { "date": "2024-01-21", "value": 2700.00 },
        ]
    });

    Ok(Json(json!({
        "success": true,
        "data": trends
    })))
}

pub async fn forecast(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AnalyticsQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let forecast = json!({
        "predicted_revenue": 15000.00,
        "lower_bound": 12000.00,
        "upper_bound": 18000.00,
        "confidence": 0.85,
        "period": "next 7 days",
        "by_product": [
            {
                "product_id": "123e4567-e89b-12d3-a456-426614174010",
                "product_name": "Eggs",
                "forecasted_demand": [
                    { "date": "2024-01-15", "value": 50 },
                    { "date": "2024-01-16", "value": 55 },
                    { "date": "2024-01-17", "value": 60 },
                ],
                "confidence": 0.90
            }
        ]
    });

    Ok(Json(json!({
        "success": true,
        "data": forecast
    })))
}

pub async fn insights(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let insights = json!({
        "period": "last 7 days",
        "summary": "Your sales are trending upward with a 15% increase compared to last week. Eggs continue to be your best-selling product.",
        "recommendations": [
            "Consider increasing inventory for Eggs - demand is rising",
            "Best sales hours are 10 AM - 2 PM - consider promotions during slow periods",
            "Your average transaction value is below target - try bundling products",
        ],
        "alerts": [
            {
                "alert_type": "trending_product",
                "severity": "info",
                "message": "Eggs sales up 25% this week",
                "suggestion": "Increase stock levels"
            },
            {
                "alert_type": "inventory",
                "severity": "warning",
                "message": "Milk stock running low",
                "suggestion": "Reorder within 2 days"
            }
        ]
    });

    Ok(Json(json!({
        "success": true,
        "data": insights
    })))
}
