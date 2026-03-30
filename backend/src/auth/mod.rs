pub mod jwt;

use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, jwk::Jwk};
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
    algorithm: Algorithm,
}

impl JwtValidator {
    /// Create from a Supabase JWKS key (ES256).
    /// This is the correct path for Supabase projects that use asymmetric signing.
    pub fn from_jwk(jwk: &Jwk) -> anyhow::Result<Self> {
        let decoding_key = DecodingKey::from_jwk(jwk)?;
        Ok(Self {
            decoding_key,
            algorithm: Algorithm::ES256,
        })
    }

    /// Fallback: symmetric HS256 secret.
    pub fn from_secret(secret: &str) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            algorithm: Algorithm::HS256,
        }
    }

    pub fn validate(&self, token: &str) -> anyhow::Result<Claims> {
        let mut validation = Validation::new(self.algorithm);
        validation.leeway = 60;
        // Supabase JWTs carry aud: "authenticated" — skip audience check.
        validation.validate_aud = false;
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;
        Ok(token_data.claims)
    }
}
