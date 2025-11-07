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
    pub fn load() -> Result<Config, Box<dyn std::error::Error>> {
        let config_content = fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }
}

pub mod common;
