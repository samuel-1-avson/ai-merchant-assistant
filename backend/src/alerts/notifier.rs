use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;
use serde_json::json;

use crate::alerts::Alert;

#[derive(Clone, Debug)]
pub enum NotificationEvent {
    NewAlert(Alert),
    AlertRead { alert_id: Uuid, user_id: Uuid },
    TransactionUpdate { user_id: Uuid, transaction_id: Uuid },
    SystemMessage { user_id: Uuid, message: String },
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
        self.sender.send(event)?;
        Ok(())
    }

    pub fn notify_new_alert(&self, alert: Alert) {
        let _ = self.notify(NotificationEvent::NewAlert(alert));
    }

    pub fn notify_transaction_update(&self, user_id: Uuid, transaction_id: Uuid) {
        let _ = self.notify(NotificationEvent::TransactionUpdate { user_id, transaction_id });
    }
}

impl Default for NotificationHub {
    fn default() -> Self {
        Self::new()
    }
}
