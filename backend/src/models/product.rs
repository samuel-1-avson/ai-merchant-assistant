use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Product {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub default_price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    pub unit: String,
    pub stock_quantity: i32,
    pub low_stock_threshold: i32,
    pub is_active: bool,
    pub image_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub sku: Option<String>,
    pub default_price: Option<Decimal>,
    pub cost_price: Option<Decimal>,
    pub unit: Option<String>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub default_price: Option<Decimal>,
    pub stock_quantity: Option<i32>,
}
