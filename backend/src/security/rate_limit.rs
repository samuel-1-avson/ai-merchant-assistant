//! Rate limiting for API protection
//!
//! This module provides configurable rate limiting with:
//! - Per-endpoint rate limits
//! - Per-user rate limits
//! - IP-based rate limiting
//! - Configurable windows and burst sizes

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    body::Body,
};
use tracing::{info, warn, debug};

/// Rate limit configuration for different endpoint types
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Default limit for general endpoints (requests per minute)
    pub default_limit: u32,
    /// Strict limit for expensive operations (AI, OCR)
    pub ai_limit: u32,
    /// Auth endpoints limit (login, register)
    pub auth_limit: u32,
    /// Window duration
    pub window: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            default_limit: 100,      // 100 requests per minute
            ai_limit: 20,            // 20 AI requests per minute
            auth_limit: 10,          // 10 auth attempts per minute
            window: Duration::from_secs(60),
        }
    }
}

/// Token bucket for rate limiting
struct TokenBucket {
    tokens: f64,
    last_update: Instant,
}

/// Rate limiter with per-key tracking
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    config: RateLimitConfig,
}

impl RateLimiter {
    /// Create new rate limiter with config
    pub fn new(config: RateLimitConfig) -> Self {
        // Start cleanup task
        let buckets = Arc::new(RwLock::new(HashMap::new()));
        Self::start_cleanup_task(buckets.clone(), config.window);

        Self { buckets, config }
    }

    /// Check if request is allowed for given key
    pub async fn is_allowed(&self, key: &str, limit: u32) -> (bool, u32) {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();

        let bucket = buckets.entry(key.to_string()).or_insert(TokenBucket {
            tokens: limit as f64,
            last_update: now,
        });

        // Calculate tokens to add based on time passed
        let time_passed = now.duration_since(bucket.last_update);
        let tokens_to_add = time_passed.as_secs_f64() / self.config.window.as_secs_f64() * limit as f64;
        
        bucket.tokens = (bucket.tokens + tokens_to_add).min(limit as f64);
        bucket.last_update = now;

        let remaining = bucket.tokens.floor() as u32;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            debug!("Rate limit allowed for key: {}, remaining: {}", key, remaining.saturating_sub(1));
            (true, remaining.saturating_sub(1))
        } else {
            warn!("Rate limit exceeded for key: {}", key);
            (false, 0)
        }
    }

    /// Check with default limit
    pub async fn check(&self, key: &str) -> (bool, u32) {
        self.is_allowed(key, self.config.default_limit).await
    }

    /// Check with AI limit
    pub async fn check_ai(&self, key: &str) -> (bool, u32) {
        self.is_allowed(key, self.config.ai_limit).await
    }

    /// Check with auth limit
    pub async fn check_auth(&self, key: &str) -> (bool, u32) {
        self.is_allowed(key, self.config.auth_limit).await
    }

    /// Start background cleanup task
    fn start_cleanup_task(buckets: Arc<RwLock<HashMap<String, TokenBucket>>>, window: Duration) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(window * 2).await;
                
                let mut buckets = buckets.write().await;
                let before_count = buckets.len();
                
                // Remove buckets that haven't been used in 2 windows
                let now = Instant::now();
                buckets.retain(|_, bucket| {
                    now.duration_since(bucket.last_update) < window * 2
                });
                
                let after_count = buckets.len();
                if before_count != after_count {
                    debug!("Rate limiter cleanup: removed {} stale buckets", before_count - after_count);
                }
            }
        });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(RateLimitConfig::default())
    }
}

/// Extract client IP for rate limiting
fn extract_client_ip(req: &Request) -> String {
    // Try X-Forwarded-For header first (for proxied requests)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(ip) = forwarded.to_str() {
            return ip.split(',').next().unwrap_or(ip).trim().to_string();
        }
    }
    
    // Try X-Real-IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip) = real_ip.to_str() {
            return ip.to_string();
        }
    }
    
    // Fall back to socket address
    if let Some(addr) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        return addr.ip().to_string();
    }
    
    "unknown".to_string()
}

/// Get rate limit key based on endpoint and user
fn get_rate_limit_key(req: &Request, user_id: Option<&str>) -> String {
    let path = req.uri().path();
    let client_ip = extract_client_ip(req);
    
    // Use user ID if available, otherwise IP
    let identifier = user_id.unwrap_or(&client_ip);
    
    // Different keys for different endpoint types
    if path.contains("/voice") || path.contains("/ai") || path.contains("/ocr") {
        format!("ai:{}", identifier)
    } else if path.contains("/auth") || path.contains("/login") || path.contains("/register") {
        format!("auth:{}", identifier)
    } else {
        format!("api:{}", identifier)
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    rate_limiter: Arc<RateLimiter>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let path = req.uri().path().to_string();
    let key = get_rate_limit_key(&req, None); // TODO: Extract from JWT
    
    // Determine which limit to use
    let (allowed, remaining) = if path.contains("/voice") || path.contains("/ocr") {
        rate_limiter.check_ai(&key).await
    } else if path.contains("/auth") {
        rate_limiter.check_auth(&key).await
    } else {
        rate_limiter.check(&key).await
    };

    if allowed {
        let response = next.run(req).await;
        
        // Add rate limit headers
        let (parts, body) = response.into_parts();
        let mut response = axum::response::Response::from_parts(parts, body);
        
        response.headers_mut().insert(
            "X-RateLimit-Limit",
            if path.contains("/voice") || path.contains("/ocr") {
                rate_limiter.config.ai_limit.to_string().parse().unwrap()
            } else if path.contains("/auth") {
                rate_limiter.config.auth_limit.to_string().parse().unwrap()
            } else {
                rate_limiter.config.default_limit.to_string().parse().unwrap()
            }
        );
        response.headers_mut().insert(
            "X-RateLimit-Remaining",
            remaining.to_string().parse().unwrap()
        );
        
        response
    } else {
        RateLimitError.into_response()
    }
}

/// Rate limit error response
pub struct RateLimitError;

impl IntoResponse for RateLimitError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({
            "success": false,
            "error": "Rate limit exceeded",
            "message": "Too many requests. Please try again later.",
            "retry_after": 60
        });
        
        (
            StatusCode::TOO_MANY_REQUESTS,
            [("Retry-After", "60")],
            axum::Json(body)
        ).into_response()
    }
}

/// Rate limiting middleware factory
/// Usage: `.layer(rate_limit_layer(rate_limiter))`
pub fn rate_limit_layer(
    rate_limiter: Arc<RateLimiter>,
) -> tower::layer::util::Identity {
    // For now, return identity layer - rate limiting can be applied per-handler
    // Full middleware implementation would require proper tower::Layer implementation
    tower::layer::util::Identity::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let config = RateLimitConfig {
            default_limit: 5,
            window: Duration::from_secs(60),
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // Should allow 5 requests
        for i in 0..5 {
            let (allowed, remaining) = limiter.check("test_key").await;
            assert!(allowed, "Request {} should be allowed", i);
            assert_eq!(remaining, 4 - i);
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let config = RateLimitConfig {
            default_limit: 2,
            window: Duration::from_secs(60),
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        // Use up all tokens
        limiter.check("test_key").await;
        limiter.check("test_key").await;

        // Next request should be blocked
        let (allowed, _) = limiter.check("test_key").await;
        assert!(!allowed);
    }
}
