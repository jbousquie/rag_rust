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
}