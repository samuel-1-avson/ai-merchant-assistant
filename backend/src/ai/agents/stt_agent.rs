use std::sync::Arc;

use crate::ai::clients::{CloudSTTClient, TranscriptionResult};

pub struct STTAgent {
    client: Arc<dyn CloudSTTClient>,
}

impl STTAgent {
    pub fn new(client: Arc<dyn CloudSTTClient>) -> Self {
        Self { client }
    }

    pub async fn transcribe(&self, audio_bytes: Vec<u8>) -> anyhow::Result<TranscriptionResult> {
        self.client.transcribe(audio_bytes).await.map_err(|e| anyhow::anyhow!(e))
    }
}
