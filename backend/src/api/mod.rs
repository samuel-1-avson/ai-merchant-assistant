pub mod routes;
pub mod middleware;

use axum::{
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::models::ApiResponse;

pub fn success_response<T: serde::Serialize>(data: T) -> Response {
    Json(ApiResponse::success(data)).into_response()
}

pub fn error_response(message: &str) -> Response {
    Json(ApiResponse::<()>::error(message.to_string())).into_response()
}
