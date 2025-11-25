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
use serde_json::{Value, json}; // Ajout de Value et json

use crate::Config;
use crate::AppError;
use crate::rag_proxy::retriever::retrieve_context;
use crate::clients::llm::LlmClient;

/// Handles incoming RAG requests
///
/// This function processes an incoming chat completion request by:
/// 1. Parsing the request dynamically to preserve its structure
/// 2. Extracting the user's question from the messages
/// 3. Retrieving relevant context from Qdrant using the question
/// 4. Modifying the request by injecting the RAG context into the system message
/// 5. Forwarding the modified request to the LLM endpoint
/// 6. Returning the LLM's response to the client
///
/// This approach uses dynamic JSON manipulation to ensure compatibility
/// with various clients while enhancing the system message with RAG context.
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
    // 1. Convert request bytes to a dynamic JSON Value
    let mut request_json: Value = serde_json::from_slice(&request).map_err(|e| {
        eprintln!("Failed to parse request as JSON: {}", e);
        AppError::Json(e)
    })?;

    // 2. Extract the user's question from the messages
    let user_question = {
        // Navigate the dynamic Value to find the user's question
        if let Some(messages) = request_json.get_mut("messages").and_then(|v| v.as_array_mut()) {
            // Look for the last user message as the question
            let mut question = "No question provided".to_string();
            for msg in messages.iter().rev() {
                if let (Some(role), Some(content)) = (msg.get("role").and_then(|v| v.as_str()), msg.get("content")) {
                    if role == "user" {
                        if let Some(text) = content.as_str() {
                            question = text.to_string();
                            break;
                        } else if let Some(parts) = content.as_array() {
                            // Handle multimodal messages by concatenating text parts
                            let text_parts: Vec<String> = parts
                                .iter()
                                .filter_map(|part| {
                                    if part.get("type").and_then(|t| t.as_str()) == Some("text") {
                                        part.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            question = text_parts.join(" ");
                            break;
                        }
                    }
                }
            }
            question
        } else {
            // Fallback if 'messages' array is not found
            eprintln!("No 'messages' array found in request");
            "No question provided".to_string()
        }
    };

    // 3. Retrieve relevant context from Qdrant
    let context = retrieve_context(&user_question, &config).await?;

    // 4. Inject RAG context into the system message
    if !context.is_empty() {
        let new_context = format!("--- Context from: RAG ---\n{}", context);

        // Navigate the Value to find and modify the messages
        if let Some(messages_val) = request_json.get_mut("messages") {
            if let Some(messages_arr) = messages_val.as_array_mut() {
                let mut system_found = false;
                for msg in messages_arr.iter_mut() {
                    // Check if it's a system message
                    if let Some(role_val) = msg.get("role").and_then(|v| v.as_str()) {
                        if role_val == "system" {
                            // Modify the system message content
                            if let Some(content_val) = msg.get_mut("content") {
                                // Handle both text and array content types
                                let original_content = if let Some(text) = content_val.as_str() {
                                    text.to_string()
                                } else if let Some(parts) = content_val.as_array() {
                                    // For array content, reconstruct text or handle as needed
                                    let text_parts: Vec<String> = parts
                                        .iter()
                                        .filter_map(|part| {
                                            if part.get("type").and_then(|t| t.as_str()) == Some("text") {
                                                part.get("text").and_then(|t| t.as_str()).map(|s| s.to_string())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                    text_parts.join(" ")
                                } else {
                                    // Default if content type is unknown
                                    eprintln!("Unknown content type in system message, appending to empty string.");
                                    "".to_string()
                                };
                                // Update content, converting to string format for simplicity
                                // A more robust solution might preserve the array format
                                let updated_content = format!("{}{}", original_content, new_context);
                                *content_val = Value::String(updated_content);
                            }
                            system_found = true;
                            break; // Only update the first system message found
                        }
                    }
                }
                if !system_found {
                    // If no system message exists, add a new one at the beginning
                    let new_system_msg = json!({
                        "role": "system",
                        "content": new_context
                    });
                    messages_arr.insert(0, new_system_msg);
                }
            } else {
                eprintln!("'messages' field is not an array, cannot inject RAG context.");
            }
        } else {
            eprintln!("'messages' field not found in request, cannot inject RAG context.");
        }
    }

    // 5. Convert the modified Value back to a JSON string
    let modified_request_str = serde_json::to_string(&request_json).map_err(|e| {
        eprintln!("Failed to serialize modified request: {}", e);
        AppError::Json(e)
    })?;

    // 6. Send the modified request to the LLM client
    let llm_client = LlmClient::new(&config);
    let llm_response = llm_client.send_request(modified_request_str).await?;
    let llm_response_body = llm_response.text().await.map_err(|e| {
        eprintln!("Error reading LLM response body: {}", e);
        AppError::Reqwest(e)
    })?;

    // 7. Return the LLM response to the client
    match serde_json::from_str::<Value>(&llm_response_body) {
        Ok(json_value) => {
            let mut response = axum::Json(json_value).into_response();
            response.headers_mut().insert(
                "Content-Type",
                axum::http::HeaderValue::from_static("application/json"),
            );
            Ok(response)
        }
        Err(_) => {
            let mut response = llm_response_body.into_response();
            response.headers_mut().insert(
                "Content-Type",
                axum::http::HeaderValue::from_static("text/plain"),
            );
            Ok(response)
        }
    }
}
