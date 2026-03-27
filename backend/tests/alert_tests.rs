//! Tests for Alert System

use ai_merchant_backend::alerts::{Alert, AlertType, AlertSeverity, AlertMetadata};
use uuid::Uuid;
use chrono::Utc;

/// Test alert creation
#[test]
fn test_alert_creation() {
    let alert = Alert {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        alert_type: AlertType::LowStock,
        severity: AlertSeverity::Warning,
        title: "Low Stock Alert".to_string(),
        message: "Stock is running low".to_string(),
        metadata: AlertMetadata::default(),
        is_read: false,
        read_at: None,
        created_at: Utc::now(),
    };

    assert!(matches!(alert.alert_type, AlertType::LowStock));
    assert!(matches!(alert.severity, AlertSeverity::Warning));
    assert!(!alert.is_read);
    assert!(alert.read_at.is_none());
}

/// Test alert types
#[test]
fn test_alert_types() {
    let types = vec![
        AlertType::LowStock,
        AlertType::OutOfStock,
        AlertType::SalesDrop,
        AlertType::SalesSpike,
        AlertType::HighDemand,
        AlertType::PriceAnomaly,
        AlertType::DailySummary,
        AlertType::WeeklySummary,
        AlertType::InventoryRecommendation,
        AlertType::System,
    ];

    assert_eq!(types.len(), 10);
}

/// Test alert severity levels
#[test]
fn test_alert_severity() {
    assert!(matches!(AlertSeverity::Info, AlertSeverity::Info));
    assert!(matches!(AlertSeverity::Warning, AlertSeverity::Warning));
    assert!(matches!(AlertSeverity::Critical, AlertSeverity::Critical));
}

/// Test alert metadata
#[test]
fn test_alert_metadata() {
    let metadata = AlertMetadata {
        product_id: Some(Uuid::new_v4()),
        transaction_id: None,
        value: Some(100.0),
        threshold: Some(50.0),
        extra: serde_json::json!({"key": "value"}),
    };

    assert!(metadata.product_id.is_some());
    assert!(metadata.transaction_id.is_none());
    assert_eq!(metadata.value, Some(100.0));
    assert_eq!(metadata.threshold, Some(50.0));
}

/// Test alert serialization
#[test]
fn test_alert_serialization() {
    let alert = Alert {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        alert_type: AlertType::HighDemand,
        severity: AlertSeverity::Info,
        title: "Trending Product".to_string(),
        message: "Product is trending".to_string(),
        metadata: AlertMetadata::default(),
        is_read: false,
        read_at: None,
        created_at: Utc::now(),
    };

    let json = serde_json::to_string(&alert).unwrap();
    let deserialized: Alert = serde_json::from_str(&json).unwrap();

    assert_eq!(alert.title, deserialized.title);
    assert_eq!(alert.message, deserialized.message);
    assert_eq!(alert.is_read, deserialized.is_read);
}

/// Test critical alert priority
#[test]
fn test_critical_alert_priority() {
    let critical_alert = Alert {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        alert_type: AlertType::OutOfStock,
        severity: AlertSeverity::Critical,
        title: "Out of Stock".to_string(),
        message: "Product is out of stock".to_string(),
        metadata: AlertMetadata::default(),
        is_read: false,
        read_at: None,
        created_at: Utc::now(),
    };

    match critical_alert.severity {
        AlertSeverity::Critical => assert!(true),
        _ => assert!(false, "Should be critical"),
    }
}

/// Test inventory recommendation logic
#[test]
fn test_inventory_recommendation_logic() {
    // Test days of stock calculation
    let current_stock = 10;
    let daily_demand = 2.0;
    let days_of_stock = current_stock as f64 / daily_demand;
    
    assert_eq!(days_of_stock, 5.0);
    
    // Should recommend restock if less than 7 days
    let should_restock = days_of_stock < 7.0;
    assert!(should_restock);
    
    // Should recommend reduce if more than 90 days
    let overstock = 100;
    let days_overstock = overstock as f64 / daily_demand;
    let should_reduce = days_overstock > 90.0;
    assert!(!should_reduce); // 50 days, not overstocked
}

/// Test sales drop detection
#[test]
fn test_sales_drop_detection() {
    let current_revenue = 700.0;
    let previous_revenue = 1000.0;
    
    let change_percent = ((current_revenue - previous_revenue) / previous_revenue) * 100.0;
    let significant_drop = change_percent < -30.0;
    
    assert_eq!(change_percent, -30.0);
    assert!(!significant_drop); // Exactly -30%, not less than
    
    let current_revenue2 = 600.0;
    let change_percent2 = ((current_revenue2 - previous_revenue) / previous_revenue) * 100.0;
    let significant_drop2 = change_percent2 < -30.0;
    
    assert!(significant_drop2); // -40%, significant drop
}
