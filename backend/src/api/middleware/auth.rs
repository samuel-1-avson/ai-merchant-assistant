use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::auth::JwtValidator;

/// Extension to store authenticated user info in request
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub email: String,
}

/// Extract JWT token from Authorization header
fn extract_token_from_header<B>(req: &Request<B>) -> Option<String> {
    req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            if value.starts_with("Bearer ") {
                Some(value[7..].to_string())
            } else {
                None
            }
        })
}

/// JWT authentication middleware
pub async fn jwt_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from header
    let token = match extract_token_from_header(&request) {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate JWT
    let validator = JwtValidator::new(&state.config.jwt_secret);
    let claims = match validator.validate(&token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Parse user ID
    let user_id = match Uuid::parse_str(&claims.sub) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Add user info to request extensions
    let auth_user = AuthUser {
        user_id,
        email: claims.email,
    };
    request.extensions_mut().insert(auth_user);

    Ok(next.run(request).await)
}

/// Extract auth user from request extensions
pub fn get_auth_user<B>(req: &Request<B>) -> Option<AuthUser> {
    req.extensions().get::<AuthUser>().cloned()
}

/// Optional auth middleware - doesn't fail if no token provided
pub async fn optional_auth_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if let Some(token) = extract_token_from_header(&request) {
        let validator = JwtValidator::new(&state.config.jwt_secret);
        if let Ok(claims) = validator.validate(&token) {
            if let Ok(user_id) = Uuid::parse_str(&claims.sub) {
                let auth_user = AuthUser {
                    user_id,
                    email: claims.email,
                };
                request.extensions_mut().insert(auth_user);
            }
        }
    }

    next.run(request).await
}
