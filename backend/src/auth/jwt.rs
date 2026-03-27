use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

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
