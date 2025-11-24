use reqwest::Client;
use crate::{Config, AppError};
use axum::body::Bytes;

pub struct LlmClient {
    client: Client,
    endpoint: String,
    api_key: String,
}

impl LlmClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            endpoint: config.llm.endpoint.clone(),
            api_key: config.llm.api_key.clone(),
        }
    }

    pub async fn forward_request(&self, body: Bytes) -> Result<reqwest::Response, AppError> {
        self.client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .body(body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error forwarding request to LLM: {}", e);
                AppError::Reqwest(e)
            })
    }
    
    pub async fn send_request(&self, body: String) -> Result<reqwest::Response, AppError> {
        self.client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .body(body)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Error sending request to LLM: {}", e);
                AppError::Reqwest(e)
            })
    }
}
