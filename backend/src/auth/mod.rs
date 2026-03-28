pub mod jwt;

use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // User ID
    pub email: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub aud: Option<String>,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtValidator {
    decoding_key: DecodingKey,
}

impl JwtValidator {
    pub fn new(secret: &str) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }
    
    pub fn validate(&self, token: &str) -> anyhow::Result<Claims> {
        let mut validation = Validation::new(Algorithm::HS256);
        // Allow some clock skew
        validation.leeway = 60;
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}
