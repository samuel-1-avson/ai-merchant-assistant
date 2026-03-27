use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    // For now, pass through all requests
    // In production, validate JWT token here
    Ok(next.run(request).await)
}
