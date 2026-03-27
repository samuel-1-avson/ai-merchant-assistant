//! Rate limiting for API protection

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use axum::{
    extract::ConnectInfo,
    http::StatusCode,
    response::IntoResponse,
};

pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
    max_requests: u32,
    window: Duration,
}

struct TokenBucket {
    tokens: u32,
    last_update: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            max_requests,
            window,
        }
    }

    pub async fn check_rate_limit(&self, key: &str) -> bool {
        let mut buckets = self.buckets.write().await;
        let now = Instant::now();

        let bucket = buckets.entry(key.to_string()).or_insert(TokenBucket {
            tokens: self.max_requests,
            last_update: now,
        });

        let time_passed = now.duration_since(bucket.last_update);
        let tokens_to_add = (time_passed.as_secs_f64() / self.window.as_secs_f64() * self.max_requests as f64) as u32;
        
        bucket.tokens = (bucket.tokens + tokens_to_add).min(self.max_requests);
        bucket.last_update = now;

        if bucket.tokens > 0 {
            bucket.tokens -= 1;
            true
        } else {
            false
        }
    }
}

pub struct RateLimitError;

impl IntoResponse for RateLimitError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::TOO_MANY_REQUESTS, "Rate limit exceeded").into_response()
    }
}
