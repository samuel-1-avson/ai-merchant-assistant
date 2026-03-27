use sqlx::PgPool;

pub mod repositories;

#[derive(Clone)]
pub struct Database {
    pub pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        // Note: Migrations are disabled. Schema is managed via Supabase Dashboard.
        // Run supabase_schema.sql in the Supabase SQL Editor to set up the schema.
        
        Ok(Self { pool })
    }
}
