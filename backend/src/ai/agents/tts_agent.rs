use std::sync::Arc;

use crate::ai::clients::CloudTTSClient;

pub struct TTSAgent {
    client: Arc<dyn CloudTTSClient>,
}

impl TTSAgent {
    pub fn new(client: Arc<dyn CloudTTSClient>) -> Self {
        Self { client }
    }

    pub async fn synthesize(&self, text: &str) -> anyhow::Result<Vec<u8>> {
        self.client.synthesize(text).await.map_err(|e| anyhow::anyhow!(e))
    }
}
