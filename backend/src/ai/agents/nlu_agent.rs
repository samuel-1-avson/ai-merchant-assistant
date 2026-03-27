use std::sync::Arc;

use crate::ai::clients::CloudLLMClient;
use crate::models::transaction::ExtractedEntities;

pub struct NLUAgent {
    client: Arc<dyn CloudLLMClient>,
}

impl NLUAgent {
    pub fn new(client: Arc<dyn CloudLLMClient>) -> Self {
        Self { client }
    }

    pub async fn extract_entities(&self, text: &str) -> anyhow::Result<ExtractedEntities> {
        self.client.extract_entities(text).await.map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn classify_intent(&self, text: &str) -> anyhow::Result<String> {
        let prompt = format!(
            r#"Classify the intent of this text: "{}"

Possible intents: record_sale, query_analytics, update_inventory, set_alert, general_conversation

Respond with only the intent name."#,
            text
        );

        let response = self.client.generate(&prompt).await.map_err(|e| anyhow::anyhow!(e))?;
        Ok(response.trim().to_lowercase())
    }
}
