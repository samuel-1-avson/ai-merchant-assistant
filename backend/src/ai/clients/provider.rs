//! AI Provider Management with Health Checks and Fallback
//!
//! This module provides a robust AI client system with:
//! - Multiple provider support (HuggingFace, Together, RunPod)
//! - Health checking and automatic failover
//! - Circuit breaker pattern for failing providers
//! - Metrics and logging

use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

use super::{
    huggingface::{HuggingFaceClient, HuggingFaceVisionClient},
    runpod::RunPodClient,
    together::TogetherClient,
    groq::GroqClient,
    CloudLLMClient, CloudSTTClient, CloudTTSClient, CloudVisionClient,
    TranscriptionResult, VisionResult, AIError,
};

/// Provider health status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProviderHealth {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Provider configuration
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub name: String,
    pub priority: u8, // Lower = higher priority (0 = primary)
    pub enabled: bool,
}

/// Health check result
#[derive(Debug)]
struct HealthCheck {
    last_check: Instant,
    consecutive_failures: u32,
    health: ProviderHealth,
}

impl Default for HealthCheck {
    fn default() -> Self {
        Self {
            last_check: Instant::now(),
            consecutive_failures: 0,
            health: ProviderHealth::Healthy,
        }
    }
}

/// Multi-provider STT client with failover
pub struct FailoverSTTClient {
    providers: Vec<(Box<dyn CloudSTTClient>, ProviderConfig, RwLock<HealthCheck>)>,
    current_index: AtomicUsize,
    circuit_threshold: u32,
}

impl FailoverSTTClient {
    pub fn new(providers: Vec<(Box<dyn CloudSTTClient>, ProviderConfig)>) -> Self {
        let providers = providers
            .into_iter()
            .map(|(client, config)| (client, config, RwLock::new(HealthCheck::default())))
            .collect();

        Self {
            providers,
            current_index: AtomicUsize::new(0),
            circuit_threshold: 3,
        }
    }

    /// Get the next healthy provider index
    async fn get_healthy_provider_index(&self) -> Option<usize> {
        for (idx, (_, config, health)) in self.providers.iter().enumerate() {
            if !config.enabled {
                continue;
            }

            let health_check = health.read().await;
            if health_check.consecutive_failures < self.circuit_threshold {
                return Some(idx);
            }
        }
        None
    }

    /// Mark a provider as failed
    async fn mark_failure(&self, index: usize) {
        if let Some((_, _, health)) = self.providers.get(index) {
            let mut health_check = health.write().await;
            health_check.consecutive_failures += 1;
            health_check.last_check = Instant::now();

            if health_check.consecutive_failures >= self.circuit_threshold {
                health_check.health = ProviderHealth::Unhealthy;
                warn!(
                    "Provider at index {} marked as unhealthy after {} failures",
                    index, health_check.consecutive_failures
                );
            }
        }
    }

    /// Mark a provider as successful
    async fn mark_success(&self, index: usize) {
        if let Some((_, _, health)) = self.providers.get(index) {
            let mut health_check = health.write().await;
            if health_check.consecutive_failures > 0 {
                health_check.consecutive_failures = 0;
                health_check.health = ProviderHealth::Healthy;
                info!("Provider at index {} recovered", index);
            }
        }
    }
}

#[async_trait]
impl CloudSTTClient for FailoverSTTClient {
    async fn transcribe(&self, audio_bytes: Vec<u8>) -> Result<TranscriptionResult, AIError> {
        let start_idx = self.current_index.load(Ordering::Relaxed);
        
        for offset in 0..self.providers.len() {
            let idx = (start_idx + offset) % self.providers.len();
            
            let (_, config, _) = self.providers.get(idx).unwrap();
            if !config.enabled {
                continue;
            }

            info!("Attempting STT with provider at index {} ({})", idx, config.name);

            // Try to transcribe
            let result = self.providers[idx].0.transcribe(audio_bytes.clone()).await;

            match &result {
                Ok(transcription) => {
                    info!("STT successful with provider {}: '{}'", config.name, transcription.text);
                    self.mark_success(idx).await;
                    self.current_index.store(idx, Ordering::Relaxed);
                    return result;
                }
                Err(AIError::RateLimited) => {
                    warn!("Provider {} rate limited, trying next...", config.name);
                    self.mark_failure(idx).await;
                }
                Err(AIError::ServiceUnavailable) => {
                    warn!("Provider {} unavailable, trying next...", config.name);
                    self.mark_failure(idx).await;
                }
                Err(e) => {
                    warn!("Provider {} error: {}, trying next...", config.name, e);
                    self.mark_failure(idx).await;
                }
            }
        }

        error!("All STT providers failed");
        Err(AIError::ServiceUnavailable)
    }
}

/// Multi-provider LLM client with failover
pub struct FailoverLLMClient {
    providers: Vec<(Box<dyn CloudLLMClient>, ProviderConfig, RwLock<HealthCheck>)>,
    current_index: AtomicUsize,
    circuit_threshold: u32,
}

impl FailoverLLMClient {
    pub fn new(providers: Vec<(Box<dyn CloudLLMClient>, ProviderConfig)>) -> Self {
        let providers = providers
            .into_iter()
            .map(|(client, config)| (client, config, RwLock::new(HealthCheck::default())))
            .collect();

        Self {
            providers,
            current_index: AtomicUsize::new(0),
            circuit_threshold: 3,
        }
    }

    async fn mark_failure(&self, index: usize) {
        if let Some((_, _, health)) = self.providers.get(index) {
            let mut health_check = health.write().await;
            health_check.consecutive_failures += 1;
            health_check.last_check = Instant::now();

            if health_check.consecutive_failures >= self.circuit_threshold {
                health_check.health = ProviderHealth::Unhealthy;
                warn!(
                    "LLM Provider at index {} marked as unhealthy after {} failures",
                    index, health_check.consecutive_failures
                );
            }
        }
    }

    async fn mark_success(&self, index: usize) {
        if let Some((_, _, health)) = self.providers.get(index) {
            let mut health_check = health.write().await;
            if health_check.consecutive_failures > 0 {
                health_check.consecutive_failures = 0;
                health_check.health = ProviderHealth::Healthy;
                info!("LLM Provider at index {} recovered", index);
            }
        }
    }
}

#[async_trait]
impl CloudLLMClient for FailoverLLMClient {
    async fn generate(&self, prompt: &str) -> Result<String, AIError> {
        let start_idx = self.current_index.load(Ordering::Relaxed);
        
        for offset in 0..self.providers.len() {
            let idx = (start_idx + offset) % self.providers.len();
            
            let (_, config, health) = self.providers.get(idx).unwrap();
            if !config.enabled {
                continue;
            }

            // Skip unhealthy providers
            let health_check = health.read().await;
            if health_check.consecutive_failures >= self.circuit_threshold {
                continue;
            }
            drop(health_check);

            info!("Attempting LLM generation with provider {}", config.name);
            let start = Instant::now();

            let result = self.providers[idx].0.generate(prompt).await;

            match &result {
                Ok(response) => {
                    let elapsed = start.elapsed();
                    info!(
                        "LLM generation successful with provider {} in {:?}",
                        config.name, elapsed
                    );
                    self.mark_success(idx).await;
                    self.current_index.store(idx, Ordering::Relaxed);
                    return result;
                }
                Err(AIError::RateLimited) => {
                    warn!("LLM Provider {} rate limited", config.name);
                    self.mark_failure(idx).await;
                }
                Err(AIError::ServiceUnavailable) => {
                    warn!("LLM Provider {} unavailable", config.name);
                    self.mark_failure(idx).await;
                }
                Err(e) => {
                    warn!("LLM Provider {} error: {}", config.name, e);
                    self.mark_failure(idx).await;
                }
            }
        }

        error!("All LLM providers failed");
        Err(AIError::ServiceUnavailable)
    }

    async fn extract_entities(&self, text: &str) -> Result<crate::models::transaction::ExtractedEntities, AIError> {
        let start_idx = self.current_index.load(Ordering::Relaxed);
        
        for offset in 0..self.providers.len() {
            let idx = (start_idx + offset) % self.providers.len();
            
            let (_, config, health) = self.providers.get(idx).unwrap();
            if !config.enabled {
                continue;
            }

            let health_check = health.read().await;
            if health_check.consecutive_failures >= self.circuit_threshold {
                continue;
            }
            drop(health_check);

            info!("Attempting entity extraction with provider {}", config.name);
            let result = self.providers[idx].0.extract_entities(text).await;

            match &result {
                Ok(entities) => {
                    info!(
                        "Entity extraction successful with provider {}: {:?}",
                        config.name, entities
                    );
                    self.mark_success(idx).await;
                    self.current_index.store(idx, Ordering::Relaxed);
                    return result;
                }
                Err(e) => {
                    warn!("Entity extraction failed with provider {}: {}", config.name, e);
                    self.mark_failure(idx).await;
                }
            }
        }

        error!("All LLM providers failed for entity extraction");
        Err(AIError::ServiceUnavailable)
    }
}

/// Multi-provider TTS client with failover
pub struct FailoverTTSClient {
    providers: Vec<(Box<dyn CloudTTSClient>, ProviderConfig, RwLock<HealthCheck>)>,
    current_index: AtomicUsize,
    circuit_threshold: u32,
}

impl FailoverTTSClient {
    pub fn new(providers: Vec<(Box<dyn CloudTTSClient>, ProviderConfig)>) -> Self {
        let providers = providers
            .into_iter()
            .map(|(client, config)| (client, config, RwLock::new(HealthCheck::default())))
            .collect();

        Self {
            providers,
            current_index: AtomicUsize::new(0),
            circuit_threshold: 3,
        }
    }

    async fn mark_failure(&self, index: usize) {
        if let Some((_, _, health)) = self.providers.get(index) {
            let mut health_check = health.write().await;
            health_check.consecutive_failures += 1;
            health_check.last_check = Instant::now();

            if health_check.consecutive_failures >= self.circuit_threshold {
                health_check.health = ProviderHealth::Unhealthy;
            }
        }
    }

    async fn mark_success(&self, index: usize) {
        if let Some((_, _, health)) = self.providers.get(index) {
            let mut health_check = health.write().await;
            health_check.consecutive_failures = 0;
            health_check.health = ProviderHealth::Healthy;
        }
    }
}

#[async_trait]
impl CloudTTSClient for FailoverTTSClient {
    async fn synthesize(&self, text: &str) -> Result<Vec<u8>, AIError> {
        let start_idx = self.current_index.load(Ordering::Relaxed);
        
        for offset in 0..self.providers.len() {
            let idx = (start_idx + offset) % self.providers.len();
            
            let (_, config, health) = self.providers.get(idx).unwrap();
            if !config.enabled {
                continue;
            }

            let health_check = health.read().await;
            if health_check.consecutive_failures >= self.circuit_threshold {
                continue;
            }
            drop(health_check);

            let result = self.providers[idx].0.synthesize(text).await;

            match &result {
                Ok(audio) => {
                    info!("TTS successful with provider {}", config.name);
                    self.mark_success(idx).await;
                    self.current_index.store(idx, Ordering::Relaxed);
                    return result;
                }
                Err(e) => {
                    warn!("TTS failed with provider {}: {}", config.name, e);
                    self.mark_failure(idx).await;
                }
            }
        }

        error!("All TTS providers failed");
        Err(AIError::ServiceUnavailable)
    }
}

/// Multi-provider Vision client with failover
pub struct FailoverVisionClient {
    providers: Vec<(Box<dyn CloudVisionClient>, ProviderConfig, RwLock<HealthCheck>)>,
    current_index: AtomicUsize,
    circuit_threshold: u32,
}

impl FailoverVisionClient {
    pub fn new(providers: Vec<(Box<dyn CloudVisionClient>, ProviderConfig)>) -> Self {
        let providers = providers
            .into_iter()
            .map(|(client, config)| (client, config, RwLock::new(HealthCheck::default())))
            .collect();
        Self {
            providers,
            current_index: AtomicUsize::new(0),
            circuit_threshold: 3,
        }
    }

    async fn mark_failure(&self, index: usize) {
        if let Some((_, _, health)) = self.providers.get(index) {
            let mut h = health.write().await;
            h.consecutive_failures += 1;
            h.last_check = Instant::now();
            if h.consecutive_failures >= self.circuit_threshold {
                h.health = ProviderHealth::Unhealthy;
                warn!("Vision provider {} marked unhealthy", index);
            }
        }
    }

    async fn mark_success(&self, index: usize) {
        if let Some((_, _, health)) = self.providers.get(index) {
            let mut h = health.write().await;
            h.consecutive_failures = 0;
            h.health = ProviderHealth::Healthy;
        }
    }

    async fn try_providers<F, Fut>(&self, op: F) -> Result<VisionResult, AIError>
    where
        F: Fn(&dyn CloudVisionClient) -> Fut,
        Fut: std::future::Future<Output = Result<VisionResult, AIError>>,
    {
        let start_idx = self.current_index.load(Ordering::Relaxed);
        for offset in 0..self.providers.len() {
            let idx = (start_idx + offset) % self.providers.len();
            let (client, config, health) = &self.providers[idx];
            if !config.enabled {
                continue;
            }
            {
                let h = health.read().await;
                if h.consecutive_failures >= self.circuit_threshold {
                    continue;
                }
            }
            info!("Attempting vision task with provider {}", config.name);
            let result = op(client.as_ref()).await;
            match &result {
                Ok(_) => {
                    self.mark_success(idx).await;
                    self.current_index.store(idx, Ordering::Relaxed);
                    return result;
                }
                Err(AIError::RateLimited) | Err(AIError::ServiceUnavailable) => {
                    warn!("Vision provider {} failed, trying next", config.name);
                    self.mark_failure(idx).await;
                }
                Err(e) => {
                    warn!("Vision provider {} error: {}", config.name, e);
                    self.mark_failure(idx).await;
                }
            }
        }
        error!("All vision providers failed");
        Err(AIError::ServiceUnavailable)
    }
}

#[async_trait]
impl CloudVisionClient for FailoverVisionClient {
    async fn extract_text_from_image(&self, image_bytes: &[u8]) -> Result<VisionResult, AIError> {
        let start_idx = self.current_index.load(Ordering::Relaxed);
        for offset in 0..self.providers.len() {
            let idx = (start_idx + offset) % self.providers.len();
            let (client, config, health) = &self.providers[idx];
            if !config.enabled { continue; }
            {
                let h = health.read().await;
                if h.consecutive_failures >= self.circuit_threshold { continue; }
            }
            info!("Attempting vision extract_text with provider {}", config.name);
            let result = client.extract_text_from_image(image_bytes).await;
            match &result {
                Ok(_) => {
                    self.mark_success(idx).await;
                    self.current_index.store(idx, Ordering::Relaxed);
                    return result;
                }
                Err(AIError::RateLimited) | Err(AIError::ServiceUnavailable) => {
                    warn!("Vision provider {} failed, trying next", config.name);
                    self.mark_failure(idx).await;
                }
                Err(e) => {
                    warn!("Vision provider {} error: {}", config.name, e);
                    self.mark_failure(idx).await;
                }
            }
        }
        error!("All vision providers failed for extract_text_from_image");
        Err(AIError::ServiceUnavailable)
    }

    async fn analyze_receipt(&self, image_bytes: &[u8]) -> Result<VisionResult, AIError> {
        let start_idx = self.current_index.load(Ordering::Relaxed);
        for offset in 0..self.providers.len() {
            let idx = (start_idx + offset) % self.providers.len();
            let (client, config, health) = &self.providers[idx];
            if !config.enabled { continue; }
            {
                let h = health.read().await;
                if h.consecutive_failures >= self.circuit_threshold { continue; }
            }
            info!("Attempting vision analyze_receipt with provider {}", config.name);
            let result = client.analyze_receipt(image_bytes).await;
            match &result {
                Ok(_) => {
                    self.mark_success(idx).await;
                    self.current_index.store(idx, Ordering::Relaxed);
                    return result;
                }
                Err(AIError::RateLimited) | Err(AIError::ServiceUnavailable) => {
                    warn!("Vision provider {} failed, trying next", config.name);
                    self.mark_failure(idx).await;
                }
                Err(e) => {
                    warn!("Vision provider {} error: {}", config.name, e);
                    self.mark_failure(idx).await;
                }
            }
        }
        error!("All vision providers failed for analyze_receipt");
        Err(AIError::ServiceUnavailable)
    }
}

/// Builder for creating AI clients with failover
pub struct AIClientBuilder {
    huggingface_token: Option<String>,
    groq_key: Option<String>,
    together_key: Option<String>,
    runpod_endpoint: Option<String>,
    runpod_key: Option<String>,
}

impl AIClientBuilder {
    pub fn new() -> Self {
        Self {
            huggingface_token: None,
            groq_key: None,
            together_key: None,
            runpod_endpoint: None,
            runpod_key: None,
        }
    }

    pub fn with_huggingface(mut self, token: Option<String>) -> Self {
        self.huggingface_token = token;
        self
    }

    pub fn with_groq(mut self, key: Option<String>) -> Self {
        self.groq_key = key;
        self
    }

    pub fn with_together(mut self, key: Option<String>) -> Self {
        self.together_key = key;
        self
    }

    pub fn with_runpod(mut self, endpoint: Option<String>, key: Option<String>) -> Self {
        self.runpod_endpoint = endpoint;
        self.runpod_key = key;
        self
    }

    pub fn build_stt_client(self) -> Arc<dyn CloudSTTClient> {
        let mut providers: Vec<(Box<dyn CloudSTTClient>, ProviderConfig)> = Vec::new();

        // HuggingFace (Primary)
        if let Some(token) = self.huggingface_token.clone() {
            providers.push((
                Box::new(HuggingFaceClient::new(token)),
                ProviderConfig {
                    name: "huggingface".to_string(),
                    priority: 0,
                    enabled: true,
                },
            ));
        }

        // RunPod (Fallback for STT)
        if let (Some(endpoint), Some(key)) = (self.runpod_endpoint.clone(), self.runpod_key.clone()) {
            providers.push((
                Box::new(RunPodClient::new(endpoint, key)),
                ProviderConfig {
                    name: "runpod".to_string(),
                    priority: 1,
                    enabled: true,
                },
            ));
        }

        if providers.is_empty() {
            panic!("No STT providers configured!");
        }

        Arc::new(FailoverSTTClient::new(providers))
    }

    pub fn build_llm_client(self) -> Arc<dyn CloudLLMClient> {
        let mut providers: Vec<(Box<dyn CloudLLMClient>, ProviderConfig)> = Vec::new();

        // Groq (Primary — free tier, ~400ms, Llama 3.1 8B Instant. Registered first so it
        // is tried before HuggingFace whose text-gen models return 404 on the free router.)
        if let Some(key) = self.groq_key.clone() {
            providers.push((
                Box::new(GroqClient::new(key)),
                ProviderConfig {
                    name: "groq".to_string(),
                    priority: 0,
                    enabled: true,
                },
            ));
        }

        // HuggingFace (Fallback 1 — text gen models currently 404 on free inference router;
        // kept here so it activates automatically if HF access improves)
        if let Some(token) = self.huggingface_token.clone() {
            providers.push((
                Box::new(HuggingFaceClient::new(token)),
                ProviderConfig {
                    name: "huggingface".to_string(),
                    priority: 1,
                    enabled: true,
                },
            ));
        }

        // Together AI (Fallback 2 — requires paid account deposit)
        if let Some(key) = self.together_key.clone() {
            providers.push((
                Box::new(TogetherClient::new(key)),
                ProviderConfig {
                    name: "together".to_string(),
                    priority: 2,
                    enabled: true,
                },
            ));
        }

        // RunPod (Last resort — self-hosted endpoint)
        if let (Some(endpoint), Some(key)) = (self.runpod_endpoint.clone(), self.runpod_key.clone()) {
            providers.push((
                Box::new(RunPodClient::new(endpoint, key)),
                ProviderConfig {
                    name: "runpod".to_string(),
                    priority: 3,
                    enabled: true,
                },
            ));
        }

        if providers.is_empty() {
            panic!("No LLM providers configured!");
        }

        Arc::new(FailoverLLMClient::new(providers))
    }

    pub fn build_tts_client(self) -> Arc<dyn CloudTTSClient> {
        let mut providers: Vec<(Box<dyn CloudTTSClient>, ProviderConfig)> = Vec::new();

        if let Some(token) = self.huggingface_token {
            providers.push((
                Box::new(HuggingFaceClient::new(token)),
                ProviderConfig {
                    name: "huggingface".to_string(),
                    priority: 0,
                    enabled: true,
                },
            ));
        }

        if providers.is_empty() {
            panic!("No TTS providers configured! HuggingFace token required.");
        }

        Arc::new(FailoverTTSClient::new(providers))
    }

    /// Build the vision client.
    ///
    /// Primary model: **llava-hf/llava-1.5-7b-hf** (HuggingFace free inference).
    /// This enables genuine image understanding for receipt OCR and product scanning.
    pub fn build_vision_client(self) -> Arc<dyn CloudVisionClient> {
        let mut providers: Vec<(Box<dyn CloudVisionClient>, ProviderConfig)> = Vec::new();

        if let Some(token) = self.huggingface_token {
            providers.push((
                Box::new(HuggingFaceVisionClient::new(token)),
                ProviderConfig {
                    name: "huggingface-llava".to_string(),
                    priority: 0,
                    enabled: true,
                },
            ));
        }

        if providers.is_empty() {
            panic!("No vision providers configured! HuggingFace token required.");
        }

        Arc::new(FailoverVisionClient::new(providers))
    }
}

impl Default for AIClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

use std::sync::Arc;
