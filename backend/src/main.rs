use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber;

mod ai;
mod alerts;
mod analytics;
mod api;
mod auth;
mod config;
mod db;
mod external;
mod i18n;
mod models;
mod ocr;
mod realtime;
mod security;
mod services;
mod utils;

use crate::ai::orchestrator::AIOrchestrator;
use crate::ai::clients::CloudClientFactory;
use crate::alerts::notifier::NotificationHub;
use crate::config::AppConfig;
use crate::db::Database;
use crate::db::repositories::transaction_repo::TransactionRepository;
use crate::db::repositories::product_repo::ProductRepository;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    pub ai_orchestrator: Arc<AIOrchestrator>,
    pub notification_hub: Option<Arc<NotificationHub>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("🏁 Starting AI Merchant Assistant Backend...");

    // Load configuration
    let config = AppConfig::from_env()?;
    info!("✅ Configuration loaded successfully");

    // Initialize database
    let db = Database::new(&config.database_url).await?;
    info!("✅ Database connected successfully");

    // Initialize AI clients
    let stt_client = CloudClientFactory::create_stt_client(
        &config.ai_provider,
        config.huggingface_api_token.clone()
    );
    let llm_client = CloudClientFactory::create_llm_client(
        &config.ai_provider,
        config.huggingface_api_token.clone()
    );
    let tts_client = CloudClientFactory::create_tts_client(
        &config.ai_provider,
        config.huggingface_api_token.clone()
    );

    // Initialize repositories
    let transaction_repo = Arc::new(TransactionRepository::new(db.pool.clone()));
    let product_repo = Arc::new(ProductRepository::new(db.pool.clone()));

    // Initialize AI Orchestrator
    let ai_orchestrator = Arc::new(AIOrchestrator::new(
        Arc::from(stt_client),
        Arc::from(llm_client),
        Arc::from(tts_client),
        transaction_repo,
        product_repo,
    ));
    info!("✅ AI Orchestrator initialized");

    // Initialize Notification Hub
    let notification_hub = Arc::new(NotificationHub::new());
    info!("✅ Notification Hub initialized");

    // Create shared state
    let state = Arc::new(AppState {
        config,
        db,
        ai_orchestrator,
        notification_hub: Some(notification_hub),
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // Authentication
        .route("/api/v1/auth/register", post(api::routes::auth::register))
        .route("/api/v1/auth/login", post(api::routes::auth::login))
        
        // Transactions
        .route("/api/v1/transactions", get(api::routes::transactions::list).post(api::routes::transactions::create))
        .route("/api/v1/transactions/voice", post(api::routes::transactions::create_voice))
        
        // Products
        .route("/api/v1/products", get(api::routes::products::list).post(api::routes::products::create))
        
        // Analytics
        .route("/api/v1/analytics/summary", get(api::routes::analytics::summary))
        .route("/api/v1/analytics/trends", get(api::routes::analytics::trends))
        .route("/api/v1/analytics/forecast", get(api::routes::analytics::forecast))
        .route("/api/v1/analytics/insights", get(api::routes::analytics::insights))
        
        // Alerts
        .route("/api/v1/alerts", get(api::routes::alerts::list))
        .route("/api/v1/alerts/:id/read", post(api::routes::alerts::mark_read))
        .route("/api/v1/alerts/check", post(api::routes::alerts::check_now))
        
        // Voice
        .route("/api/v1/voice/transcribe", post(api::routes::voice::transcribe))
        
        // OCR
        .route("/api/v1/ocr/receipt", post(api::routes::ocr::process_receipt))
        .route("/api/v1/ocr/product", post(api::routes::ocr::scan_product))
        
        // i18n
        .route("/api/v1/i18n/translations", get(api::routes::i18n::get_translations))
        .route("/api/v1/i18n/languages", get(api::routes::i18n::get_supported_languages))
        .route("/api/v1/i18n/format-number", get(api::routes::i18n::format_number))
        
        // WebSocket
        .route("/ws", get(realtime::websocket::handler))
        
        // Middleware
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = SocketAddr::from(([0, 0, 0, 0], port.parse()?));
    info!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
