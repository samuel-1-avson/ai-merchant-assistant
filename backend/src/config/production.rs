//! Production configuration

use super::AppConfig;
use std::env;

/// Production-specific configuration
#[derive(Clone, Debug)]
pub struct ProductionConfig {
    pub app: AppConfig,
    pub database_pool_size: u32,
    pub redis_url: String,
    pub sentry_dsn: Option<String>,
    pub log_level: String,
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub health_check_interval: u64,
}

impl ProductionConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        let app_config = AppConfig::from_env()?;
        
        Ok(Self {
            app: app_config,
            database_pool_size: env::var("DATABASE_POOL_SIZE")
                .unwrap_or_else(|_| "20".to_string())
                .parse()?,
            redis_url: env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            sentry_dsn: env::var("SENTRY_DSN").ok(),
            log_level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            enable_metrics: env::var("ENABLE_METRICS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()?,
            metrics_port: env::var("METRICS_PORT")
                .unwrap_or_else(|_| "9090".to_string())
                .parse()?,
            health_check_interval: env::var("HEALTH_CHECK_INTERVAL")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
        })
    }
}

/// Health check configuration
#[derive(Clone, Debug)]
pub struct HealthConfig {
    pub database_timeout: u64,
    pub ai_service_timeout: u64,
    pub storage_timeout: u64,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            database_timeout: 5,
            ai_service_timeout: 10,
            storage_timeout: 5,
        }
    }
}
