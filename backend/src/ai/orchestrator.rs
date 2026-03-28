use std::sync::Arc;
use uuid::Uuid;
use tracing::{info, warn};
use rust_decimal::prelude::ToPrimitive;

use crate::ai::clients::{CloudSTTClient, CloudLLMClient, CloudTTSClient, CloudVisionClient, TranscriptionResult};
use crate::ai::agents::{stt_agent::STTAgent, nlu_agent::NLUAgent, transaction_agent::TransactionAgent, tts_agent::TTSAgent};
use crate::ai::confirmation::{ConfirmationManager, PendingConfirmation};
use crate::ai::session::SessionStore;
use crate::models::transaction::{VoiceTransactionResponse, ExtractedEntities, Transaction};
use crate::db::repositories::transaction_repo::TransactionRepository;
use crate::db::repositories::product_repo::ProductRepository;
use crate::ocr::service::{OCRService, OCRProcessingResult, ProductScanResult};

// ── Intent ────────────────────────────────────────────────────────────────

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

// ── Voice processing result ───────────────────────────────────────────────

/// Outcome of a voice transaction request.
#[derive(Debug, Clone)]
pub enum VoiceProcessingResult {
    /// Transaction committed immediately (high confidence).
    Immediate(VoiceTransactionResponse),
    /// Transaction awaits user confirmation (low confidence / new product).
    Pending(PendingConfirmation),
}

// ── Orchestrator ──────────────────────────────────────────────────────────

/// Central AI orchestrator.
///
/// Coordinates all agents (STT → NLU → Transaction → TTS) and:
///  - Uses the shared `SessionStore` for cross-request conversation memory.
///  - Delegates receipt OCR to `OCRService` backed by LLaVA-1.5-7B vision model.
pub struct AIOrchestrator {
    stt_agent: STTAgent,
    nlu_agent: NLUAgent,
    transaction_agent: TransactionAgent,
    tts_agent: TTSAgent,
    confirmation_manager: ConfirmationManager,
    ocr_service: OCRService,
    session_store: Arc<SessionStore>,
    transaction_repo: Arc<TransactionRepository>,
}

impl AIOrchestrator {
    pub fn new(
        stt_client: Arc<dyn CloudSTTClient>,
        llm_client: Arc<dyn CloudLLMClient>,
        tts_client: Arc<dyn CloudTTSClient>,
        vision_client: Arc<dyn CloudVisionClient>,
        transaction_repo: Arc<TransactionRepository>,
        product_repo: Arc<ProductRepository>,
        session_store: Arc<SessionStore>,
    ) -> Self {
        let ocr_service = OCRService::new(vision_client, product_repo.clone());
        Self {
            stt_agent: STTAgent::new(stt_client),
            nlu_agent: NLUAgent::new(llm_client),
            transaction_agent: TransactionAgent::new(transaction_repo.clone(), product_repo),
            tts_agent: TTSAgent::new(tts_client),
            confirmation_manager: ConfirmationManager::new(),
            ocr_service,
            session_store,
            transaction_repo,
        }
    }

    // ── Voice transaction pipeline ────────────────────────────────────

    /// Full voice-to-transaction pipeline.
    ///
    /// Models used:
    ///   STT  : openai/whisper-large-v3-turbo  (HuggingFace)
    ///   NLU  : meta-llama/Llama-3.1-8B-Instruct (HuggingFace, temp=0 for JSON)
    ///   TTS  : myshell-ai/MeloTTS-English      (HuggingFace)
    pub async fn process_voice_transaction(
        &self,
        audio_bytes: Vec<u8>,
        user_id: Uuid,
    ) -> anyhow::Result<VoiceProcessingResult> {
        info!("Voice pipeline start for user {}", user_id);

        // Step 1 — Speech-to-Text (Whisper Large V3 Turbo)
        let transcription = self.stt_agent.transcribe(audio_bytes).await?;
        info!("STT: '{}' (conf={:.2})", transcription.text, transcription.confidence);

        // Step 2 — Load session context and enrich the intent prompt
        let session = self.session_store.get_session(user_id).await;
        let preamble = session.build_context_preamble();
        let contextual_text = if preamble.is_empty() {
            transcription.text.clone()
        } else {
            format!("{}{}", preamble, transcription.text)
        };

        // Step 3 — Intent classification (Llama 3.1 8B)
        let intent_str = self.nlu_agent.classify_intent(&contextual_text).await?;
        let intent = Intent::from(intent_str.as_str());
        info!("Intent: {:?}", intent);

        // Step 4 — Entity extraction (Llama 3.1 8B, temp=0)
        let entities = self.nlu_agent.extract_entities(&transcription.text).await?;
        info!("Entities: {:?}", entities);

        // Step 5 — Persist intent + entities to session memory
        self.session_store.update_context(
            user_id,
            &intent_str,
            entities.product.clone(),
            entities.quantity,
            entities.price,
        ).await;

        // Step 6 — Act on intent
        match intent {
            Intent::RecordSale => {
                self.handle_sale_intent(entities, user_id, transcription.text).await
            }
            Intent::QueryAnalytics => {
                Ok(VoiceProcessingResult::Immediate(
                    self.analytics_stub(user_id, &transcription.text, entities),
                ))
            }
            _ => {
                Ok(VoiceProcessingResult::Immediate(
                    self.general_stub(user_id, &transcription.text, entities),
                ))
            }
        }
    }

    // ── Sale intent handlers ──────────────────────────────────────────

    pub async fn handle_sale_intent(
        &self,
        entities: ExtractedEntities,
        user_id: Uuid,
        transcription: String,
    ) -> anyhow::Result<VoiceProcessingResult> {
        if super::agents::nlu_agent::NLUAgent::contains_multiple_products(&transcription) {
            return self.handle_multi_product_intent(user_id, transcription).await;
        }
        self.handle_single_product_intent(entities, user_id, transcription).await
    }

    async fn handle_single_product_intent(
        &self,
        entities: ExtractedEntities,
        user_id: Uuid,
        transcription: String,
    ) -> anyhow::Result<VoiceProcessingResult> {
        // Voice input is an explicit user action — always commit immediately.
        // Product matching is attempted for catalog linking but never gates saving.
        if let Some(ref name) = entities.product {
            match self.transaction_agent.find_similar_products(user_id, name).await {
                Ok(matches) => {
                    let label = matches.first()
                        .map(|m| m.product.name.as_str())
                        .unwrap_or("new product");
                    info!("Product match: {} — committing immediately", label);
                }
                Err(e) => warn!("Product lookup failed ({}), continuing", e),
            }
        }

        let tx_result = self.transaction_agent
            .process_transaction(entities.clone(), user_id)
            .await?;

        let summary = format!(
            "Sold {} {} of {} for ${:.2}",
            entities.quantity.unwrap_or(1.0),
            entities.unit.as_deref().unwrap_or("units"),
            entities.product.as_deref().unwrap_or("item"),
            entities.price.unwrap_or(0.0) * entities.quantity.unwrap_or(1.0)
        );
        self.session_store.record_transaction(user_id, summary).await;

        Ok(VoiceProcessingResult::Immediate(VoiceTransactionResponse {
            transaction: tx_result.transaction,
            transcription,
            extracted_entities: entities,
        }))
    }

    async fn handle_multi_product_intent(
        &self,
        user_id: Uuid,
        transcription: String,
    ) -> anyhow::Result<VoiceProcessingResult> {
        let multi_entities = self.nlu_agent
            .extract_multi_product_entities(&transcription)
            .await?;

        if multi_entities.items.is_empty() {
            warn!("Multi-product extraction empty, falling back to single");
            let entities = self.nlu_agent.extract_entities(&transcription).await?;
            return self.handle_single_product_intent(entities, user_id, transcription).await;
        }

        let result = self.transaction_agent
            .process_multi_product_transaction(multi_entities.clone(), user_id)
            .await?;

        for item in &result.items {
            if let Some(ref tx) = item.transaction {
                let summary = format!(
                    "Multi-sale: {} {} for ${:.2}",
                    item.extracted_item.quantity,
                    item.extracted_item.product,
                    tx.total.to_f64().unwrap_or(0.0)
                );
                self.session_store.record_transaction(user_id, summary).await;
            }
        }

        let summary_tx = Transaction {
            id: Uuid::new_v4(),
            user_id,
            product_id: None,
            product_name: Some(format!("{} items", result.items.len())),
            quantity: rust_decimal::Decimal::from(result.items.len() as i32),
            unit: "items".to_string(),
            price: result.total_amount,
            total: result.total_amount,
            notes: Some(format!("Multi-product sale: {} items", result.items.len())),
            voice_recording_url: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Ok(VoiceProcessingResult::Immediate(VoiceTransactionResponse {
            transaction: summary_tx,
            transcription,
            extracted_entities: ExtractedEntities {
                product: Some(format!("{} items", result.items.len())),
                quantity: Some(result.items.len() as f64),
                unit: Some("items".to_string()),
                price: result.total_amount.to_f64(),
                currency: multi_entities.currency,
            },
        }))
    }

    // ── Confirmation actions ──────────────────────────────────────────

    pub async fn confirm_transaction(
        &self,
        confirmation_id: &Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<Transaction> {
        let confirmation = self.confirmation_manager.confirm(confirmation_id, user_id).await
            .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))?;

        let tx_result = self.transaction_agent
            .process_transaction(confirmation.extracted_entities.clone(), user_id)
            .await?;

        let e = &confirmation.extracted_entities;
        let summary = format!(
            "Confirmed: {} {} of {} for ${:.2}",
            e.quantity.unwrap_or(1.0),
            e.unit.as_deref().unwrap_or("units"),
            e.product.as_deref().unwrap_or("item"),
            e.price.unwrap_or(0.0) * e.quantity.unwrap_or(1.0)
        );
        self.session_store.record_transaction(user_id, summary).await;
        self.confirmation_manager.remove(confirmation_id).await;

        Ok(tx_result.transaction)
    }

    pub async fn reject_transaction(
        &self,
        confirmation_id: &Uuid,
        user_id: Uuid,
    ) -> anyhow::Result<()> {
        self.confirmation_manager.reject(confirmation_id, user_id).await
            .map_err(|e| anyhow::anyhow!("Rejection failed: {}", e))?;
        self.confirmation_manager.remove(confirmation_id).await;
        Ok(())
    }

    pub async fn get_pending_confirmations(&self, user_id: Uuid) -> Vec<PendingConfirmation> {
        self.confirmation_manager.get_user_confirmations(user_id).await
    }

    pub async fn get_confirmation(&self, id: &Uuid) -> Option<PendingConfirmation> {
        self.confirmation_manager.get_confirmation(id).await
    }

    // ── STT / TTS passthrough ─────────────────────────────────────────

    pub async fn transcribe_audio(&self, audio_bytes: Vec<u8>) -> anyhow::Result<TranscriptionResult> {
        self.stt_agent.transcribe(audio_bytes).await
    }

    /// Synthesize a message to audio bytes using MeloTTS (HuggingFace).
    pub async fn generate_response(&self, message: &str) -> anyhow::Result<Vec<u8>> {
        self.tts_agent.synthesize(message).await
    }

    // ── OCR / Vision ──────────────────────────────────────────────────

    /// Process a receipt image using LLaVA-1.5-7B (vision model).
    ///
    /// Returns structured receipt data including matched product links.
    pub async fn process_receipt_image(
        &self,
        user_id: Uuid,
        image_data: Vec<u8>,
    ) -> anyhow::Result<OCRProcessingResult> {
        info!("Delegating receipt OCR to OCRService (user={})", user_id);
        self.ocr_service.process_receipt(user_id, image_data).await
    }

    /// Scan a product image to identify the product using LLaVA-1.5-7B.
    pub async fn scan_product_image(&self, image_data: &[u8]) -> anyhow::Result<ProductScanResult> {
        info!("Scanning product image with vision model");
        self.ocr_service.scan_product_image(image_data).await
    }

    // ── Assistant chat ────────────────────────────────────────────────

    /// Generate a conversational reply from the LLM given a full prompt.
    ///
    /// The caller (assistant route) is responsible for injecting the merchant
    /// data context into the prompt.  Returns `Ok(None)` when the LLM is
    /// unavailable so the caller can produce a data-driven fallback response.
    pub async fn generate_chat_response(&self, prompt: &str) -> anyhow::Result<Option<String>> {
        match self.nlu_agent.generate_text(prompt).await {
            Ok(text) => Ok(Some(text)),
            Err(e) => {
                warn!("LLM chat unavailable ({}), caller will use fallback", e);
                Ok(None)
            }
        }
    }

    // ── Intent stubs ──────────────────────────────────────────────────

    fn analytics_stub(&self, user_id: Uuid, text: &str, entities: ExtractedEntities) -> VoiceTransactionResponse {
        VoiceTransactionResponse {
            transaction: Transaction {
                id: Uuid::nil(),
                user_id,
                product_id: None,
                product_name: None,
                quantity: rust_decimal::Decimal::ZERO,
                unit: "query".to_string(),
                price: rust_decimal::Decimal::ZERO,
                total: rust_decimal::Decimal::ZERO,
                notes: Some(format!("Analytics query: {}", text)),
                voice_recording_url: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            transcription: text.to_string(),
            extracted_entities: entities,
        }
    }

    fn general_stub(&self, user_id: Uuid, text: &str, entities: ExtractedEntities) -> VoiceTransactionResponse {
        VoiceTransactionResponse {
            transaction: Transaction {
                id: Uuid::nil(),
                user_id,
                product_id: None,
                product_name: None,
                quantity: rust_decimal::Decimal::ZERO,
                unit: "general".to_string(),
                price: rust_decimal::Decimal::ZERO,
                total: rust_decimal::Decimal::ZERO,
                notes: Some(format!("General: {}", text)),
                voice_recording_url: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            transcription: text.to_string(),
            extracted_entities: entities,
        }
    }
}
