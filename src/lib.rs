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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RagProxyConfig {
    pub port: u16,
    pub host: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    pub endpoint: String,
    pub model: String,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantConfig {
    pub host: String,
    pub port: u16,
    pub api_key: String,
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

pub mod indexing;
