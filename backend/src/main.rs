use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, State},
    middleware,
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
use crate::ai::clients::AIClientBuilder;
use crate::ai::SessionStore;
use crate::alerts::{AlertEngine, notifier::NotificationHub};
use crate::ocr::OCRService;
use crate::security::{RateLimiter, RateLimitConfig};
use crate::config::AppConfig;
use crate::db::Database;
use crate::db::repositories::transaction_repo::TransactionRepository;
use crate::db::repositories::product_repo::ProductRepository;
use crate::db::repositories::user_repo::UserRepository;
use crate::api::state::AppState;
use crate::services::transaction_service::TransactionService;
use crate::services::product_service::ProductService;
use crate::services::user_service::UserService;
use crate::services::analytics_service::AnalyticsService;
use crate::api::middleware::auth::jwt_auth_middleware;
use crate::auth::JwtValidator;

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

    // Initialize AI clients with failover support
    info!("🤖 Initializing AI clients with failover support...");
    
    let stt_client = AIClientBuilder::new()
        .with_huggingface(config.huggingface_api_token.clone())
        .with_runpod(config.runpod_endpoint_url.clone(), config.runpod_api_key.clone())
        .build_stt_client();
    
    let llm_client = AIClientBuilder::new()
        .with_huggingface(config.huggingface_api_token.clone())
        .with_groq(config.groq_api_key.clone())
        .with_together(config.together_api_key.clone())
        .with_runpod(config.runpod_endpoint_url.clone(), config.runpod_api_key.clone())
        .build_llm_client();
    
    let tts_client = AIClientBuilder::new()
        .with_huggingface(config.huggingface_api_token.clone())
        .build_tts_client();

    let vision_client = AIClientBuilder::new()
        .with_huggingface(config.huggingface_api_token.clone())
        .build_vision_client();

    info!("✅ AI clients initialized with provider failover");

    // Initialize repositories
    let transaction_repo = Arc::new(TransactionRepository::new(db.pool.clone()));
    let product_repo = Arc::new(ProductRepository::new(db.pool.clone()));
    let user_repo = Arc::new(UserRepository::new(db.pool.clone()));

    // Initialize session store for cross-request conversation memory
    let session_store = Arc::new(SessionStore::new());

    // Initialize AI Orchestrator
    let ai_orchestrator = Arc::new(AIOrchestrator::new(
        stt_client,
        llm_client,
        tts_client,
        vision_client.clone(),
        transaction_repo.clone(),
        product_repo.clone(),
        session_store,
    ));
    info!("✅ AI Orchestrator initialized with failover support");

    // Initialize Notification Hub
    let notification_hub = Arc::new(NotificationHub::new());
    info!("✅ Notification Hub initialized");

    // Initialize Alert Engine
    let alert_engine = Arc::new(AlertEngine::new(db.pool.clone()));
    info!("✅ Alert Engine initialized");

    // Initialize Rate Limiter (middleware integration pending)
    let _rate_limiter = Arc::new(RateLimiter::new(RateLimitConfig::default()));
    info!("✅ Rate Limiter initialized");

    // Initialize OCR Service (uses vision model llava-hf/llava-1.5-7b-hf)
    let ocr_service = Arc::new(OCRService::new(
        vision_client.clone(),
        product_repo.clone(),
    ));
    info!("✅ OCR Service initialized with LLaVA-1.5-7B vision model");

    // Initialize Services
    let transaction_service = Arc::new(TransactionService::new(transaction_repo));
    let product_service = Arc::new(ProductService::new(product_repo));
    let user_service = Arc::new(UserService::new(user_repo));
    let analytics_service = Arc::new(AnalyticsService::new(db.pool.clone()));
    info!("✅ Services initialized");

    // Initialize JWT validator — fetch Supabase JWKS (ES256) with fallback to HS256
    let jwt_validator = Arc::new(
        build_jwt_validator(&config.supabase_url, &config.supabase_jwt_secret).await
    );

    // Create shared state
    let state = Arc::new(AppState {
        config,
        db,
        jwt_validator,
        ai_orchestrator,
        notification_hub: Some(notification_hub),
        alert_engine,
        ocr_service,
        transaction_service,
        product_service,
        user_service,
        analytics_service,
    });

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // ── Public routes (no authentication required) ────────────────────────
    let public_routes = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/auth/register", post(api::routes::auth::register))
        .route("/api/v1/auth/login", post(api::routes::auth::login))
        .route("/api/v1/auth/google", post(api::routes::oauth::google_auth))
        .route("/api/v1/auth/github", post(api::routes::oauth::github_auth))
        // i18n is public (no user data)
        .route("/api/v1/i18n/translations", get(api::routes::i18n::get_translations))
        .route("/api/v1/i18n/languages", get(api::routes::i18n::get_supported_languages))
        .route("/api/v1/i18n/format-number", get(api::routes::i18n::format_number))
        // WebSocket: authenticates via ?token= query param inside the handler
        .route("/ws", get(realtime::websocket::handler));

    // ── Protected routes (JWT required) ──────────────────────────────────
    let protected_routes = Router::new()
        // Transactions
        .route("/api/v1/transactions", get(api::routes::transactions::list).post(api::routes::transactions::create))
        .route("/api/v1/transactions/voice", post(api::routes::transactions::create_voice))
        .route("/api/v1/transactions/confirmations", get(api::routes::transactions::pending_confirmations))
        .route("/api/v1/transactions/confirmations/:id/confirm", post(api::routes::transactions::confirm_by_id))
        .route("/api/v1/transactions/confirmations/:id/reject", post(api::routes::transactions::reject_by_id))
        .route("/api/v1/transactions/confirm", post(api::routes::transactions::confirm))
        .route("/api/v1/transactions/reject", post(api::routes::transactions::reject))
        .route("/api/v1/transactions/pending", get(api::routes::transactions::pending_confirmations))

        // Products
        .route("/api/v1/products", get(api::routes::products::list).post(api::routes::products::create))

        // Analytics
        .route("/api/v1/analytics/summary", get(api::routes::analytics::summary))
        .route("/api/v1/analytics/trends", get(api::routes::analytics::trends))
        .route("/api/v1/analytics/forecast", get(api::routes::analytics::forecast))
        .route("/api/v1/analytics/insights", get(api::routes::analytics::insights))
        .route("/api/v1/analytics/products", get(api::routes::analytics::product_performance))

        // Alerts
        .route("/api/v1/alerts", get(api::routes::alerts::list))
        .route("/api/v1/alerts/counts", get(api::routes::alerts::counts))
        .route("/api/v1/alerts/:id/read", post(api::routes::alerts::mark_read))
        .route("/api/v1/alerts/read-all", post(api::routes::alerts::mark_all_read))
        .route("/api/v1/alerts/check", post(api::routes::alerts::check_now))

        // Voice (stateless AI — no user data, but still require auth)
        .route("/api/v1/voice/transcribe", post(api::routes::voice::transcribe))
        .route("/api/v1/voice/synthesize", post(api::routes::voice::synthesize))

        // OCR
        .route("/api/v1/ocr/receipt", post(api::routes::ocr::process_receipt))
        .route("/api/v1/ocr/receipt/transactions", post(api::routes::ocr::create_transactions))
        .route("/api/v1/ocr/test", get(api::routes::ocr::test_ocr))
        .route("/api/v1/ocr/product", post(api::routes::ocr::scan_product))

        // AI Assistant chat
        .route("/api/v1/assistant/chat", post(api::routes::assistant::chat))

        // Apply JWT auth middleware to all routes in this group
        .layer(middleware::from_fn_with_state(state.clone(), jwt_auth_middleware));

    // ── Assemble app ──────────────────────────────────────────────────────
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // Global middleware (order matters - applied in reverse)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(DefaultBodyLimit::max(25 * 1024 * 1024)) // 25 MB — needed for base64-encoded audio
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

/// Build an ES256 JWT validator from Supabase JWKS.
/// Priority: SUPABASE_JWKS env var → HTTP fetch → HS256 fallback.
async fn build_jwt_validator(supabase_url: &str, fallback_secret: &str) -> JwtValidator {
    // 1. Try SUPABASE_JWKS env var (avoids network call in containerised environments)
    if let Ok(jwks_json) = std::env::var("SUPABASE_JWKS") {
        match serde_json::from_str::<jsonwebtoken::jwk::JwkSet>(&jwks_json)
            .map_err(|e| anyhow::anyhow!(e))
            .and_then(|jwks| {
                jwks.keys.into_iter().next()
                    .ok_or_else(|| anyhow::anyhow!("JWKS env has no keys"))
            })
            .and_then(|jwk| JwtValidator::from_jwk(&jwk))
        {
            Ok(v) => {
                info!("✅ JWT validator initialized from SUPABASE_JWKS env (ES256)");
                return v;
            }
            Err(e) => tracing::warn!("SUPABASE_JWKS env var invalid: {}", e),
        }
    }

    // 2. Try fetching JWKS from Supabase
    let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", supabase_url);
    let result: anyhow::Result<JwtValidator> = async {
        let resp = reqwest::get(&jwks_url).await
            .map_err(|e| anyhow::anyhow!("JWKS fetch: {}", e))?;
        let jwks: jsonwebtoken::jwk::JwkSet = resp.json().await
            .map_err(|e| anyhow::anyhow!("JWKS parse: {}", e))?;
        let jwk = jwks.keys.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("JWKS has no keys"))?;
        JwtValidator::from_jwk(&jwk)
    }.await;

    match result {
        Ok(v) => {
            info!("✅ JWT validator initialized from JWKS endpoint (ES256)");
            v
        }
        Err(e) => {
            tracing::warn!("⚠️  JWKS unavailable ({}), falling back to HS256", e);
            JwtValidator::from_secret(fallback_secret)
        }
    }
}
