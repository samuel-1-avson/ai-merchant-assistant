use tokio::sync::broadcast;
use uuid::Uuid;

use crate::alerts::Alert;

#[derive(Clone, Debug)]
pub enum NotificationEvent {
    NewAlert(Alert),
    AlertRead { alert_id: Uuid, user_id: Uuid },
    TransactionUpdate { user_id: Uuid, transaction_id: Uuid },
    SystemMessage { user_id: Uuid, message: String },
}

/// Alert with metadata for API responses
#[derive(Clone, Debug, serde::Serialize)]
pub struct AlertItem {
    pub id: Uuid,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub struct NotificationHub {
    sender: broadcast::Sender<NotificationEvent>,
}

impl NotificationHub {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(100);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<NotificationEvent> {
        self.sender.subscribe()
    }

    pub fn notify(&self, event: NotificationEvent) -> anyhow::Result<()> {
        let _ = self.sender.send(event);
        Ok(())
    }

    pub fn notify_new_alert(&self, alert: Alert) {
        let _ = self.notify(NotificationEvent::NewAlert(alert));
    }

    pub fn notify_transaction_update(&self, user_id: Uuid, transaction_id: Uuid) {
        let _ = self.notify(NotificationEvent::TransactionUpdate { user_id, transaction_id });
    }

    /// Get unread notifications for a user
    /// TODO: Connect to database for real alerts
    pub async fn get_unread_notifications(&self, _user_id: &Uuid) -> anyhow::Result<Vec<AlertItem>> {
        // For now, return empty list - should query database
        Ok(vec![])
    }

    /// Mark an alert as read
    /// TODO: Update in database
    pub async fn mark_as_read(&self, _alert_id: &Uuid, _user_id: &Uuid) -> anyhow::Result<bool> {
        // For now, just acknowledge - should update database
        Ok(true)
    }

    /// Check and generate new alerts based on system state
    /// TODO: Implement real alert generation logic
    pub async fn check_and_generate_alerts(&self, _user_id: &Uuid) -> anyhow::Result<Vec<AlertItem>> {
        // For now, return empty list - should check conditions and generate alerts
        Ok(vec![])
    }
}

impl Default for NotificationHub {
    fn default() -> Self {
        Self::new()
    }
}
