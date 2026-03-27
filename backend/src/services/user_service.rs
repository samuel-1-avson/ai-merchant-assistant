use std::sync::Arc;
use uuid::Uuid;
use argon2::{Argon2, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{SaltString, rand_core::OsRng};

use crate::db::repositories::user_repo::{UserRepository, UserWithPassword};
use crate::models::user::{User, CreateUserRequest, LoginRequest, AuthResponse};
use crate::auth::jwt::generate_token;

pub struct UserService {
    user_repo: Arc<UserRepository>,
}

impl UserService {
    pub fn new(user_repo: Arc<UserRepository>) -> Self {
        Self { user_repo }
    }

    pub async fn get_user(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        self.user_repo.find_by_id(id).await
    }

    pub async fn get_user_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        self.user_repo.find_by_email(email).await
    }

    pub async fn find_by_google_id(&self, google_id: &str) -> anyhow::Result<Option<User>> {
        self.user_repo.find_by_google_id(google_id).await
    }

    pub async fn find_by_github_id(&self, github_id: &str) -> anyhow::Result<Option<User>> {
        self.user_repo.find_by_github_id(github_id).await
    }

    pub async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        self.user_repo.find_by_email(email).await
    }

    /// Create a new user with password hashing
    pub async fn create_user(&self, request: &CreateUserRequest) -> anyhow::Result<User> {
        // Check if user already exists
        if let Some(_) = self.user_repo.find_by_email(&request.email).await? {
            anyhow::bail!("User with this email already exists");
        }

        // Get password or error
        let password = request.password.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Password is required for email registration"))?;

        // Hash password
        let argon2 = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
            .to_string();

        // Create user in database
        self.user_repo.create_user(request, &password_hash).await
    }

    /// Create OAuth user (Google/GitHub)
    pub async fn create_oauth_user(&self, request: &CreateUserRequest) -> anyhow::Result<User> {
        // Check if user already exists
        if let Some(_) = self.user_repo.find_by_email(&request.email).await? {
            anyhow::bail!("User with this email already exists");
        }

        // Create OAuth user in database
        self.user_repo.create_oauth_user(request).await
    }

    /// Link Google account to existing user
    pub async fn link_google_account(&self, user_id: Uuid, google_id: &str) -> anyhow::Result<()> {
        self.user_repo.link_google_account(user_id, google_id).await
    }

    /// Link GitHub account to existing user
    pub async fn link_github_account(&self, user_id: Uuid, github_id: &str) -> anyhow::Result<()> {
        self.user_repo.link_github_account(user_id, github_id).await
    }

    /// Authenticate user and generate JWT token
    pub async fn authenticate(&self, request: &LoginRequest) -> anyhow::Result<AuthResponse> {
        // Find user by email
        let user = self.user_repo
            .find_by_email_with_password(&request.email)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Invalid email or password"))?;

        // Verify password
        let argon2 = Argon2::default();
        let parsed_hash = argon2::PasswordHash::new(&user.password_hash)
            .map_err(|e| anyhow::anyhow!("Invalid password hash: {}", e))?;
        
        argon2
            .verify_password(request.password.as_bytes(), &parsed_hash)
            .map_err(|_| anyhow::anyhow!("Invalid email or password"))?;

        // Generate JWT token
        let token = generate_token(user.id, &user.email)?;

        // Return user without password hash
        let user_response = User {
            id: user.id,
            email: user.email,
            full_name: user.full_name,
            business_name: user.business_name,
            password_hash: None,
            google_id: None,
            github_id: None,
            email_verified: false,
            created_at: user.created_at,
            updated_at: user.updated_at,
        };

        Ok(AuthResponse {
            user: user_response,
            token,
        })
    }

    /// Mark email as verified
    pub async fn mark_email_verified(&self, user_id: Uuid) -> anyhow::Result<()> {
        self.user_repo.mark_email_verified(user_id).await
    }
}
