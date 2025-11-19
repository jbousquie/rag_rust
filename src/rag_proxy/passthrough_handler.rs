//! RAG Proxy Passthrough Handler Module
//!
//! This module contains a handler function for processing incoming requests
//! in passthrough mode, where requests are forwarded directly to the LLM
//! without any RAG processing.

use axum::{Json, body::Bytes, http::{StatusCode, HeaderValue}, response::IntoResponse};
use serde::{Deserialize, Serialize};
use reqwest;

use crate::load_config;

/// Chat completion request structure
/// This matches the OpenAI API format for chat completions
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatCompletionRequest {
    /// The model to use for the completion
    pub model: String,
    /// The messages in the conversation
    pub messages: Vec<ChatMessage>,
    /// Whether to stream the response
    pub stream: Option<bool>,
}

/// Content of a message - can be either a string or an array of content parts
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content
    Text(String),
    /// Array of content parts (used in multimodal messages)
    Parts(Vec<ContentPart>),
}

/// A part of content in a message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContentPart {
    /// Type of the content part
    pub r#type: String,
    /// Text content (only for text type)
    pub text: Option<String>,
}

/// A message in the conversation
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatMessage {
    /// The role of the message sender (user, assistant, system)
    pub role: String,
    /// The content of the message
    pub content: MessageContent,
}

/// Handles incoming requests in passthrough mode
///
/// This function processes an incoming request by forwarding it directly
/// to the configured LLM endpoint without any RAG processing.
///
/// # Arguments
/// * `request` - The incoming request as raw bytes
///
/// # Returns
/// * `impl IntoResponse` - The response from the LLM service
pub async fn handle_passthrough_request(request: Bytes) -> impl IntoResponse {
    // Load configuration
    let config = load_config();

    // Create HTTP client
    let client = reqwest::Client::new();

    // Forward the request to the LLM endpoint
    match client
        .post(&config.llm.endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.llm.api_key))
        .body(request.to_vec())
        .send()
        .await
    {
        Ok(response) => {
            // Get the response body
            match response.text().await {
                Ok(body) => {
                    // Try to parse as JSON to check if it's a valid OpenAI response
                    match serde_json::from_str::<serde_json::Value>(&body) {
                        Ok(json_value) => {
                            // Create response with proper headers
                            let mut response = Json(json_value).into_response();
                            response.headers_mut().insert("Content-Type", HeaderValue::from_static("application/json"));
                            response
                        }
                        Err(_) => {
                            // If not valid JSON, return as text
                            let mut response = body.into_response();
                            response.headers_mut().insert("Content-Type", HeaderValue::from_static("text/plain"));
                            response
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading LLM response body: {}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Error reading LLM response").into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("Error forwarding request to LLM: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to forward request to LLM").into_response()
        }
    }
}