use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub email: String,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtGenerator {
    encoding_key: EncodingKey,
}

impl JwtGenerator {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
        }
    }
    
    pub fn generate(&self, user_id: &str, email: &str) -> anyhow::Result<String> {
        let now = Utc::now();
        let exp = now + Duration::hours(24);
        
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };
        
        let token = encode(&Header::default(), &claims, &self.encoding_key)?;
        Ok(token)
    }
}

/// Standalone function to generate JWT token using environment secret
pub fn generate_token(user_id: Uuid, email: &str) -> anyhow::Result<String> {
    let secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default_secret_key_change_in_production".to_string());
    
    let generator = JwtGenerator::new(&secret);
    generator.generate(&user_id.to_string(), email)
}
