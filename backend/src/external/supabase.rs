use reqwest::Client;

pub struct SupabaseClient {
    client: Client,
    url: String,
    service_key: String,
}

impl SupabaseClient {
    pub fn new(url: String, service_key: String) -> Self {
        Self {
            client: Client::new(),
            url,
            service_key,
        }
    }

    pub async fn create_user(&self, email: &str, password: &str) -> anyhow::Result<serde_json::Value> {
        let response = self.client
            .post(format!("{}/auth/v1/admin/users", self.url))
            .header("apikey", &self.service_key)
            .header("Authorization", format!("Bearer {}", self.service_key))
            .json(&serde_json::json!({
                "email": email,
                "password": password,
                "email_confirm": true
            }))
            .send()
            .await?;

        let user: serde_json::Value = response.json().await?;
        Ok(user)
    }

    pub async fn sign_in(&self, email: &str, password: &str) -> anyhow::Result<serde_json::Value> {
        let response = self.client
            .post(format!("{}/auth/v1/token?grant_type=password", self.url))
            .header("apikey", &self.service_key)
            .json(&serde_json::json!({
                "email": email,
                "password": password
            }))
            .send()
            .await?;

        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }
}
