use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tracing::info;

pub mod repositories;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        // Configure pool for direct Supabase PostgreSQL connection
        // Using direct connection (port 5432) instead of connection pooler (port 6543)
        // to avoid prepared statement issues with PgBouncer
        
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .min_connections(2)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .test_before_acquire(true)
            .connect(database_url)
            .await?;

        // Note: Migrations are disabled. Schema is managed via Supabase Dashboard.
        // Run supabase_schema.sql in the Supabase SQL Editor to set up the schema.
        
        // Test the connection with a simple query
        let row: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await?;
        
        info!("✅ Database connection pool established successfully (test query: {})", row.0);
        
        Ok(Self { pool })
    }
}
