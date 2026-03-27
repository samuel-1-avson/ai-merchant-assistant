use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    pub database_url: String,
    pub redis_url: Option<String>,
    pub supabase_url: String,
    pub supabase_service_key: String,
    pub supabase_jwt_secret: String,
    pub ai_provider: String,
    pub huggingface_api_token: Option<String>,
    pub together_api_key: Option<String>,
    pub runpod_endpoint_url: Option<String>,
    pub runpod_api_key: Option<String>,
    pub easyocr_url: Option<String>,
    pub jwt_secret: String,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();

        let config = Self {
            database_url: std::env::var("DATABASE_URL")?,
            redis_url: std::env::var("REDIS_URL").ok(),
            supabase_url: std::env::var("SUPABASE_URL")?,
            supabase_service_key: std::env::var("SUPABASE_SERVICE_KEY")?,
            supabase_jwt_secret: std::env::var("SUPABASE_JWT_SECRET")?,
            ai_provider: std::env::var("AI_PROVIDER").unwrap_or_else(|_| "huggingface".to_string()),
            huggingface_api_token: std::env::var("HUGGINGFACE_API_TOKEN").ok(),
            together_api_key: std::env::var("TOGETHER_API_KEY").ok(),
            runpod_endpoint_url: std::env::var("RUNPOD_ENDPOINT_URL").ok(),
            runpod_api_key: std::env::var("RUNPOD_API_KEY").ok(),
            jwt_secret: std::env::var("JWT_SECRET")?,
        };

        Ok(config)
    }
}

#[derive(Clone, Debug)]
pub enum AIProvider {
    HuggingFace,
    Together,
    RunPod,
    Replicate,
}

impl From<String> for AIProvider {
    fn from(s: String) -> Self {
        match s.as_str() {
            "together" => AIProvider::Together,
            "runpod" => AIProvider::RunPod,
            "replicate" => AIProvider::Replicate,
            _ => AIProvider::HuggingFace,
        }
    }
}
