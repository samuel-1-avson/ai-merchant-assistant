use sqlx::{PgPool, Row};
use uuid::Uuid;
use rust_decimal::Decimal;
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};

use crate::models::product::{Product, CreateProductRequest};

pub struct ProductRepository {
    pool: PgPool,
}

impl ProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        user_id: Uuid,
        request: CreateProductRequest,
    ) -> anyhow::Result<Product> {
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO products (id, user_id, name, description, sku, default_price, cost_price, unit, stock_quantity, low_stock_threshold, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())
            RETURNING id, user_id, name, description, sku, default_price, cost_price, unit, stock_quantity, low_stock_threshold, is_active, image_url, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(user_id)
        .bind(request.name)
        .bind(request.description)
        .bind(request.sku)
        .bind(request.default_price.map(|d| d.to_f64().unwrap_or(0.0)))
        .bind(request.cost_price.map(|d| d.to_f64().unwrap_or(0.0)))
        .bind(request.unit.unwrap_or_else(|| "piece".to_string()))
        .bind(request.stock_quantity.unwrap_or(0))
        .bind(request.low_stock_threshold.unwrap_or(10))
        .fetch_one(&self.pool)
        .await?;

        self.row_to_product(&row)
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<Product>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, name, description, sku, default_price, cost_price, unit, 
                   stock_quantity, low_stock_threshold, is_active, image_url, created_at, updated_at
            FROM products 
            WHERE user_id = $1 AND is_active = true
            ORDER BY name ASC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| self.row_to_product(&r)).collect()
    }

    pub async fn find_by_name(
        &self,
        user_id: Uuid,
        name: &str,
    ) -> anyhow::Result<Option<Product>> {
        let row = sqlx::query(
            r#"
            SELECT id, user_id, name, description, sku, default_price, cost_price, unit, 
                   stock_quantity, low_stock_threshold, is_active, image_url, created_at, updated_at
            FROM products 
            WHERE user_id = $1 
            AND LOWER(name) = LOWER($2)
            AND is_active = true
            LIMIT 1
            "#
        )
        .bind(user_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(r) => Ok(Some(self.row_to_product(&r)?)),
            None => Ok(None),
        }
    }

    pub async fn search_by_name(
        &self,
        user_id: Uuid,
        query: &str,
    ) -> anyhow::Result<Vec<Product>> {
        let rows = sqlx::query(
            r#"
            SELECT id, user_id, name, description, sku, default_price, cost_price, unit, 
                   stock_quantity, low_stock_threshold, is_active, image_url, created_at, updated_at
            FROM products 
            WHERE user_id = $1 
            AND LOWER(name) LIKE LOWER($2)
            AND is_active = true
            ORDER BY name ASC
            LIMIT 10
            "#
        )
        .bind(user_id)
        .bind(format!("%{}%", query))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter().map(|r| self.row_to_product(&r)).collect()
    }

    /// Helper to convert row to Product
    fn row_to_product(&self, row: &sqlx::postgres::PgRow) -> anyhow::Result<Product> {
        use sqlx::Row;
        
        let default_price: Option<f64> = row.try_get("default_price")?;
        let cost_price: Option<f64> = row.try_get("cost_price")?;
        
        Ok(Product {
            id: row.try_get("id")?,
            user_id: row.try_get("user_id")?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            sku: row.try_get("sku")?,
            default_price: default_price.and_then(Decimal::from_f64),
            cost_price: cost_price.and_then(Decimal::from_f64),
            unit: row.try_get("unit")?,
            stock_quantity: row.try_get("stock_quantity")?,
            low_stock_threshold: row.try_get("low_stock_threshold")?,
            is_active: row.try_get("is_active")?,
            image_url: row.try_get("image_url")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}
