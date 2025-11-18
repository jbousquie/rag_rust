//! RAG Proxy Retriever Module
//!
//! This module handles the retrieval of relevant context from Qdrant based on
//! the user's question. It creates embeddings for the question and searches
//! Qdrant for similar documents to provide context for the LLM.

use crate::Config;
use crate::qdrant_custom_client::QdrantClient;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    prompt: String,
}

// The response structure from Ollama's embeddings API
#[derive(Deserialize)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

/// Retrieves relevant context from Qdrant based on the user's question
///
/// This function takes a user question, creates an embedding for it, and
/// searches Qdrant for similar documents to retrieve relevant context.
/// It follows the same pattern as the indexing process:
/// 1. Create an embedding for the question using Ollama
/// 2. Search Qdrant for similar documents
/// 3. Return the relevant context
///
/// # Arguments
/// * `question` - The user's question as a string slice
/// * `config` - The application configuration
///
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - The retrieved context or an error
pub async fn retrieve_context(
    question: &str,
    config: &Config,
) -> Result<String, Box<dyn std::error::Error>> {
    // Create an embedding for the question using Ollama
    let ollama_endpoint = format!("{}/api/embeddings", config.embeddings.endpoint);
    let model_name = config.embeddings.model.clone();

    let request = EmbeddingRequest {
        model: model_name,
        prompt: question.to_string(),
    };

    // Call Ollama to generate embeddings
    let client = reqwest::Client::new();
    let response = client
        .post(&ollama_endpoint)
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            eprintln!("Failed to send request to Ollama for question: {}", e);
            format!("Failed to send request to Ollama: {}", e)
        })?;

    // Parse the JSON response
    let response_text = response.text().await.map_err(|e| {
        eprintln!(
            "Failed to read response text from Ollama for question: {}",
            e
        );
        format!("Failed to read response text from Ollama: {}", e)
    })?;
    let embedding_response: OllamaEmbeddingResponse = serde_json::from_str(&response_text)
        .map_err(|e| {
            eprintln!("Failed to parse Ollama response for question: {}", e);
            format!("Failed to parse Ollama response: {}", e)
        })?;

    // Extract embedding from response
    let question_embedding = embedding_response.embedding;

    // Create a Qdrant client
    let qdrant_client = QdrantClient::new(
        config.qdrant.host.clone(),
        config.qdrant.port,
        config.qdrant.api_key.clone(),
        config.qdrant.vector_size as u64,
        config.qdrant.distance.clone(),
        config.qdrant.limit as u64,
    );

    // Search Qdrant for similar documents using the question embedding
    let search_results = qdrant_client
        .search_points(
            &config.qdrant.collection,
            question_embedding,
            config.qdrant.limit,
            None,
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to search Qdrant for question: {}", e);
            format!("Failed to search Qdrant: {}", e)
        })?;

    // Extract the text content from the search results
    let context = search_results
        .into_iter()
        .filter_map(|point| point.payload)
        .map(|payload| payload.text)
        .collect::<Vec<String>>()
        .join("\n\n");

    Ok(context)
}
