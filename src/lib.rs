//! Main library crate for the RAG proxy application.
//!
//! This crate contains all the shared functionality and configuration
//! used by both the indexing binary and the RAG proxy server.

use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub data_sources: DataSourcesConfig,
    pub indexing: IndexingConfig,
    pub rag_proxy: RagProxyConfig,
    pub llm: LlmConfig,
    pub embeddings: EmbeddingsConfig,
    pub qdrant: QdrantConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSourcesConfig {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexingConfig {
    pub path: String,
    pub file_tracker_path: String,
    pub chunk_size: usize,
    pub embeddings_chunk_size: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RagProxyConfig {
    pub port: u16,
    pub host: String,
    pub chat_completion_endpoint: String,
    pub system_message_fingerprint_length: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    pub endpoint: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingsConfig {
    pub endpoint: String,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub collection: String,
    pub vector_size: usize,
    pub distance: String,
    pub limit: u64,
    pub score_threshold: f32,
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Network error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Qdrant error: {0}")]
    Qdrant(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("PDF extraction error: {0}")]
    Pdf(String),
    #[error("DOCX extraction error: {0}")]
    Docx(String),
    #[error("LLM error: {0}")]
    Llm(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::Io(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Toml(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
            AppError::Reqwest(e) => (axum::http::StatusCode::BAD_GATEWAY, e.to_string()),
            AppError::Json(e) => (axum::http::StatusCode::BAD_REQUEST, e.to_string()),
            AppError::Qdrant(e) => (axum::http::StatusCode::BAD_GATEWAY, e),
            AppError::Config(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::Pdf(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::Docx(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
            AppError::Llm(e) => (axum::http::StatusCode::BAD_GATEWAY, e),
            AppError::Unknown(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e),
        };

        let body = serde_json::json!({
            "error": {
                "message": error_message,
                "type": "AppError"
            }
        });

        (status, axum::Json(body)).into_response()
    }
}

impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::Unknown(err)
    }
}

impl Config {
    /// Loads configuration from the config.toml file
    ///
    /// # Returns
    /// * `Result<Config, AppError>` - Configuration object if successful, error otherwise
    pub fn load() -> Result<Config, AppError> {
        let config_content = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }
}

/// Load configuration from config.toml
pub fn load_config() -> Result<Config, AppError> {
    Config::load()
}

pub mod indexing;
pub mod qdrant_custom_client;
pub mod rag_proxy;
