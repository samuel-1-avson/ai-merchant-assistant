//! Conversation session memory for the AI Merchant Assistant.
//!
//! Stores per-user session context in a shared in-memory map so that
//! every request in the same user session can access the conversation history.
//! Sessions expire after 30 minutes of inactivity.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::info;

/// Maximum number of recent transactions kept per session
const MAX_RECENT_TRANSACTIONS: usize = 10;
/// Session time-to-live (30 minutes of inactivity)
const SESSION_TTL_SECS: u64 = 30 * 60;
/// Background cleanup interval
const CLEANUP_INTERVAL_SECS: u64 = 5 * 60;

/// Per-user conversation context that persists across HTTP requests.
#[derive(Debug, Clone)]
pub struct ConversationSession {
    pub user_id: Uuid,
    /// Summaries of the last N transactions (e.g. "Sold 3 kg Apples for $5.00")
    pub recent_transactions: VecDeque<String>,
    /// Last intent the user expressed
    pub last_intent: Option<String>,
    /// Last product name mentioned (used for follow-up corrections)
    pub last_product: Option<String>,
    /// Last quantity mentioned (for corrections like "make that 10")
    pub last_quantity: Option<f64>,
    /// Last price mentioned
    pub last_price: Option<f64>,
    /// When this session was last accessed
    pub last_active: Instant,
}

impl ConversationSession {
    fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            recent_transactions: VecDeque::new(),
            last_intent: None,
            last_product: None,
            last_quantity: None,
            last_price: None,
            last_active: Instant::now(),
        }
    }

    /// Record a completed transaction in short-term memory.
    pub fn record_transaction(&mut self, summary: String) {
        if self.recent_transactions.len() >= MAX_RECENT_TRANSACTIONS {
            self.recent_transactions.pop_front();
        }
        self.recent_transactions.push_back(summary);
        self.touch();
    }

    /// Update intent and entity memory after each AI processing cycle.
    pub fn update_context(
        &mut self,
        intent: &str,
        product: Option<String>,
        quantity: Option<f64>,
        price: Option<f64>,
    ) {
        self.last_intent = Some(intent.to_string());
        if product.is_some() { self.last_product = product; }
        if quantity.is_some() { self.last_quantity = quantity; }
        if price.is_some() { self.last_price = price; }
        self.touch();
    }

    /// Build a context string that can be prepended to prompts.
    pub fn build_context_preamble(&self) -> String {
        if self.recent_transactions.is_empty() {
            return String::new();
        }
        let history: Vec<&str> = self.recent_transactions.iter().map(|s| s.as_str()).collect();
        format!(
            "Recent transactions in this session:\n{}\n\n",
            history.join("\n")
        )
    }

    fn touch(&mut self) {
        self.last_active = Instant::now();
    }

    fn is_expired(&self) -> bool {
        self.last_active.elapsed() > Duration::from_secs(SESSION_TTL_SECS)
    }
}

/// Shared, thread-safe store for all active user sessions.
///
/// Store a single `Arc<SessionStore>` in `AppState` so every request handler
/// can read and write session context without locking the whole orchestrator.
#[derive(Clone)]
pub struct SessionStore {
    sessions: Arc<RwLock<HashMap<Uuid, ConversationSession>>>,
}

impl SessionStore {
    pub fn new() -> Self {
        let store = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };
        store.start_cleanup_task();
        store
    }

    /// Get a snapshot of the session for a user (creates one if missing).
    pub async fn get_session(&self, user_id: Uuid) -> ConversationSession {
        let sessions = self.sessions.read().await;
        sessions
            .get(&user_id)
            .cloned()
            .unwrap_or_else(|| ConversationSession::new(user_id))
    }

    /// Persist an updated session back to the store.
    pub async fn save_session(&self, session: ConversationSession) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.user_id, session);
    }

    /// Convenience: record a completed transaction in the user's session.
    pub async fn record_transaction(&self, user_id: Uuid, summary: String) {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .entry(user_id)
            .or_insert_with(|| ConversationSession::new(user_id));
        session.record_transaction(summary);
    }

    /// Convenience: update intent + entity memory for a user.
    pub async fn update_context(
        &self,
        user_id: Uuid,
        intent: &str,
        product: Option<String>,
        quantity: Option<f64>,
        price: Option<f64>,
    ) {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .entry(user_id)
            .or_insert_with(|| ConversationSession::new(user_id));
        session.update_context(intent, product, quantity, price);
    }

    /// Remove a session (e.g. on user logout).
    pub async fn clear_session(&self, user_id: Uuid) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(&user_id);
    }

    /// Background task: remove sessions that have been idle > SESSION_TTL_SECS.
    fn start_cleanup_task(&self) {
        let sessions = self.sessions.clone();
        tokio::spawn(async move {
            let interval = Duration::from_secs(CLEANUP_INTERVAL_SECS);
            loop {
                tokio::time::sleep(interval).await;
                let mut map = sessions.write().await;
                let before = map.len();
                map.retain(|_, s| !s.is_expired());
                let removed = before - map.len();
                if removed > 0 {
                    info!("SessionStore: removed {} expired sessions", removed);
                }
            }
        });
    }
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}
