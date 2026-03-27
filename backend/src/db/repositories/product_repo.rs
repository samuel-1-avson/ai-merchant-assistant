use sqlx::PgPool;
use uuid::Uuid;

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
        let product = sqlx::query_as::<_, Product>(
            r#"
            INSERT INTO products (id, user_id, name, description, sku, default_price, cost_price, unit, stock_quantity, low_stock_threshold, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, true, NOW(), NOW())
            RETURNING *
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(request.name)
        .bind(request.description)
        .bind(request.sku)
        .bind(request.default_price)
        .bind(request.cost_price)
        .bind(request.unit.unwrap_or_else(|| "piece".to_string()))
        .bind(request.stock_quantity.unwrap_or(0))
        .bind(request.low_stock_threshold.unwrap_or(10))
        .fetch_one(&self.pool)
        .await?;

        Ok(product)
    }

    pub async fn list_by_user(&self, user_id: Uuid) -> anyhow::Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            r#"
            SELECT * FROM products 
            WHERE user_id = $1 AND is_active = true
            ORDER BY name ASC
            "#
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(products)
    }

    pub async fn find_by_name(
        &self,
        user_id: Uuid,
        name: &str,
    ) -> anyhow::Result<Option<Product>> {
        let product = sqlx::query_as::<_, Product>(
            r#"
            SELECT * FROM products 
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

        Ok(product)
    }

    pub async fn search_by_name(
        &self,
        user_id: Uuid,
        query: &str,
    ) -> anyhow::Result<Vec<Product>> {
        let products = sqlx::query_as::<_, Product>(
            r#"
            SELECT * FROM products 
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

        Ok(products)
    }
}
