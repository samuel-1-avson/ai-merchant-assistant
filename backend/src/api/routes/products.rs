use axum::{
    extract::{State, Query},
    Json,
    http::StatusCode,
};
use std::sync::Arc;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::models::product::CreateProductRequest;
use crate::api::state::AppState;
use crate::utils::errors::ApiError;

#[derive(Debug, serde::Deserialize)]
pub struct ListProductsQuery {
    pub search: Option<String>,
}

/// List all products for the authenticated user
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token after auth is implemented
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    let products = if let Some(search_term) = query.search {
        // Search products by name
        state
            .product_service
            .search_by_name(user_id, &search_term)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?
    } else {
        // List all products
        state
            .product_service
            .list_products(user_id)
            .await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?
    };

    Ok(Json(json!({
        "success": true,
        "data": products,
        "meta": {
            "count": products.len()
        }
    })))
}

/// Create a new product
pub async fn create(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    // Validate request
    if request.name.trim().is_empty() {
        return Err(ApiError::ValidationError("Product name is required".to_string()));
    }

    // Check for duplicate product name
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

/// Search products by name (alias for list with search parameter)
pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListProductsQuery>,
) -> Result<Json<Value>, ApiError> {
    // Reuse the list function logic
    list(State(state), Query(query)).await
}

/// Get product suggestions for voice input (fuzzy matching)
pub async fn suggestions(
    State(state): State<Arc<AppState>>,
    Query(query): Query<super::voice::ProductSuggestionQuery>,
) -> Result<Json<Value>, ApiError> {
    // TODO: Get actual user_id from JWT token
    let user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|_| ApiError::Unauthorized("Invalid user".to_string()))?;

    let suggestions = state
        .product_service
        .search_by_name(user_id, &query.q)
        .await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

    Ok(Json(json!({
        "success": true,
        "data": suggestions
    })))
}
