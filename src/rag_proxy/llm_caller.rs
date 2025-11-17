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
    // Create the request payload
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

    // Send request to LLM endpoint
    let response = client
        .post(&config.llm.endpoint)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.llm.api_key))
        .json(&payload)
        .send()
        .await?;

    // Parse the response
    let response_json: serde_json::Value = response.json().await?;

    // Extract the response text
    let response_text = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("No response content")
        .to_string();

    Ok(response_text)
}