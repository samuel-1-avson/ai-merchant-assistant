use axum::{
    extract::{State, Query, Extension},
    Json,
};
use std::sync::Arc;
use serde_json::{json, Value};

use crate::models::product::CreateProductRequest;
use crate::api::state::AppState;
use crate::api::middleware::AuthUser;
use crate::utils::errors::ApiError;

#[derive(Debug, serde::Deserialize)]
pub struct ListProductsQuery {
    pub search: Option<String>,
}

/// List all products for the authenticated user
pub async fn list(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    let products = if let Some(search_term) = query.search {
        state
            .product_service
            .search_by_name(user_id, &search_term)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?
    } else {
        state
            .product_service
            .list_products(user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "products": products,
            "meta": {
                "count": products.len()
            }
        }
    })))
}

/// Create a new product
pub async fn create(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    if request.name.trim().is_empty() {
        return Err(ApiError::ValidationError("Product name is required".to_string()));
    }

    if let Some(existing) = state
        .product_service
        .find_by_name(user_id, &request.name)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?
    {
        return Err(ApiError::ValidationError(
            format!("Product '{}' already exists", existing.name)
        ));
    }

    let product = state
        .product_service
        .create_product(user_id, request)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": product,
        "message": "Product created successfully"
    })))
}

/// Search products by name
pub async fn search(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<Value>, ApiError> {
    list(State(state), Extension(auth_user), Query(query)).await
}
