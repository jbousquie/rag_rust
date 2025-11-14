//! Custom Qdrant client for testing server connectivity.
//!
//! This module provides a simple client that can test if the Qdrant server
//! is running by calling the telemetry endpoint.

use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantClient {
    pub host: String,
    pub port: u16,
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryResponse {
    pub status: String,
    // Add other fields as needed, but for now we'll just focus on status
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionExistsResponse {
    pub result: CollectionExistsResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CollectionExistsResult {
    pub exists: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCollectionResponse {
    pub result: bool,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorParams {
    pub size: u64,
    pub distance: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SparseVectorParams {
    pub index: SparseVectorIndex,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SparseVectorIndex {
    pub on_disk: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCollectionRequest {
    pub vectors: serde_json::Value,
}

impl QdrantClient {
    /// Creates a new Qdrant client
    ///
    /// # Arguments
    /// * `host` - The host address of the Qdrant server
    /// * `port` - The port number of the Qdrant server
    /// * `api_key` - The API key for authentication
    ///
    /// # Returns
    /// * `QdrantClient` - A new instance of the Qdrant client
    pub fn new(host: String, port: u16, api_key: String) -> Self {
        QdrantClient {
            host,
            port,
            api_key,
        }
    }

    /// Tests if the Qdrant server is running by calling the telemetry endpoint
    ///
    /// # Returns
    /// * `Result<TelemetryResponse, reqwest::Error>` - Telemetry information if successful, error otherwise
    pub async fn health_check(&self) -> Result<TelemetryResponse, reqwest::Error> {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/telemetry", self.host, self.port);

        // Send API key as a header parameter
        let response = client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()
            .await?;

        let telemetry: TelemetryResponse = response.json().await?;
        Ok(telemetry)
    }

    /// Blocking version of health_check for synchronous contexts
    ///
    /// # Returns
    /// * `Result<TelemetryResponse, reqwest::Error>` - Telemetry information if successful, error otherwise
    pub fn health_check_blocking(&self) -> Result<TelemetryResponse, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}:{}/telemetry", self.host, self.port);

        // Send API key as a header parameter
        let response = client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()?;

        let telemetry: TelemetryResponse = response.json()?;
        Ok(telemetry)
    }

    /// Checks if a collection exists in Qdrant
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to check
    ///
    /// # Returns
    /// * `Result<bool, reqwest::Error>` - True if collection exists, false otherwise, or error
    pub async fn collection_exists(&self, collection_name: &str) -> Result<bool, reqwest::Error> {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/collections/{}/exists", self.host, self.port, collection_name);

        let response = client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()
            .await?;

        let result: CollectionExistsResponse = response.json().await?;
        Ok(result.result.exists)
    }

    /// Blocking version of collection_exists for synchronous contexts
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to check
    ///
    /// # Returns
    /// * `Result<bool, reqwest::Error>` - True if collection exists, false otherwise, or error
    pub fn collection_exists_blocking(&self, collection_name: &str) -> Result<bool, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}:{}/collections/{}/exists", self.host, self.port, collection_name);

        let response = client
            .get(&url)
            .header("api-key", &self.api_key)
            .send()?;

        let result: CollectionExistsResponse = response.json()?;
        Ok(result.result.exists)
    }

    /// Creates a collection in Qdrant with default dense vector configuration
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to create
    ///
    /// # Returns
    /// * `Result<bool, reqwest::Error>` - True if collection was created successfully, false otherwise, or error
    pub async fn create_collection(&self, collection_name: &str) -> Result<bool, reqwest::Error> {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/collections/{}", self.host, self.port, collection_name);

        let request_body = CreateCollectionRequest {
            vectors: serde_json::json!({
                "size": 384,
                "distance": "Cosine"
            }),
        };

        let response = client
            .put(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let result: CreateCollectionResponse = response.json().await?;
        Ok(result.result)
    }

    /// Blocking version of create_collection for synchronous contexts
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to create
    ///
    /// # Returns
    /// * `Result<bool, reqwest::Error>` - True if collection was created successfully, false otherwise, or error
    pub fn create_collection_blocking(&self, collection_name: &str) -> Result<bool, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let url = format!("http://{}:{}/collections/{}", self.host, self.port, collection_name);

        let request_body = CreateCollectionRequest {
            vectors: serde_json::json!({
                "size": 384,
                "distance": "Cosine"
            }),
        };

        let response = client
            .put(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()?;

        let result: CreateCollectionResponse = response.json()?;
        Ok(result.result)
    }
}