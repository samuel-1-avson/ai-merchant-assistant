//! Transaction Confirmation System
//!
//! This module handles pending transaction confirmations, allowing users
//! to review and approve/reject transactions before they're committed.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn};

use crate::models::transaction::ExtractedEntities;
use crate::db::repositories::product_repo::ProductMatch;

/// Status of a pending confirmation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConfirmationStatus {
    Pending,
    Confirmed,
    Rejected,
    Expired,
}

/// Serializable product info for confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductInfo {
    pub id: Uuid,
    pub name: String,
    pub match_score: i64,
}

impl From<ProductMatch> for ProductInfo {
    fn from(pm: ProductMatch) -> Self {
        Self {
            id: pm.product.id,
            name: pm.product.name,
            match_score: pm.score,
        }
    }
}

/// A pending transaction awaiting user confirmation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingConfirmation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub status: ConfirmationStatus,
    pub extracted_entities: ExtractedEntities,
    pub proposed_product: Option<ProductInfo>,
    pub confidence: f64,
    pub is_new_product: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub original_transcription: String,
    /// Calculated fields for display
    pub display_name: String,
    pub display_quantity: String,
    pub display_price: String,
    pub display_total: String,
}

impl PendingConfirmation {
    /// Check if this confirmation has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get remaining time in seconds
    pub fn remaining_seconds(&self) -> u64 {
        let remaining = self.expires_at.timestamp() - Utc::now().timestamp();
        remaining.max(0) as u64
    }

    /// Format the confirmation for user display
    pub fn format_for_display(&self) -> String {
        let product_name = self.proposed_product.as_ref()
            .map(|p| p.name.clone())
            .or_else(|| self.extracted_entities.product.clone())
            .unwrap_or_else(|| "Unknown Product".to_string());

        let qty = self.extracted_entities.quantity.unwrap_or(1.0);
        let unit = self.extracted_entities.unit.as_deref().unwrap_or("units");
        let price = self.extracted_entities.price.unwrap_or(0.0);
        let total = qty * price;

        format!(
            "Sale: {} {} of {} at ${:.2} each = ${:.2} total",
            qty, unit, product_name, price, total
        )
    }
}

/// Configuration for the confirmation system
#[derive(Debug, Clone)]
pub struct ConfirmationConfig {
    /// How long confirmations remain valid in seconds (default: 5 minutes)
    pub expiration_seconds: i64,
    /// Cleanup interval in seconds (default: 1 minute)
    pub cleanup_interval: Duration,
    /// Auto-confirm threshold (0.0 - 1.0, transactions above this skip confirmation)
    pub auto_confirm_threshold: f64,
}

impl Default for ConfirmationConfig {
    fn default() -> Self {
        Self {
            expiration_seconds: 300, // 5 minutes
            cleanup_interval: Duration::from_secs(60), // 1 minute
            auto_confirm_threshold: 0.9, // 90% confidence auto-approves
        }
    }
}

/// Manager for pending transaction confirmations
pub struct ConfirmationManager {
    confirmations: Arc<RwLock<HashMap<Uuid, PendingConfirmation>>>,
    config: ConfirmationConfig,
}

impl ConfirmationManager {
    /// Create a new confirmation manager
    pub fn new() -> Self {
        Self::with_config(ConfirmationConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: ConfirmationConfig) -> Self {
        let manager = Self {
            confirmations: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // Start cleanup task
        manager.start_cleanup_task();

        manager
    }

    /// Create a new pending confirmation
    pub async fn create_confirmation(
        &self,
        user_id: Uuid,
        entities: ExtractedEntities,
        product_match: Option<ProductMatch>,
        confidence: f64,
        is_new_product: bool,
        transcription: String,
    ) -> PendingConfirmation {
        let id = Uuid::new_v4();
        let now = Utc::now();

        // Format display values
        let product_name = product_match.as_ref()
            .map(|p| p.product.name.clone())
            .or_else(|| entities.product.clone())
            .unwrap_or_else(|| "Unknown Product".to_string());

        let qty = entities.quantity.unwrap_or(1.0);
        let unit = entities.unit.clone().unwrap_or_else(|| "units".to_string());
        let price = entities.price.unwrap_or(0.0);
        let total = qty * price;

        let confirmation = PendingConfirmation {
            id,
            user_id,
            status: ConfirmationStatus::Pending,
            extracted_entities: entities,
            proposed_product: product_match.map(|pm| pm.into()),
            confidence,
            is_new_product,
            created_at: now,
            expires_at: now + chrono::Duration::seconds(self.config.expiration_seconds),
            original_transcription: transcription,
            display_name: product_name,
            display_quantity: format!("{} {}", qty, unit),
            display_price: format!("${:.2}", price),
            display_total: format!("${:.2}", total),
        };

        let mut confirmations = self.confirmations.write().await;
        confirmations.insert(id, confirmation.clone());

        info!(
            "Created pending confirmation {} for user {} (confidence: {:.0}%)",
            id, user_id, confidence * 100.0
        );

        confirmation
    }

    /// Get a pending confirmation by ID
    pub async fn get_confirmation(&self, id: &Uuid) -> Option<PendingConfirmation> {
        let confirmations = self.confirmations.read().await;
        confirmations.get(id).cloned()
    }

    /// Get all pending confirmations for a user
    pub async fn get_user_confirmations(&self, user_id: Uuid) -> Vec<PendingConfirmation> {
        let confirmations = self.confirmations.read().await;
        confirmations
            .values()
            .filter(|c| c.user_id == user_id && c.status == ConfirmationStatus::Pending && !c.is_expired())
            .cloned()
            .collect()
    }

    /// Confirm a pending transaction
    pub async fn confirm(&self, id: &Uuid, user_id: Uuid) -> Result<PendingConfirmation, ConfirmationError> {
        let mut confirmations = self.confirmations.write().await;

        let confirmation = confirmations
            .get_mut(id)
            .ok_or(ConfirmationError::NotFound)?;

        // Verify ownership
        if confirmation.user_id != user_id {
            return Err(ConfirmationError::Unauthorized);
        }

        // Check if already processed
        if confirmation.status != ConfirmationStatus::Pending {
            return Err(ConfirmationError::AlreadyProcessed);
        }

        // Check expiration
        if confirmation.is_expired() {
            confirmation.status = ConfirmationStatus::Expired;
            return Err(ConfirmationError::Expired);
        }

        // Mark as confirmed
        confirmation.status = ConfirmationStatus::Confirmed;
        info!("Confirmation {} confirmed by user {}", id, user_id);

        Ok(confirmation.clone())
    }

    /// Reject a pending transaction
    pub async fn reject(&self, id: &Uuid, user_id: Uuid) -> Result<PendingConfirmation, ConfirmationError> {
        let mut confirmations = self.confirmations.write().await;

        let confirmation = confirmations
            .get_mut(id)
            .ok_or(ConfirmationError::NotFound)?;

        // Verify ownership
        if confirmation.user_id != user_id {
            return Err(ConfirmationError::Unauthorized);
        }

        // Check if already processed
        if confirmation.status != ConfirmationStatus::Pending {
            return Err(ConfirmationError::AlreadyProcessed);
        }

        // Mark as rejected
        confirmation.status = ConfirmationStatus::Rejected;
        info!("Confirmation {} rejected by user {}", id, user_id);

        Ok(confirmation.clone())
    }

    /// Check if a transaction should auto-confirm based on confidence
    pub fn should_auto_confirm(&self, confidence: f64, is_new_product: bool) -> bool {
        // Auto-confirm if confidence is high and product is known
        confidence >= self.config.auto_confirm_threshold && !is_new_product
    }

    /// Remove a confirmation
    pub async fn remove(&self, id: &Uuid) {
        let mut confirmations = self.confirmations.write().await;
        confirmations.remove(id);
    }

    /// Get count of pending confirmations for a user
    pub async fn pending_count(&self, user_id: Uuid) -> usize {
        let confirmations = self.confirmations.read().await;
        confirmations
            .values()
            .filter(|c| c.user_id == user_id && c.status == ConfirmationStatus::Pending && !c.is_expired())
            .count()
    }

    /// Start the background cleanup task
    fn start_cleanup_task(&self) {
        let confirmations = self.confirmations.clone();
        let interval = self.config.cleanup_interval;

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;

                let mut confirmations = confirmations.write().await;
                let before_count = confirmations.len();

                // Remove expired confirmations
                confirmations.retain(|_id, confirmation| {
                    let should_retain = !confirmation.is_expired() || 
                        confirmation.status != ConfirmationStatus::Pending;
                    
                    if !should_retain {
                        info!("Cleaned up expired confirmation {}", confirmation.id);
                    }
                    
                    should_retain
                });

                let after_count = confirmations.len();
                if before_count != after_count {
                    info!("Cleanup: removed {} expired confirmations", before_count - after_count);
                }
            }
        });
    }
}

impl Default for ConfirmationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during confirmation operations
#[derive(Debug, Clone, PartialEq)]
pub enum ConfirmationError {
    NotFound,
    Unauthorized,
    AlreadyProcessed,
    Expired,
}

impl std::fmt::Display for ConfirmationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfirmationError::NotFound => write!(f, "Confirmation not found"),
            ConfirmationError::Unauthorized => write!(f, "Not authorized to access this confirmation"),
            ConfirmationError::AlreadyProcessed => write!(f, "Confirmation already processed"),
            ConfirmationError::Expired => write!(f, "Confirmation has expired"),
        }
    }
}

impl std::error::Error for ConfirmationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_confirmation_lifecycle() {
        let manager = ConfirmationManager::new();
        let user_id = Uuid::new_v4();

        let entities = ExtractedEntities {
            product: Some("Apples".to_string()),
            quantity: Some(5.0),
            unit: Some("kg".to_string()),
            price: Some(2.99),
            currency: Some("USD".to_string()),
        };

        // Create confirmation
        let confirmation = manager.create_confirmation(
            user_id,
            entities,
            None,
            0.85,
            false,
            "Sold 5 kg of apples for $2.99".to_string(),
        ).await;

        assert_eq!(confirmation.status, ConfirmationStatus::Pending);
        assert!(!confirmation.is_expired());

        // Confirm it
        let confirmed = manager.confirm(&confirmation.id, user_id).await.unwrap();
        assert_eq!(confirmed.status, ConfirmationStatus::Confirmed);

        // Try to confirm again - should fail
        let result = manager.confirm(&confirmation.id, user_id).await;
        assert!(matches!(result, Err(ConfirmationError::AlreadyProcessed)));
    }

    #[test]
    fn test_auto_confirm_threshold() {
        let config = ConfirmationConfig {
            auto_confirm_threshold: 0.9,
            ..Default::default()
        };
        let manager = ConfirmationManager::with_config(config);

        // High confidence + known product = auto-confirm
        assert!(manager.should_auto_confirm(0.95, false));

        // Low confidence = manual confirm
        assert!(!manager.should_auto_confirm(0.5, false));

        // New product = manual confirm (even with high confidence)
        assert!(!manager.should_auto_confirm(0.95, true));
    }
}
