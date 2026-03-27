use std::sync::Arc;
use uuid::Uuid;

use crate::db::repositories::product_repo::ProductRepository;
use crate::models::product::{Product, CreateProductRequest};

pub struct ProductService {
    repo: Arc<ProductRepository>,
}

impl ProductService {
    pub fn new(repo: Arc<ProductRepository>) -> Self {
        Self { repo }
    }

    pub async fn create_product(
        &self,
        user_id: Uuid,
        request: CreateProductRequest,
    ) -> anyhow::Result<Product> {
        self.repo.create(user_id, request).await
    }

    pub async fn list_products(&self, user_id: Uuid) -> anyhow::Result<Vec<Product>> {
        self.repo.list_by_user(user_id).await
    }

    pub async fn find_by_name(&self, user_id: Uuid, name: &str) -> anyhow::Result<Option<Product>> {
        self.repo.find_by_name(user_id, name).await
    }

    pub async fn search_by_name(&self, user_id: Uuid, query: &str) -> anyhow::Result<Vec<Product>> {
        self.repo.search_by_name(user_id, query).await
    }
}
