//! RAG Proxy Handler Module
//!
//! This module contains the handler function for processing incoming RAG requests.
//! It implements the core logic for the RAG pipeline: extracting the question from
//! the request, retrieving relevant context from Qdrant, and calling the LLM with
//! the combined prompt to generate a response.

use axum::{
    body::Bytes,
    response::IntoResponse,
    extract::State,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::Config;
use crate::AppError;
use crate::rag_proxy::retriever::retrieve_context;
use crate::clients::llm::LlmClient;

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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    /// The role of the message sender (user, assistant, system)
    pub role: String,
    /// The content of the message
    pub content: MessageContent,
}


/// Handles incoming RAG requests
///
/// This function processes an incoming chat completion request by:
/// 1. Extracting the user's question from the messages
/// 2. Retrieving relevant context from Qdrant using the question
/// 3. Modifying the original JSON string by replacing system message content with enhanced context
/// 4. Forwarding the modified request directly to the LLM endpoint
/// 5. Returning the LLM's response directly to the client
///
/// This approach preserves the exact JSON structure of the original request
/// to ensure compatibility with various clients like QwenCLI while enhancing
/// the system message with RAG context. The implementation is similar to
/// passthrough mode but with the addition of context injection.
///
/// # Arguments
/// * `request` - The incoming chat completion request as raw bytes
///
/// # Returns
/// * `Result<impl IntoResponse, AppError>` - The response from the LLM service
pub async fn handle_rag_request(
    State(config): State<Arc<Config>>,
    request: Bytes
) -> Result<impl IntoResponse, AppError> {
    // Convert bytes to string for JSON manipulation
    // Convert bytes to string for JSON manipulation
    let request_str = std::str::from_utf8(&request).map_err(|e| {
        eprintln!("Failed to parse request as UTF-8: {}", e);
        AppError::Unknown(format!("Invalid UTF-8: {}", e))
    })?;

    // Parse the request to extract the user's question
    let parsed_request: ChatCompletionRequest = serde_json::from_slice(&request)?;

    let user_question = {
        // Extract the user's question from the messages
        parsed_request.messages
            .last() // Use the last message as the user question (most recent user input)
            .or_else(|| parsed_request.messages.iter().find(|msg| msg.role == "user")) // Fallback to find any user message
            .map(|msg| {
                match &msg.content {
                    MessageContent::Text(text) => text.clone(),
                    MessageContent::Parts(parts) => {
                        // For multimodal messages, concatenate all text parts
                        parts
                            .iter()
                            .filter_map(|part| {
                                if part.r#type == "text" {
                                    part.text.clone()
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                            .join(" ")
                    }
                }
            })
            .unwrap_or_else(|| "No question provided".to_string())
    };

    // Configuration is now injected via State

    // Retrieve relevant context from Qdrant
    // Retrieve relevant context from Qdrant
    let context = retrieve_context(&user_question, &config).await?;

    // If we have context, modify the original JSON string by replacing system message content
    let modified_request_str = if !context.is_empty() {
        // Format the new context
        let new_context = format!("--- Context from: RAG ---\n{}", context);

        // Extract original system message content to identify what to replace
        let request_json: serde_json::Value = serde_json::from_str(request_str)?;

        let original_system_content = if let Some(messages) = request_json.get("messages") {
            if let Some(messages_array) = messages.as_array() {
                // Look for the last system message to identify its content
                let mut result = None;
                for msg in messages_array.iter().rev() {
                    if msg.get("role").and_then(|r| r.as_str()) == Some("system") {
                        if let Some(content) = msg.get("content") {
                            if let Some(content_str) = content.as_str() {
                                // Extract the original system content to replace
                                result = Some(content_str.to_string());
                                break;
                            } else {
                                // For complex content with parts
                                match serde_json::from_value::<MessageContent>(content.clone()) {
                                    Ok(MessageContent::Text(text)) => {
                                        result = Some(text);
                                        break;
                                    },
                                    Ok(MessageContent::Parts(parts)) => {
                                        // For parts, concatenate text parts
                                        let text_parts: Vec<String> = parts
                                            .iter()
                                            .filter_map(|part| {
                                                if part.r#type == "text" && part.text.is_some() {
                                                    Some(part.text.clone().unwrap_or_default())
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect();
                                        result = Some(text_parts.join(" "));
                                        break;
                                    },
                                    Err(_) => continue,
                                }
                            }
                        }
                    }
                }
                result
            } else {
                None
            }
        } else {
            None
        };

        // If we found the original system content, replace it in the original JSON string
        if let Some(original_content) = original_system_content {
            // Get the fingerprint length from configuration
            let fingerprint_length = config.rag_proxy.system_message_fingerprint_length;

            // Get the last fingerprint_length characters of the original content
            let fingerprint_end = if original_content.len() > fingerprint_length {
                original_content.len() - fingerprint_length
            } else {
                0
            };
            let fingerprint = &original_content[fingerprint_end..];

            // Escape the fingerprint and new content for JSON
            let escaped_fingerprint = serde_json::to_string(fingerprint)
                .unwrap_or_else(|_| serde_json::Value::String(fingerprint.to_string()).to_string());
            let escaped_new_context = serde_json::to_string(&new_context)
                .unwrap_or_else(|_| serde_json::Value::String(new_context.clone()).to_string());

            // Remove the quotes added by serde_json::to_string
            let escaped_fingerprint = &escaped_fingerprint[1..escaped_fingerprint.len()-1];
            let escaped_new_context = &escaped_new_context[1..escaped_new_context.len()-1];

            // Create the replacement string: fingerprint concatenated with new context
            let replacement = format!("{}{}", fingerprint, escaped_new_context);

            // Replace the fingerprint with the replacement in the JSON string
            request_str.replace(escaped_fingerprint, &replacement)
        } else {
            // If no system message was found, add a new one by finding the messages array
            // and inserting a new system message at the beginning
            let mut modified_str = request_str.to_string();

            // Find the beginning of the "messages" array by looking for "messages":[
            if let Some(messages_start) = modified_str.find(r#""messages":["#) {
                // Find the position after the colon and opening bracket
                let array_start = messages_start + 10; // length of ""messages":["

                // Insert the new system message at the beginning of the messages array
                let escaped_new_context = serde_json::to_string(&new_context)
                    .unwrap_or_else(|_| serde_json::Value::String(new_context.clone()).to_string());

                // Remove the quotes added by serde_json::to_string
                let escaped_new_context = &escaped_new_context[1..escaped_new_context.len()-1];

                let system_message_json = format!(
                    r#"{{"role":"system","content":"{}"}}"#,
                    escaped_new_context
                );

                // Insert the new system message at the beginning of the messages array
                modified_str.insert_str(array_start, &format!("{},", system_message_json));

                modified_str
            } else {
                // If no messages field is found, return the original string
                request_str.to_string()
            }
        }
    } else {
        // If no context, use the original request string
        request_str.to_string()
    };

    // Create LLM client
    let llm_client = LlmClient::new(&config);

    // Send the modified request directly to the LLM endpoint
    let llm_response = llm_client.send_request(modified_request_str).await?;

    // Get the response body from the LLM
    let llm_response_body = llm_response.text().await.map_err(|e| {
        eprintln!("Error reading LLM response body: {}", e);
        AppError::Reqwest(e)
    })?;

    // Try to parse the response as JSON
    // Try to parse the response as JSON
    match serde_json::from_str::<serde_json::Value>(&llm_response_body) {
        Ok(json_value) => {
            // Create response with proper headers
            let mut response = axum::Json(json_value).into_response();
            response.headers_mut().insert(
                "Content-Type",
                axum::http::HeaderValue::from_static("application/json"),
            );
            Ok(response)
        }
        Err(_) => {
            // If not valid JSON, return as text
            let mut response = llm_response_body.into_response();
            response.headers_mut().insert(
                "Content-Type",
                axum::http::HeaderValue::from_static("text/plain"),
            );
            Ok(response)
        }
    }
}
