//! CORS configuration for production

use tower_http::cors::CorsLayer;
use axum::http::HeaderValue;

pub fn production_cors() -> CorsLayer {
    let allowed_origins: Vec<HeaderValue> = vec![
        "https://aimerchant.app",
        "https://www.aimerchant.app",
    ].into_iter()
    .filter_map(|o| HeaderValue::from_str(o).ok())
    .collect();

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(true)
        .max_age(std::time::Duration::from_secs(86400))
}

pub fn development_cors() -> CorsLayer {
    CorsLayer::permissive()
}
