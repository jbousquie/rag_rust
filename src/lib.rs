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

impl Config {
    /// Loads configuration from the config.toml file
    ///
    /// # Returns
    /// * `Result<Config, Box<dyn std::error::Error>>` - Configuration object if successful, error otherwise
    pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }
}

/// Load configuration from config.toml
pub fn load_config() -> Config {
    Config::load().expect("Failed to load configuration")
}

pub mod indexing;
pub mod qdrant_custom_client;
pub mod rag_proxy;
