//! RAG Proxy LLM Caller Module
//!
//! This module handles communication with the LLM (Large Language Model) service.
//! It constructs the appropriate request payload and sends it to the LLM endpoint
//! to get a response based on the provided prompt.

use crate::Config;
use reqwest;
use serde_json::json;

/// Calls the LLM service with the provided prompt
///
/// This function takes a prompt string and sends it to the configured LLM endpoint.
/// It constructs the appropriate request payload in OpenAI API format and returns
/// the LLM's response text.
///
/// # Arguments
/// * `prompt` - The prompt to send to the LLM
/// * `config` - The application configuration
///
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - The LLM's response text or an error
pub async fn call_llm(prompt: &str, config: &Config) -> Result<String, Box<dyn std::error::Error>> {
    // Create the request payload with proper message structure
    let payload = json!({
        "model": config.llm.model,
        "messages": [
            {
                "role": "user",
                "content": prompt
            }
        ],
        "stream": false
    });

    // Create HTTP client
    let client = reqwest::Client::new();

    // Log the request payload for debugging
    println!("LLM Request payload: {:?}", payload);

    // Send request to LLM endpoint
    let response = client
        .post(&config.llm.endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.llm.api_key))
        .json(&payload)
        .send()
        .await?;

    // Log the raw response for debugging
    let raw_response = response.text().await?;
    println!("LLM Raw response: {}", raw_response);

    // Parse the response
    let response_json: serde_json::Value = serde_json::from_str(&raw_response)?;

    // Extract the response text with better error handling
    let response_text = match response_json.get("choices") {
        Some(choices) => {
            if let Some(choice) = choices.as_array().and_then(|arr| arr.first()) {
                // Try to get content from different possible locations
                let content = choice
                    .get("message")
                    .and_then(|msg| msg.get("content"))
                    .or_else(|| choice.get("content"));

                content
                    .and_then(|c| c.as_str())
                    .unwrap_or("No response content")
                    .to_string()
            } else {
                "No response choices".to_string()
            }
        }
        None => {
            // If choices is not present, try to get content directly
            if let Some(content) = response_json.get("content") {
                content
                    .as_str()
                    .unwrap_or("No response content")
                    .to_string()
            } else {
                // If we still don't have content, try to get it from a nested structure
                if let Some(message) = response_json.get("message") {
                    if let Some(content) = message.get("content") {
                        content
                            .as_str()
                            .unwrap_or("No response content")
                            .to_string()
                    } else {
                        "No response content".to_string()
                    }
                } else {
                    "No response content".to_string()
                }
            }
        }
    };

    Ok(response_text)
}
