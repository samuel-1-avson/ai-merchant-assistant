use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tracing::{info, error};

use crate::ai::clients::{CloudSTTClient, CloudLLMClient, CloudTTSClient, TranscriptionResult};
use crate::ai::agents::{stt_agent::STTAgent, nlu_agent::NLUAgent, transaction_agent::TransactionAgent, tts_agent::TTSAgent};
use crate::models::transaction::{VoiceTransactionResponse, ExtractedEntities};
use crate::db::repositories::transaction_repo::TransactionRepository;
use crate::db::repositories::product_repo::ProductRepository;

#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub user_id: Uuid,
    pub recent_transactions: Vec<String>,
    pub current_intent: Option<String>,
    pub pending_confirmation: Option<ExtractedEntities>,
}

impl Default for ConversationContext {
    fn default() -> Self {
        Self {
            user_id: Uuid::new_v4(),
            recent_transactions: Vec::new(),
            current_intent: None,
            pending_confirmation: None,
        }
    }
}

pub struct AIOrchestrator {
    stt_agent: STTAgent,
    nlu_agent: NLUAgent,
    transaction_agent: TransactionAgent,
    tts_agent: TTSAgent,
    context: Arc<RwLock<ConversationContext>>,
}

#[derive(Debug, Clone)]
pub enum Intent {
    RecordSale,
    QueryAnalytics,
    UpdateInventory,
    SetAlert,
    GeneralConversation,
    Unknown,
}

impl From<&str> for Intent {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "record_sale" | "sale" | "sold" | "transaction" => Intent::RecordSale,
            "query_analytics" | "analytics" | "stats" | "report" => Intent::QueryAnalytics,
            "update_inventory" | "inventory" | "stock" => Intent::UpdateInventory,
            "set_alert" | "alert" | "notification" => Intent::SetAlert,
            "general_conversation" | "chat" | "talk" => Intent::GeneralConversation,
            _ => Intent::Unknown,
        }
    }
}

impl AIOrchestrator {
    pub fn new(
        stt_client: Arc<dyn CloudSTTClient>,
        llm_client: Arc<dyn CloudLLMClient>,
        tts_client: Arc<dyn CloudTTSClient>,
        transaction_repo: Arc<TransactionRepository>,
        product_repo: Arc<ProductRepository>,
    ) -> Self {
        Self {
            stt_agent: STTAgent::new(stt_client),
            nlu_agent: NLUAgent::new(llm_client.clone()),
            transaction_agent: TransactionAgent::new(transaction_repo, product_repo),
            tts_agent: TTSAgent::new(tts_client),
            context: Arc::new(RwLock::new(ConversationContext::default())),
        }
    }

    pub async fn process_voice_transaction(
        &self,
        audio_bytes: Vec<u8>,
        user_id: Uuid,
    ) -> anyhow::Result<VoiceTransactionResponse> {
        info!("Starting voice transaction processing for user: {}", user_id);

        // Step 1: Speech-to-Text
        info!("Step 1: Transcribing audio...");
        let transcription = self.stt_agent.transcribe(audio_bytes).await?;
        info!("Transcription: {}", transcription.text);

        // Step 2: Intent Classification
        info!("Step 2: Classifying intent...");
        let intent_str = self.nlu_agent.classify_intent(&transcription.text).await?;
        let intent = Intent::from(intent_str.as_str());
        info!("Intent detected: {:?}", intent);

        // Step 3: Entity Extraction
        info!("Step 3: Extracting entities...");
        let entities = self.nlu_agent.extract_entities(&transcription.text).await?;
        info!("Entities: {:?}", entities);

        // Step 4: Process based on intent
        let result = match intent {
            Intent::RecordSale => {
                info!("Processing sale transaction...");
                let transaction = self.transaction_agent.process_transaction(entities.clone(), user_id).await?;
                
                // Step 5: Generate voice response
                let response_text = format!(
                    "Recorded sale of {} {} of {} for ${:.2}",
                    entities.quantity.unwrap_or(1.0),
                    entities.unit.as_deref().unwrap_or("units"),
                    entities.product.as_deref().unwrap_or("unknown product"),
                    entities.price.unwrap_or(0.0)
                );
                
                VoiceTransactionResponse {
                    transaction,
                    transcription: transcription.text.clone(),
                    extracted_entities: entities,
                }
            }
            Intent::QueryAnalytics => {
                // For analytics queries, we don't create a transaction
                VoiceTransactionResponse {
                    transaction: crate::models::transaction::Transaction {
                        id: Uuid::nil(),
                        user_id,
                        product_id: None,
                        quantity: rust_decimal::Decimal::ZERO,
                        unit: "piece".to_string(),
                        price: rust_decimal::Decimal::ZERO,
                        total: rust_decimal::Decimal::ZERO,
                        notes: Some(format!("Analytics query: {}", transcription.text)),
                        voice_recording_url: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    },
                    transcription: transcription.text,
                    extracted_entities: entities,
                }
            }
            _ => {
                // General conversation or unknown intent
                VoiceTransactionResponse {
                    transaction: crate::models::transaction::Transaction {
                        id: Uuid::nil(),
                        user_id,
                        product_id: None,
                        quantity: rust_decimal::Decimal::ZERO,
                        unit: "piece".to_string(),
                        price: rust_decimal::Decimal::ZERO,
                        total: rust_decimal::Decimal::ZERO,
                        notes: Some(format!("General conversation: {}", transcription.text)),
                        voice_recording_url: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    },
                    transcription: transcription.text,
                    extracted_entities: entities,
                }
            }
        };

        // Update context
        let mut context = self.context.write().await;
        context.user_id = user_id;
        context.current_intent = Some(intent_str);
        context.recent_transactions.push(result.transaction.id.to_string());

        info!("Voice transaction processing completed");
        Ok(result)
    }

    pub async fn generate_response(&self, message: &str) -> anyhow::Result<Vec<u8>> {
        self.tts_agent.synthesize(message).await
    }

    pub async fn get_context(&self) -> ConversationContext {
        self.context.read().await.clone()
    }

    /// Transcribe audio to text (direct access to STT)
    pub async fn transcribe_audio(&self, audio_bytes: Vec<u8>) -> anyhow::Result<TranscriptionResult> {
        self.stt_agent.transcribe(audio_bytes).await
    }

    /// Process receipt image using OCR
    /// TODO: Implement real OCR processing
    pub async fn process_receipt_image(&self, _image_data: &[u8]) -> anyhow::Result<ReceiptProcessingResult> {
        // For now, return a placeholder - should use OCR to extract receipt data
        Ok(ReceiptProcessingResult {
            merchant_name: None,
            date: None,
            items: vec![],
            total: None,
            raw_text: "OCR not yet implemented".to_string(),
        })
    }

    /// Scan product image using computer vision
    /// TODO: Implement real product recognition
    pub async fn scan_product_image(&self, _image_data: &[u8]) -> anyhow::Result<ProductScanResult> {
        // For now, return a placeholder - should use CV to identify product
        Ok(ProductScanResult {
            product_name: None,
            confidence: 0.0,
            suggested_category: None,
            raw_text: "Product recognition not yet implemented".to_string(),
        })
    }
}

/// Result of receipt processing
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReceiptProcessingResult {
    pub merchant_name: Option<String>,
    pub date: Option<String>,
    pub items: Vec<ReceiptItem>,
    pub total: Option<f64>,
    pub raw_text: String,
}

/// Individual item from receipt
#[derive(Debug, Clone, serde::Serialize)]
pub struct ReceiptItem {
    pub name: String,
    pub quantity: f64,
    pub price: f64,
}

/// Result of product scanning
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProductScanResult {
    pub product_name: Option<String>,
    pub confidence: f64,
    pub suggested_category: Option<String>,
    pub raw_text: String,
}
