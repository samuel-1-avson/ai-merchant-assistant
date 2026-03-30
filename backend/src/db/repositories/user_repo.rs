use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{User, CreateUserRequest};

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, full_name, business_name, password_hash, google_id, github_id, email_verified, created_at, updated_at
            FROM users 
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_email(&self, email: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, full_name, business_name, password_hash, google_id, github_id, email_verified, created_at, updated_at
            FROM users 
            WHERE email = $1
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_google_id(&self, google_id: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, full_name, business_name, password_hash, google_id, github_id, email_verified, created_at, updated_at
            FROM users 
            WHERE google_id = $1
            "#
        )
        .bind(google_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    pub async fn find_by_github_id(&self, github_id: &str) -> anyhow::Result<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, full_name, business_name, password_hash, google_id, github_id, email_verified, created_at, updated_at
            FROM users 
            WHERE github_id = $1
            "#
        )
        .bind(github_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    /// Find user with password hash for authentication
    pub async fn find_by_email_with_password(&self, email: &str) -> anyhow::Result<Option<UserWithPassword>> {
        let row = sqlx::query(
            r#"
            SELECT id, email, password_hash, full_name, business_name, created_at, updated_at
            FROM users 
            WHERE email = $1 AND password_hash IS NOT NULL
            "#
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                use sqlx::Row;
                Ok(Some(UserWithPassword {
                    id: row.try_get("id")?,
                    email: row.try_get("email")?,
                    password_hash: row.try_get("password_hash")?,
                    full_name: row.try_get("full_name")?,
                    business_name: row.try_get("business_name")?,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                }))
            }
            None => Ok(None),
        }
    }

    /// Create a new user with password
    pub async fn create_user(&self, request: &CreateUserRequest, password_hash: &str) -> anyhow::Result<User> {
        let id = Uuid::new_v4();
        
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, full_name, business_name, email_verified, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            RETURNING id, email, full_name, business_name, password_hash, google_id, github_id, email_verified, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&request.email)
        .bind(password_hash)
        .bind(&request.full_name)
        .bind(&request.business_name)
        .bind(request.email_verified)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Create OAuth user (Google/GitHub)
    pub async fn create_oauth_user(&self, request: &CreateUserRequest) -> anyhow::Result<User> {
        let id = Uuid::new_v4();
        
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (id, email, password_hash, full_name, business_name, google_id, github_id, email_verified, created_at, updated_at)
            VALUES ($1, $2, NULL, $3, $4, $5, $6, $7, NOW(), NOW())
            RETURNING id, email, full_name, business_name, password_hash, google_id, github_id, email_verified, created_at, updated_at
            "#
        )
        .bind(id)
        .bind(&request.email)
        .bind(&request.full_name)
        .bind(&request.business_name)
        .bind(&request.google_id)
        .bind(&request.github_id)
        .bind(request.email_verified)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    /// Link Google account to existing user
    pub async fn link_google_account(&self, user_id: Uuid, google_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE users 
            SET google_id = $1, updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(google_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Link GitHub account to existing user
    pub async fn link_github_account(&self, user_id: Uuid, github_id: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE users 
            SET github_id = $1, updated_at = NOW()
            WHERE id = $2
            "#
        )
        .bind(github_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Upsert a Supabase OAuth user using their exact Supabase UUID.
    ///
    /// Called on every authenticated request to ensure the user row exists
    /// before any FK-dependent insert (transactions, products, etc.).
    /// ON CONFLICT DO NOTHING makes this a cheap no-op after the first call.
    pub async fn upsert_supabase_user(&self, id: Uuid, email: &str) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, email, email_verified, created_at, updated_at)
            VALUES ($1, $2, true, NOW(), NOW())
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(id)
        .bind(email)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Mark email as verified
    pub async fn mark_email_verified(&self, user_id: Uuid) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            UPDATE users 
            SET email_verified = TRUE, updated_at = NOW()
            WHERE id = $1
            "#
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// User with password hash for authentication
pub struct UserWithPassword {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub full_name: Option<String>,
    pub business_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
