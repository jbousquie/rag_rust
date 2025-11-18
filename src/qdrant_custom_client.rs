//! Custom Qdrant client for testing server connectivity.
//!
//! This module provides a simple client that can test if the Qdrant server
//! is running by calling the telemetry endpoint.
//! https://api.qdrant.tech/api-reference/points/upsert-points

use reqwest;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QdrantClient {
    pub host: String,
    pub port: u16,
    pub api_key: String,
    pub vector_size: u64,
    pub distance: String,
    pub limit: u64,
    pub score_threshold: f32,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchPointsRequest {
    pub vector: Vec<f32>,
    pub limit: u64,
    pub score_threshold: f32,
    pub filter: Option<serde_json::Value>,
    pub with_payload: Option<bool>,
    pub with_vector: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchPointsResponse {
    pub result: SearchPointsResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchPointsResult {
    pub points: Vec<ScoredPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchPointsPayload {
    pub source: String,
    pub text: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ScoredPoint {
    pub id: serde_json::Value,
    pub vector: Option<Vec<f32>>,
    pub payload: Option<SearchPointsPayload>,
    pub score: f32,
    pub version: u64,
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
    /// * `limit` - The max number of result to return
    /// * `score_threshold` - Only the points with score better than the threshold are returned
    ///
    /// # Returns
    /// * `QdrantClient` - A new instance of the Qdrant client
    pub fn new(
        host: String,
        port: u16,
        api_key: String,
        vector_size: u64,
        distance: String,
        limit: u64,
        score_threshold: f32,
    ) -> Self {
        QdrantClient {
            host,
            port,
            api_key,
            vector_size,
            distance,
            limit,
            score_threshold,
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
                // Calculate hash of chunk_value using a proper hashing method
                // This hash is used as the ID for the point to prevent duplicates
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                chunk_value.hash(&mut hasher);
                let hash_value = format!("{:0>32x}", hasher.finish());

                // Convert hash to a UUID format to comply with Qdrant requirements
                // Format: 8-4-4-4-12 hex digits (like 550e8400-e29b-41d4-a716-446655440000)
                let uuid_string = format!(
                    "{}-{}-{}-{}-{}",
                    &hash_value[..8],
                    &hash_value[8..12],
                    &hash_value[12..16],
                    &hash_value[16..20],
                    &hash_value[20..32]
                );

                let payload = serde_json::json!({
                    "text": chunk_value,
                    "source": filename.clone()
                });
                Point::from_id_vector_payload(&uuid_string, vector, payload)
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

        // Log the response status for debugging
        let status = response.status();
        println!("Qdrant upsert response status: {}", status);
        if !status.is_success() {
            // Try to get the error details from the response body
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            println!("Qdrant error details: {}", error_text);
        }
        Ok(status == 200)
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
                // Calculate hash of chunk_value using a proper hashing method
                // This hash is used as the ID for the point to prevent duplicates
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                chunk_value.hash(&mut hasher);
                let hash_value = format!("{:0>32x}", hasher.finish());

                // Convert hash to a UUID format to comply with Qdrant requirements
                // Format: 8-4-4-4-12 hex digits (like 550e8400-e29b-41d4-a716-446655440000)
                let uuid_string = format!(
                    "{}-{}-{}-{}-{}",
                    &hash_value[..8],
                    &hash_value[8..12],
                    &hash_value[12..16],
                    &hash_value[16..20],
                    &hash_value[20..32]
                );

                let payload = serde_json::json!({
                    "text": chunk_value,
                    "source": filename.clone()
                });
                Point::from_id_vector_payload(&uuid_string, vector, payload)
            })
            .collect();

        let request_body = UpsertPointsRequest { points };
        let response = client
            .put(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()?;

        // Log the response status for debugging
        let status = response.status();
        println!("Qdrant upsert response status: {}", status);
        if !status.is_success() {
            // Try to get the error details from the response body
            let error_text = response
                .text()
                .unwrap_or_else(|_| "Failed to read error response".to_string());
            println!("Qdrant error details: {}", error_text);
        }
        Ok(status == 200)
    }

    /// Searches for points in a Qdrant collection based on a question embedding
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to search in
    /// * `question_vector` - Vector representation of the question
    /// * `limit` - Maximum number of results to return
    /// * `score_threshold` - Only the points with score better than the threshold are returned
    /// * `filter` - Optional filter to apply to the search
    ///
    /// # Returns
    /// * `Result<Vec<ScoredPoint>, String>` - Search results or error message
    pub async fn search_points(
        &self,
        collection_name: &str,
        question_vector: Vec<f32>,
        limit: u64,
        score_threshold: f32,
        filter: Option<serde_json::Value>,
    ) -> Result<Vec<ScoredPoint>, String> {
        let client = reqwest::Client::new();
        let url = format!(
            "http://{}:{}/collections/{}/points/query",
            self.host, self.port, collection_name
        );

        // Create the query request body for the new endpoint
        let mut request_body = serde_json::json!({
            "query": question_vector,
            "limit": limit,
            "score_threshold": score_threshold,
            "with_payload": true,
            "with_vector": false
        });
        // Add filter if provided
        if let Some(f) = filter {
            request_body["filter"] = f;
        }

        let response = client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if response.status() != StatusCode::OK {
            return Err(format!(
                "Request failed with status code: {}",
                response.status()
            ));
        }

        // Store the text response for debugging
        let response_text = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response text: {}", e))?;

        // Parse the JSON response
        let search_response: SearchPointsResponse = match serde_json::from_str(&response_text) {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Failed to parse JSON response: {}", e)),
        };

        // Return the parsed points
        Ok(search_response.result.points)
    }

    /// Blocking version of search_points for synchronous contexts
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to search in
    /// * `question_vector` - Vector representation of the question
    /// * `limit` - Maximum number of results to return
    /// * `score_threshold` - Only the points with score better than the threshold are returned
    /// * `filter` - Optional filter to apply to the search
    ///
    /// # Returns
    /// * `Result<Vec<ScoredPoint>, String>` - Search results or error message
    pub fn search_points_blocking(
        &self,
        collection_name: &str,
        question_vector: Vec<f32>,
        limit: u64,
        score_threshold: f32,
        filter: Option<serde_json::Value>,
    ) -> Result<Vec<ScoredPoint>, String> {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "http://{}:{}/collections/{}/points/query",
            self.host, self.port, collection_name
        );

        // Create the query request body for the new endpoint
        let mut request_body = serde_json::json!({
            "query": question_vector,
            "limit": limit,
            "score_threshold": score_threshold,
            "with_payload": true,
            "with_vector": false
        });

        // Add filter if provided
        if let Some(f) = filter {
            request_body["filter"] = f;
        }

        let response = client
            .post(&url)
            .header("api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if response.status() != StatusCode::OK {
            return Err(format!(
                "Request failed with status code: {}",
                response.status()
            ));
        }

        // Store the text response for debugging
        let response_text = response
            .text()
            .map_err(|e| format!("Failed to read response text: {}", e))?;

        // Parse the JSON response
        let search_response: SearchPointsResponse = match serde_json::from_str(&response_text) {
            Ok(resp) => resp,
            Err(e) => return Err(format!("Failed to parse JSON response: {}", e)),
        };

        // Return the parsed points
        Ok(search_response.result.points)
    }
}
