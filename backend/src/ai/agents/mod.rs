pub mod stt_agent;
pub mod nlu_agent;
pub mod transaction_agent;
pub mod tts_agent;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub message_id: Uuid,
    pub sender: AgentType,
    pub recipient: AgentType,
    pub message_type: MessageType,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentType {
    MasterOrchestrator,
    STTAgent,
    NLUAgent,
    TransactionAgent,
    AnalyticsAgent,
    TTSAgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    TaskRequest,
    TaskResponse,
    EventNotification,
    ErrorReport,
    ContextUpdate,
}
