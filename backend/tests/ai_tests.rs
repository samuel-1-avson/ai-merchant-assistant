//! Tests for AI Agent functionality

use ai_merchant_backend::ai::orchestrator::{Intent, ConversationContext};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test intent classification from string
#[test]
fn test_intent_from_string() {
    assert!(matches!(Intent::from("record_sale"), Intent::RecordSale));
    assert!(matches!(Intent::from("sale"), Intent::RecordSale));
    assert!(matches!(Intent::from("SOLD"), Intent::RecordSale));
    assert!(matches!(Intent::from("query_analytics"), Intent::QueryAnalytics));
    assert!(matches!(Intent::from("analytics"), Intent::QueryAnalytics));
    assert!(matches!(Intent::from("update_inventory"), Intent::UpdateInventory));
    assert!(matches!(Intent::from("stock"), Intent::UpdateInventory));
    assert!(matches!(Intent::from("set_alert"), Intent::SetAlert));
    assert!(matches!(Intent::from("general_conversation"), Intent::GeneralConversation));
    assert!(matches!(Intent::from("unknown"), Intent::Unknown));
    assert!(matches!(Intent::from("random_text"), Intent::Unknown));
}

/// Test conversation context default
#[tokio::test]
async fn test_conversation_context() {
    let context = ConversationContext::default();
    
    assert!(context.recent_transactions.is_empty());
    assert!(context.current_intent.is_none());
    assert!(context.pending_confirmation.is_none());
}

/// Test shared context with RwLock
#[tokio::test]
async fn test_shared_context() {
    let context = Arc::new(RwLock::new(ConversationContext::default()));
    
    // Write to context
    {
        let mut ctx = context.write().await;
        ctx.current_intent = Some("record_sale".to_string());
        ctx.recent_transactions.push("tx-123".to_string());
    }
    
    // Read from context
    {
        let ctx = context.read().await;
        assert_eq!(ctx.current_intent, Some("record_sale".to_string()));
        assert_eq!(ctx.recent_transactions.len(), 1);
    }
}
