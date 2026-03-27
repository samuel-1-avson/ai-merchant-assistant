pub mod ai;
pub mod alerts;
pub mod analytics;
pub mod api;
pub mod auth;
pub mod config;
pub mod db;
pub mod external;
pub mod i18n;
pub mod models;
pub mod ocr;
pub mod realtime;
pub mod security;
pub mod services;
pub mod utils;

use std::sync::Arc;
use crate::ai::orchestrator::AIOrchestrator;
use crate::alerts::notifier::NotificationHub;
use crate::config::AppConfig;
use crate::db::Database;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    pub ai_orchestrator: Arc<AIOrchestrator>,
    pub notification_hub: Option<Arc<NotificationHub>>,
}
