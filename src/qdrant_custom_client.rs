//! Custom Qdrant client for testing server connectivity.
//!
//! This module provides a simple client that can test if the Qdrant server
//! is running by calling the telemetry endpoint.
//! https://api.qdrant.tech/api-reference/points/upsert-points

use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantClient {
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub vector_size: u64,
    pub distance: String,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct UpsertPointsRequest {
    pub points: Vec<Point>,
}

// https://api.qdrant.tech/api-reference/points/upsert-points
#[derive(Debug, Serialize, Deserialize)]
pub struct Point {
    pub id: serde_json::Value,
    pub vector: Vec<f32>,
    pub payload: Option<serde_json::Value>,
}

impl Point {
    pub fn new(
        id: serde_json::Value,
        vector: Vec<f32>,
        payload: Option<serde_json::Value>,
    ) -> Self {
        Point {
            id,
            vector,
            payload,
        }
    }

    pub fn from_id_vector(id: &str, vector: Vec<f32>) -> Self {
        Point {
            id: serde_json::Value::String(id.to_string()),
            vector,
            payload: None,
        }
    }

    pub fn from_id_vector_payload(id: &str, vector: Vec<f32>, payload: serde_json::Value) -> Self {
        Point {
            id: serde_json::Value::String(id.to_string()),
            vector,
            payload: Some(payload),
        }
    }
}

impl QdrantClient {
    /// Creates a new Qdrant client
    ///
    /// # Arguments
    /// * `host` - The host address of the Qdrant server
    /// * `port` - The port number of the Qdrant server
    /// * `api_key` - The API key for authentication
    /// * `vector_size` - The size of the vectors to be stored
    /// * `distance` - The distance metric to use for vector similarity
    ///
    /// # Returns
    /// * `QdrantClient` - A new instance of the Qdrant client
    pub fn new(
        host: String,
        port: u16,
        api_key: String,
        vector_size: u64,
        distance: String,
    ) -> Self {
        QdrantClient {
            host,
            port,
            api_key,
            vector_size,
            distance,
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
        let response = client.get(&url).header("api-key", &self.api_key).send()?;

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
        let url = format!(
            "http://{}:{}/collections/{}/exists",
            self.host, self.port, collection_name
        );

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
    pub fn collection_exists_blocking(
        &self,
        collection_name: &str,
    ) -> Result<bool, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "http://{}:{}/collections/{}/exists",
            self.host, self.port, collection_name
        );

        let response = client.get(&url).header("api-key", &self.api_key).send()?;

        let result: CollectionExistsResponse = response.json()?;
        Ok(result.result.exists)
    }

    /// Creates a collection in Qdrant with configurable dense vector configuration
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to create
    ///
    /// # Returns
    /// * `Result<bool, reqwest::Error>` - True if collection was created successfully, false otherwise, or error
    pub async fn create_collection(&self, collection_name: &str) -> Result<bool, reqwest::Error> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/collections/{}",
            self.host, self.port, collection_name
        );

        let request_body = CreateCollectionRequest {
            vectors: serde_json::json!({
                "size": self.vector_size,
                "distance": self.distance
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
    pub fn create_collection_blocking(
        &self,
        collection_name: &str,
    ) -> Result<bool, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "http://{}:{}/collections/{}",
            self.host, self.port, collection_name
        );

        let request_body = CreateCollectionRequest {
            vectors: serde_json::json!({
                "size": self.vector_size,
                "distance": self.distance
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

    /// Upserts points (embeddings) into a Qdrant collection
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to upsert points into
    /// * `filename` - Name of the source file for the chunks
    /// * `chunks` - Vector of chunk values and their corresponding vectors
    ///
    /// # Returns
    /// * `Result<bool, reqwest::Error>` - True if points were upserted successfully, false otherwise, or error
    pub async fn upsert_points(
        &self,
        collection_name: &str,
        filename: String,
        chunks: Vec<(String, Vec<f32>)>,
    ) -> Result<bool, reqwest::Error> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/collections/{}/points",
            self.host, self.port, collection_name
        );

        let points: Vec<Point> = chunks
            .into_iter()
            .map(|(chunk_value, vector)| {
                let payload = serde_json::json!({
                    "text": chunk_value,
                    "source": filename.clone()
                });
                Point::from_id_vector_payload(&uuid::Uuid::new_v4().to_string(), vector, payload)
            })
            .collect();

        let request_body = UpsertPointsRequest { points };

        let response = client
            .put(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;
        Ok(response.status() == 200)
    }

    /// Blocking version of upsert_points for synchronous contexts
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to upsert points into
    /// * `filename` - Name of the source file for the chunks
    /// * `chunks` - Vector of chunk values and their corresponding vectors
    ///
    /// # Returns
    /// * `Result<bool, reqwest::Error>` - True if points were upserted successfully, false otherwise, or error
    pub fn upsert_points_blocking(
        &self,
        collection_name: &str,
        filename: String,
        chunks: Vec<(String, Vec<f32>)>,
    ) -> Result<bool, reqwest::Error> {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "http://{}:{}/collections/{}/points",
            self.host, self.port, collection_name
        );

        let points: Vec<Point> = chunks
            .into_iter()
            .map(|(chunk_value, vector)| {
                let payload = serde_json::json!({
                    "text": chunk_value,
                    "source": filename.clone()
                });
                Point::from_id_vector_payload(&uuid::Uuid::new_v4().to_string(), vector, payload)
            })
            .collect();

        let request_body = UpsertPointsRequest { points };

        let response = client
            .put(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()?;
        Ok(response.status() == 200)
    }
}
