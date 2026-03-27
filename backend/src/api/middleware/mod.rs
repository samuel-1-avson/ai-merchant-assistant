pub mod auth;

pub use auth::{jwt_auth_middleware, optional_auth_middleware, AuthUser, get_auth_user};

use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

/// Legacy pass-through middleware (for backward compatibility)
pub async fn auth_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Pass through - actual auth is done via jwt_auth_middleware
    Ok(next.run(request).await)
}
