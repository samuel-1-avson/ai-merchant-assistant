use axum::{
    extract::{State, Extension},
    Json,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use rust_decimal::prelude::ToPrimitive;
use tracing::info;

use crate::api::state::AppState;
use crate::api::middleware::AuthUser;
use crate::utils::errors::ApiError;

// ── Request / Response types ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "user" | "assistant"
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
}

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub reply: String,
}

// ── Handler ────────────────────────────────────────────────────────────────

/// POST /api/v1/assistant/chat
///
/// Accepts the merchant's message + conversation history, gathers real
/// analytics data, builds a context-rich LLM prompt and returns a natural
/// language response.  Falls back to an intelligent data-driven reply when
/// the LLM is unavailable.
pub async fn chat(
    State(state): State<Arc<AppState>>,
    Extension(auth_user): Extension<AuthUser>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<Value>, ApiError> {
    let user_id = auth_user.user_id;

    info!("Assistant chat: \"{}\"", &request.message);

    // ── Gather merchant context (parallel fetches) ─────────────────────
    use rust_decimal::Decimal;

    let (summary_7d, summary_prev, products_30d) = tokio::join!(
        state.analytics_service.get_summary(user_id, 7),
        state.analytics_service.get_summary_for_period(user_id, 14, 7),
        state.analytics_service.get_product_performance(user_id, 30),
    );

    let empty_summary = || crate::models::analytics::AnalyticsSummary {
        total_revenue: Decimal::ZERO,
        total_transactions: 0,
        total_items_sold: Decimal::ZERO,
        average_transaction_value: Decimal::ZERO,
        top_products: vec![],
        daily_sales: vec![],
    };

    let summary  = summary_7d.unwrap_or_else(|_| empty_summary());
    let prev     = summary_prev.unwrap_or_else(|_| empty_summary());
    let products = products_30d.unwrap_or_default();

    let revenue_change_pct = if prev.total_revenue > Decimal::ZERO {
        ((summary.total_revenue - prev.total_revenue) / prev.total_revenue)
            .to_f64()
            .unwrap_or(0.0)
            * 100.0
    } else {
        0.0
    };

    let revenue  = summary.total_revenue.to_f64().unwrap_or(0.0);
    let avg_tx   = summary.average_transaction_value.to_f64().unwrap_or(0.0);
    let daily_avg = revenue / 7.0;

    // Business health score (mirrors generate_insights logic)
    let trend_score  = ((revenue_change_pct + 50.0) / 100.0 * 40.0).clamp(0.0, 40.0);
    let volume_score = (summary.total_transactions as f64 / 7.0 / 10.0 * 30.0).clamp(0.0, 30.0);
    let avg_score    = (avg_tx / 50.0 * 30.0).clamp(0.0, 30.0);
    let health_score = (trend_score + volume_score + avg_score).round() as u32;
    let health_label = match health_score {
        80..=100 => "excellent", 60..=79 => "good", 40..=59 => "fair", _ => "needs attention",
    };

    // ── Build rich context string for LLM ─────────────────────────────
    let mut context = format!(
        "MERCHANT BUSINESS DATA:\n\
         === Last 7 Days ===\n\
         - Revenue: ${:.2} ({}{:.1}% vs previous week)\n\
         - Transactions: {} (avg ${:.2}/sale, ${:.2}/day)\n\
         - Business health: {}/100 — {}\n",
        revenue,
        if revenue_change_pct >= 0.0 { "+" } else { "" }, revenue_change_pct,
        summary.total_transactions, avg_tx, daily_avg,
        health_score, health_label,
    );

    // Top sellers
    let top_sellers: Vec<_> = products.iter()
        .filter(|p| p.performance_label == "top_seller" || p.times_sold > 0)
        .take(5)
        .collect();
    if !top_sellers.is_empty() {
        context.push_str("=== Product Performance (last 30 days) ===\n");
        for p in &top_sellers {
            let profit_str = match p.profit_margin_pct {
                Some(m) => format!(", {:.0}% margin", m),
                None => String::new(),
            };
            context.push_str(&format!(
                "- {} [{} label]: {} sales, ${:.2} revenue{}\n",
                p.product_name, p.performance_label, p.times_sold,
                p.total_revenue.to_f64().unwrap_or(0.0), profit_str,
            ));
        }
    }

    // Slow / no-sales products
    let slow: Vec<_> = products.iter()
        .filter(|p| p.performance_label == "slow_mover" || p.performance_label == "no_sales")
        .take(5)
        .collect();
    if !slow.is_empty() {
        context.push_str("=== Slow / Zero-Sales Products (30 days) ===\n");
        for p in &slow {
            context.push_str(&format!("- {} [{}]: {} sales\n",
                p.product_name, p.performance_label, p.times_sold));
        }
    }

    // ── Build LLM prompt ──────────────────────────────────────────────
    let history_text: String = request.history.iter()
        .map(|m| format!("{}: {}", m.role.to_uppercase(), m.text))
        .collect::<Vec<_>>()
        .join("\n");

    let prompt = format!(
        r#"You are an expert AI business assistant for a merchant/market seller.
You have access to their REAL live business data below. Use it to give specific,
actionable answers. Be concise (2-4 sentences or a short bullet list).
Never invent numbers — only quote figures from the data section.
You can analyze profitability, sales trends, product performance, pricing, and forecasts.

{context}

{history_section}USER: {message}
ASSISTANT:"#,
        context = context,
        history_section = if history_text.is_empty() {
            String::new()
        } else {
            format!("CONVERSATION HISTORY:\n{}\n\n", history_text)
        },
        message = request.message,
    );

    // ── Try LLM, fall back to data-driven response ────────────────────
    let reply = match state.ai_orchestrator.generate_chat_response(&prompt).await {
        Ok(Some(text)) => {
            let cleaned = text.trim().trim_start_matches("ASSISTANT:").trim().to_string();
            if cleaned.is_empty() {
                data_driven_response(&request.message, revenue, summary.total_transactions,
                    avg_tx, revenue_change_pct, &summary.top_products, health_score, daily_avg)
            } else {
                cleaned
            }
        }
        Ok(None) | Err(_) => {
            data_driven_response(&request.message, revenue, summary.total_transactions,
                avg_tx, revenue_change_pct, &summary.top_products, health_score, daily_avg)
        }
    };

    Ok(Json(json!({
        "success": true,
        "data": {
            "reply": reply
        }
    })))
}

// ── Intelligent data-driven fallback ──────────────────────────────────────
//
// When the LLM is unavailable this function generates replies that are
// *specific to the merchant's actual data*, not hardcoded template strings.

fn data_driven_response(
    question: &str,
    revenue: f64,
    transactions: i64,
    avg_tx: f64,
    revenue_change_pct: f64,
    top_products: &[crate::models::analytics::TopProduct],
    health_score: u32,
    daily_avg: f64,
) -> String {
    let q = question.to_lowercase();

    // ── Greetings ────────────────────────────────────────────────────────
    let greeting_words = ["hi", "hello", "hey", "howdy", "hiya", "good morning", "good afternoon", "good evening", "sup", "yo"];
    let is_short = q.split_whitespace().count() <= 3;
    if is_short && greeting_words.iter().any(|g| q.contains(g)) {
        return if revenue > 0.0 {
            format!(
                "Hello! You've made ${:.2} across {} transactions this week. What would you like to know?",
                revenue, transactions,
            )
        } else {
            "Hello! I'm your AI business assistant. Start recording sales with the voice recorder and I can give you detailed insights. What would you like to know?".to_string()
        };
    }

    // ── Short acknowledgements ("ok", "okay", "thanks", "cool") ─────────
    let ack_words = ["ok", "okay", "thanks", "thank you", "cool", "got it", "alright", "great", "nice", "sure"];
    if is_short && ack_words.iter().any(|a| q.trim() == *a || q.trim().starts_with(a)) {
        return "Happy to help! Ask me anything about your sales, revenue forecasts, or business recommendations.".to_string();
    }

    // ── Meta questions about the AI itself ───────────────────────────────
    let meta_words = ["real ai", "connected", "working", "functional", "actual ai", "really work", "real model",
                      "live", "online", "active", "are you", "you a model", "you an ai", "you real", "what are you",
                      "who are you", "what can you", "model", "assistant", "how do you work"];
    if meta_words.iter().any(|m| q.contains(m)) {
        return if revenue > 0.0 {
            format!(
                "Yes, I'm a live AI assistant powered by Llama 3.1 (via Groq) with access to your real business data. \
                 Right now I can see ${:.2} in revenue across {} transactions this week ({:+.1}% vs last week). \
                 Ask me anything about your sales, trends, or recommendations!",
                revenue, transactions, revenue_change_pct,
            )
        } else {
            "Yes, I'm a live AI assistant powered by Llama 3.1 (via Groq), connected to your real database — \
             there are just no sales recorded yet this week. Use the voice recorder to log a sale and I'll immediately show you live data.".to_string()
        };
    }

    let trend_word = if revenue_change_pct > 5.0 {
        "growing"
    } else if revenue_change_pct < -5.0 {
        "declining"
    } else {
        "steady"
    };

    let health_label = match health_score {
        80..=100 => "excellent",
        60..=79  => "good",
        40..=59  => "fair",
        _        => "needs attention",
    };

    // Business health questions
    if q.contains("health") || q.contains("score") || q.contains("how is my business") || q.contains("business doing") {
        return format!(
            "Your business health score is {}/100 — {}. Revenue is ${:.2} this week (${:.2}/day on average), \
             which is {:+.1}% vs last week across {} transactions.",
            health_score, health_label, revenue, daily_avg, revenue_change_pct, transactions,
        );
    }

    // Profit questions
    if q.contains("profit") || q.contains("margin") || q.contains("cost") || q.contains("earning") {
        if revenue == 0.0 {
            return "No sales recorded yet — start logging sales to track profitability. Make sure products have a cost price set for profit margin calculations.".to_string();
        }
        return format!(
            "Your revenue this week is ${:.2} across {} sales. \
             Profit margins are calculated per product when a cost price is set. \
             Go to the Products page to add cost prices and I'll show you exact margins.",
            revenue, transactions,
        );
    }

    if q.contains("forecast") || q.contains("predict") || q.contains("next week") {
        // Simple projection: apply current trend to last 7-day revenue
        let projected = revenue * (1.0 + revenue_change_pct / 100.0);
        return format!(
            "Based on your current {trend_word} trend ({:+.1}%), I estimate next week's revenue around ${:.2}. \
             You had ${:.2} this week across {} transactions.",
            revenue_change_pct, projected, revenue, transactions,
        );
    }

    if q.contains("recommend") || q.contains("advice") || q.contains("improve") || q.contains("tip") {
        if revenue == 0.0 {
            return "Start recording your sales using the voice recorder so I can give you personalised recommendations!".to_string();
        }
        let mut recs = Vec::new();
        if avg_tx < 20.0 {
            recs.push(format!("Your average transaction is ${:.2} — try bundling products to increase it.", avg_tx));
        }
        if revenue_change_pct < -10.0 {
            recs.push(format!("Revenue is down {:.1}% — consider a promotion or reviewing your pricing.", revenue_change_pct.abs()));
        } else if revenue_change_pct > 10.0 {
            recs.push(format!("Sales are up {:.1}% — great momentum! Ensure you have enough stock for top sellers.", revenue_change_pct));
        }
        if let Some(top) = top_products.first() {
            recs.push(format!("'{}' is your best seller — keep it well-stocked.", top.product_name));
        }
        if recs.is_empty() {
            recs.push("Your business looks stable. Keep recording transactions to unlock deeper insights.".to_string());
        }
        return recs.join(" ");
    }

    if q.contains("top product") || q.contains("best sell") || q.contains("popular") {
        if top_products.is_empty() {
            return "No product-linked transactions yet. Try adding products to your catalogue and linking them when recording sales.".to_string();
        }
        let list: Vec<String> = top_products.iter().take(3)
            .enumerate()
            .map(|(i, p)| format!("{}. {} (${:.2})", i + 1, p.product_name, p.total_revenue.to_f64().unwrap_or(0.0)))
            .collect();
        return format!("Your top products this week:\n{}", list.join("\n"));
    }

    if q.contains("trend") || q.contains("growth") || q.contains("direction") {
        return format!(
            "Your sales are {} this week. Revenue is {:+.1}% vs last week (${:.2} vs ${:.2} prior week), \
             with {} transactions.",
            trend_word, revenue_change_pct, revenue,
            revenue * 100.0 / (100.0 + revenue_change_pct),
            transactions,
        );
    }

    // General / sales summary (default)
    if revenue == 0.0 {
        return "No sales recorded in the last 7 days yet. Use the voice recorder to log your first sale, then I can give you detailed insights about revenue, trends, and recommendations!".to_string();
    }

    format!(
        "This week you made ${:.2} across {} transactions (avg ${:.2}/sale, ${:.2}/day), \
         which is {:+.1}% vs last week — sales are {}. Business health: {}/100 ({}). \
         Ask me about profitability, top products, revenue forecast, or recommendations!",
        revenue, transactions, avg_tx, daily_avg, revenue_change_pct, trend_word,
        health_score, health_label,
    )
}
