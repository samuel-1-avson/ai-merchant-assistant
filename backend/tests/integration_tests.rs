//! Integration tests for AI Merchant Assistant Backend

use ai_merchant_backend::models::transaction::ExtractedEntities;
use ai_merchant_backend::analytics::{TrendDirection, TimeSeriesPoint};
use rust_decimal::Decimal;
use chrono::NaiveDate;

/// Test entity extraction data structures
#[test]
fn test_extracted_entities_default() {
    let entities = ExtractedEntities {
        product: Some("Eggs".to_string()),
        quantity: Some(2.0),
        unit: Some("crate".to_string()),
        price: Some(10.0),
        currency: Some("USD".to_string()),
    };

    assert_eq!(entities.product, Some("Eggs".to_string()));
    assert_eq!(entities.quantity, Some(2.0));
    assert_eq!(entities.price, Some(10.0));
}

/// Test trend direction enum
#[test]
fn test_trend_direction_serialization() {
    let increasing = TrendDirection::Increasing;
    let decreasing = TrendDirection::Decreasing;
    let stable = TrendDirection::Stable;

    // Test serialization
    let inc_json = serde_json::to_string(&increasing).unwrap();
    let dec_json = serde_json::to_string(&decreasing).unwrap();
    let stab_json = serde_json::to_string(&stable).unwrap();

    assert!(inc_json.contains("Increasing"));
    assert!(dec_json.contains("Decreasing"));
    assert!(stab_json.contains("Stable"));
}

/// Test time series point creation
#[test]
fn test_time_series_point() {
    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let value = Decimal::from(100);
    
    let point = TimeSeriesPoint { date, value };
    
    assert_eq!(point.date.to_string(), "2024-01-15");
    assert_eq!(point.value, Decimal::from(100));
}

/// Test decimal calculations for financial data
#[test]
fn test_decimal_calculations() {
    let price = Decimal::from(10);
    let quantity = Decimal::from(5);
    let total = price * quantity;
    
    assert_eq!(total, Decimal::from(50));
}

/// Test JSON serialization of entities
#[test]
fn test_entities_json_serialization() {
    let entities = ExtractedEntities {
        product: Some("Milk".to_string()),
        quantity: Some(3.0),
        unit: Some("liter".to_string()),
        price: Some(4.50),
        currency: Some("USD".to_string()),
    };

    let json = serde_json::to_string(&entities).unwrap();
    let deserialized: ExtractedEntities = serde_json::from_str(&json).unwrap();

    assert_eq!(entities.product, deserialized.product);
    assert_eq!(entities.quantity, deserialized.quantity);
    assert_eq!(entities.price, deserialized.price);
}
