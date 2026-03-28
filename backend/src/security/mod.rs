//! Security module for production hardening

pub mod rate_limit;
pub mod cors;

pub use rate_limit::{RateLimiter, RateLimitConfig, rate_limit_layer};

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Duration;

/// Security configuration for production
#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub rate_limit_requests: u32,
    pub rate_limit_window: Duration,
    pub max_body_size: usize,
    pub allowed_origins: Vec<String>,
    pub enable_hsts: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            rate_limit_requests: 100,
            rate_limit_window: Duration::from_secs(60),
            max_body_size: 10 * 1024 * 1024,
            allowed_origins: vec!["https://aimerchant.app".to_string()],
            enable_hsts: true,
        }
    }
}

/// Production security middleware
pub async fn security_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    
    headers.insert(
        "strict-transport-security",
        "max-age=31536000; includeSubDomains; preload".parse().unwrap(),
    );
    
    headers.insert(
        "content-security-policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:;".parse().unwrap(),
    );
    
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("x-frame-options", "DENY".parse().unwrap());
    headers.insert("x-xss-protection", "1; mode=block".parse().unwrap());
    headers.insert("referrer-policy", "strict-origin-when-cross-origin".parse().unwrap());
    
    response
}
