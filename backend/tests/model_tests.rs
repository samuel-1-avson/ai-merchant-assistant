//! Tests for Data Models

use ai_merchant_backend::models::{
    user::{User, CreateUserRequest, LoginRequest},
    transaction::{Transaction, CreateTransactionRequest, ExtractedEntities},
    product::{Product, CreateProductRequest},
    analytics::{AnalyticsSummary, TopProduct, DailySale, SalesTrend},
};
use uuid::Uuid;
use rust_decimal::Decimal;
use chrono::Utc;

/// Test user model
#[test]
fn test_user_model() {
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        full_name: Some("Test User".to_string()),
        business_name: Some("Test Business".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.full_name, Some("Test User".to_string()));
}

/// Test create user request
#[test]
fn test_create_user_request() {
    let request = CreateUserRequest {
        email: "new@example.com".to_string(),
        password: "password123".to_string(),
        full_name: Some("New User".to_string()),
        business_name: Some("New Business".to_string()),
    };

    assert_eq!(request.email, "new@example.com");
    assert_eq!(request.password, "password123");
}

/// Test login request
#[test]
fn test_login_request() {
    let request = LoginRequest {
        email: "user@example.com".to_string(),
        password: "secret".to_string(),
    };

    assert_eq!(request.email, "user@example.com");
    assert_eq!(request.password, "secret");
}

/// Test transaction model
#[test]
fn test_transaction_model() {
    let transaction = Transaction {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        product_id: Some(Uuid::new_v4()),
        quantity: Decimal::from(5),
        unit: "piece".to_string(),
        price: Decimal::from(10),
        total: Decimal::from(50),
        notes: Some("Test transaction".to_string()),
        voice_recording_url: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(transaction.quantity, Decimal::from(5));
    assert_eq!(transaction.total, Decimal::from(50));
}

/// Test create transaction request
#[test]
fn test_create_transaction_request() {
    let request = CreateTransactionRequest {
        product_id: Some(Uuid::new_v4()),
        quantity: Decimal::from(3),
        unit: Some("crate".to_string()),
        price: Decimal::from(15),
        notes: Some("Test".to_string()),
    };

    assert_eq!(request.quantity, Decimal::from(3));
    assert_eq!(request.price, Decimal::from(15));
}

/// Test product model
#[test]
fn test_product_model() {
    let product = Product {
        id: Uuid::new_v4(),
        user_id: Uuid::new_v4(),
        name: "Test Product".to_string(),
        description: Some("Description".to_string()),
        sku: Some("SKU123".to_string()),
        default_price: Some(Decimal::from(20)),
        cost_price: Some(Decimal::from(10)),
        unit: "piece".to_string(),
        stock_quantity: 100,
        low_stock_threshold: 10,
        is_active: true,
        image_url: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    assert_eq!(product.name, "Test Product");
    assert_eq!(product.stock_quantity, 100);
    assert!(product.is_active);
}

/// Test create product request
#[test]
fn test_create_product_request() {
    let request = CreateProductRequest {
        name: "New Product".to_string(),
        description: Some("New Description".to_string()),
        sku: Some("NEW123".to_string()),
        default_price: Some(Decimal::from(25)),
        cost_price: Some(Decimal::from(15)),
        unit: Some("box".to_string()),
        stock_quantity: Some(50),
        low_stock_threshold: Some(5),
    };

    assert_eq!(request.name, "New Product");
    assert_eq!(request.stock_quantity, Some(50));
}

/// Test analytics summary
#[test]
fn test_analytics_summary() {
    let summary = AnalyticsSummary {
        total_revenue: Decimal::from(1000),
        total_transactions: 50,
        total_items_sold: Decimal::from(200),
        average_transaction_value: Decimal::from(20),
        top_products: vec![],
        daily_sales: vec![],
    };

    assert_eq!(summary.total_revenue, Decimal::from(1000));
    assert_eq!(summary.total_transactions, 50);
}

/// Test top product
#[test]
fn test_top_product() {
    let product = TopProduct {
        product_id: Uuid::new_v4(),
        product_name: "Best Seller".to_string(),
        total_quantity: Decimal::from(100),
        total_revenue: Decimal::from(500),
        times_sold: 50,
    };

    assert_eq!(product.product_name, "Best Seller");
    assert_eq!(product.times_sold, 50);
}

/// Test sales trend
#[test]
fn test_sales_trend() {
    let trend = SalesTrend {
        period: "week".to_string(),
        current_value: Decimal::from(1000),
        previous_value: Decimal::from(800),
        change_percent: 25.0,
    };

    assert_eq!(trend.change_percent, 25.0);
    assert_eq!(trend.period, "week");
}

/// Test extracted entities
#[test]
fn test_extracted_entities() {
    let entities = ExtractedEntities {
        product: Some("Eggs".to_string()),
        quantity: Some(2.0),
        unit: Some("crate".to_string()),
        price: Some(10.0),
        currency: Some("USD".to_string()),
    };

    assert_eq!(entities.product, Some("Eggs".to_string()));
    assert_eq!(entities.quantity, Some(2.0));
    assert_eq!(entities.unit, Some("crate".to_string()));
    assert_eq!(entities.price, Some(10.0));
}

/// Test entity extraction with null values
#[test]
fn test_extracted_entities_nulls() {
    let entities = ExtractedEntities {
        product: None,
        quantity: None,
        unit: None,
        price: None,
        currency: None,
    };

    assert!(entities.product.is_none());
    assert!(entities.quantity.is_none());
}

/// Test JSON serialization roundtrip for models
#[test]
fn test_json_roundtrip() {
    let original = ExtractedEntities {
        product: Some("Test".to_string()),
        quantity: Some(5.0),
        unit: Some("kg".to_string()),
        price: Some(25.0),
        currency: Some("USD".to_string()),
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: ExtractedEntities = serde_json::from_str(&json).unwrap();

    assert_eq!(original.product, deserialized.product);
    assert_eq!(original.quantity, deserialized.quantity);
    assert_eq!(original.unit, deserialized.unit);
}
