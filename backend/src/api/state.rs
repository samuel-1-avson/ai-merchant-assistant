use std::sync::Arc;
use crate::ai::orchestrator::AIOrchestrator;
use crate::alerts::{AlertEngine, notifier::NotificationHub};
use crate::auth::JwtValidator;
use crate::config::AppConfig;
use crate::db::Database;
use crate::ocr::OCRService;
use crate::services::transaction_service::TransactionService;
use crate::services::product_service::ProductService;
use crate::services::user_service::UserService;
use crate::services::analytics_service::AnalyticsService;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    pub jwt_validator: Arc<JwtValidator>,
    pub ai_orchestrator: Arc<AIOrchestrator>,
    pub notification_hub: Option<Arc<NotificationHub>>,
    pub alert_engine: Arc<AlertEngine>,
    pub ocr_service: Arc<OCRService>,
    pub transaction_service: Arc<TransactionService>,
    pub product_service: Arc<ProductService>,
    pub user_service: Arc<UserService>,
    pub analytics_service: Arc<AnalyticsService>,
}
