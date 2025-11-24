use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::{Config, AppError};

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    embedding: Vec<f32>,
}

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            base_url: config.embeddings.endpoint.clone(),
            model: config.embeddings.model.clone(),
        }
    }

    pub async fn generate_embedding(&self, prompt: &str) -> Result<Vec<f32>, AppError> {
        let url = format!("{}/api/embeddings", self.base_url);
        let request = EmbeddingRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
        };

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("Failed to send embedding request to Ollama: {}", e);
                AppError::Reqwest(e)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            tracing::error!("Ollama API error: {} - {}", status, text);
            return Err(AppError::Unknown(format!("Ollama API error: {} - {}", status, text)));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| {
                tracing::error!("Failed to parse embedding response from Ollama: {}", e);
                AppError::Reqwest(e)
            })?;

        Ok(embedding_response.embedding)
    }
}
