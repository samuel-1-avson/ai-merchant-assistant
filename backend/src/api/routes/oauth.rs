//! OAuth Authentication Routes
//! 
//! Handles Google and GitHub OAuth authentication

use axum::{
    extract::State,
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tracing::{info, error};

use crate::api::state::AppState;
use crate::utils::errors::ApiError;
use crate::models::user::CreateUserRequest;

/// Google OAuth request
#[derive(Debug, Deserialize)]
pub struct GoogleOAuthRequest {
    pub token: String,
}

/// OAuth user info response from Google
#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    sub: String,  // Google's unique user ID
    email: String,
    name: Option<String>,
    picture: Option<String>,
    email_verified: Option<bool>,
}

/// OAuth response
#[derive(Debug, Serialize)]
pub struct OAuthResponse {
    pub user: Value,
    pub token: String,
}

/// Handle Google OAuth authentication
/// 
/// Flow:
/// 1. Frontend sends Google OAuth token
/// 2. Backend validates token with Google
/// 3. If valid, create or get user
/// 4. Generate JWT token
/// 5. Return user + token
pub async fn google_auth(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GoogleOAuthRequest>,
) -> Result<Json<Value>, ApiError> {
    info!("Processing Google OAuth request");
    
    // Validate the Google token and get user info
    let google_user = validate_google_token(&request.token).await
        .map_err(|e| ApiError::AuthenticationError(format!("Invalid Google token: {}", e)))?;
    
    info!("Google user authenticated: {}", google_user.email);
    
    // Check if user exists by Google ID
    let existing_user = state.user_service.find_by_google_id(&google_user.sub).await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let user = if let Some(user) = existing_user {
        info!("Existing user found for Google account: {}", user.id);
        user
    } else {
        // Check if user exists with same email
        let email_user = state.user_service.find_by_email(&google_user.email).await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        if let Some(mut user) = email_user {
            // Link Google account to existing user
            info!("Linking Google account to existing user: {}", user.id);
            state.user_service.link_google_account(user.id, &google_user.sub).await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
            user.google_id = Some(google_user.sub);
            user
        } else {
            // Create new user
            info!("Creating new user from Google OAuth");
            let create_request = CreateUserRequest {
                email: google_user.email,
                password: None, // OAuth users don't have passwords
                full_name: google_user.name,
                business_name: None,
                google_id: Some(google_user.sub),
                github_id: None,
                email_verified: google_user.email_verified.unwrap_or(false),
            };
            
            state.user_service.create_oauth_user(&create_request).await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        }
    };
    
    // Generate JWT token
    let token = crate::auth::jwt::generate_token(user.id, &user.email)
        .map_err(|e| ApiError::AuthenticationError(e.to_string()))?;
    
    Ok(Json(json!({
        "success": true,
        "data": {
            "user": {
                "id": user.id,
                "email": user.email,
                "full_name": user.full_name,
                "business_name": user.business_name,
                "email_verified": user.email_verified,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
            },
            "token": token,
        },
        "message": "Google authentication successful"
    })))
}

/// Validate Google OAuth token with Google's API
async fn validate_google_token(token: &str) -> anyhow::Result<GoogleUserInfo> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://www.googleapis.com/oauth2/v3/userinfo")
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await?;
        error!("Google token validation failed: {}", error_text);
        return Err(anyhow::anyhow!("Token validation failed: {}", error_text));
    }
    
    let user_info: GoogleUserInfo = response.json().await?;
    Ok(user_info)
}

/// GitHub OAuth request
#[derive(Debug, Deserialize)]
pub struct GitHubOAuthRequest {
    pub code: String,
}

/// GitHub access token response
#[derive(Debug, Deserialize)]
struct GitHubAccessToken {
    access_token: String,
}

/// GitHub user info
#[derive(Debug, Deserialize)]
struct GitHubUserInfo {
    id: i64,
    login: String,
    email: Option<String>,
    name: Option<String>,
    avatar_url: Option<String>,
}

/// Handle GitHub OAuth authentication
pub async fn github_auth(
    State(state): State<Arc<AppState>>,
    Json(request): Json<GitHubOAuthRequest>,
) -> Result<Json<Value>, ApiError> {
    info!("Processing GitHub OAuth request");
    
    // Exchange code for access token
    let access_token = exchange_github_code(&request.code).await
        .map_err(|e| ApiError::AuthenticationError(format!("GitHub code exchange failed: {}", e)))?;
    
    // Get user info from GitHub
    let github_user = get_github_user(&access_token).await
        .map_err(|e| ApiError::AuthenticationError(format!("Failed to get GitHub user: {}", e)))?;
    
    info!("GitHub user authenticated: {}", github_user.login);
    
    // If email is not public, fetch it separately
    let email = if let Some(email) = github_user.email {
        email
    } else {
        get_github_email(&access_token).await
            .map_err(|e| ApiError::AuthenticationError(format!("Failed to get GitHub email: {}", e)))?
    };
    
    // Check if user exists
    let github_id = github_user.id.to_string();
    let existing_user = state.user_service.find_by_github_id(&github_id).await
        .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
    
    let user = if let Some(user) = existing_user {
        user
    } else {
        // Check if user exists with same email
        let email_user = state.user_service.find_by_email(&email).await
            .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
        
        if let Some(mut user) = email_user {
            // Link GitHub account
            state.user_service.link_github_account(user.id, &github_id).await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?;
            user.github_id = Some(github_id);
            user
        } else {
            // Create new user
            let create_request = CreateUserRequest {
                email,
                password: None,
                full_name: github_user.name,
                business_name: None,
                google_id: None,
                github_id: Some(github_id),
                email_verified: true, // GitHub verifies emails
            };
            
            state.user_service.create_oauth_user(&create_request).await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?
        }
    };
    
    // Generate JWT token
    let token = crate::auth::jwt::generate_token(user.id, &user.email)
        .map_err(|e| ApiError::AuthenticationError(e.to_string()))?;
    
    Ok(Json(json!({
        "success": true,
        "data": {
            "user": {
                "id": user.id,
                "email": user.email,
                "full_name": user.full_name,
                "business_name": user.business_name,
                "email_verified": user.email_verified,
                "created_at": user.created_at,
                "updated_at": user.updated_at,
            },
            "token": token,
        },
        "message": "GitHub authentication successful"
    })))
}

/// Exchange GitHub code for access token
async fn exchange_github_code(code: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    
    let client_id = std::env::var("GITHUB_CLIENT_ID")
        .map_err(|_| anyhow::anyhow!("GITHUB_CLIENT_ID not set"))?;
    let client_secret = std::env::var("GITHUB_CLIENT_SECRET")
        .map_err(|_| anyhow::anyhow!("GITHUB_CLIENT_SECRET not set"))?;
    
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .json(&serde_json::json!({
            "client_id": client_id,
            "client_secret": client_secret,
            "code": code,
        }))
        .send()
        .await?;
    
    let token_response: GitHubAccessToken = response.json().await?;
    Ok(token_response.access_token)
}

/// Get GitHub user info
async fn get_github_user(access_token: &str) -> anyhow::Result<GitHubUserInfo> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "AI-Merchant-Assistant")
        .send()
        .await?;
    
    let user_info: GitHubUserInfo = response.json().await?;
    Ok(user_info)
}

/// Get GitHub user email
async fn get_github_email(access_token: &str) -> anyhow::Result<String> {
    let client = reqwest::Client::new();
    
    #[derive(Debug, Deserialize)]
    struct GitHubEmail {
        email: String,
        primary: bool,
        verified: bool,
    }
    
    let response = client
        .get("https://api.github.com/user/emails")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("User-Agent", "AI-Merchant-Assistant")
        .send()
        .await?;
    
    let emails: Vec<GitHubEmail> = response.json().await?;
    
    // Find primary verified email
    emails.into_iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email)
        .ok_or_else(|| anyhow::anyhow!("No verified primary email found"))
}
