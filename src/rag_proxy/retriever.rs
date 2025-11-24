//! RAG Proxy Retriever Module
//!
//! This module handles the retrieval of relevant context from Qdrant based on
//! the user's question. It creates embeddings for the question and searches
//! Qdrant for similar documents to provide context for the LLM.

use crate::Config;
use crate::AppError;
use crate::qdrant_custom_client::QdrantClient;
use crate::clients::ollama::OllamaClient;

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
/// * `Result<String, AppError>` - The retrieved context or an error
pub async fn retrieve_context(
    question: &str,
    config: &Config,
) -> Result<String, AppError> {
    // Create Ollama client
    let ollama_client = OllamaClient::new(config);

    // Generate embedding for the question using OllamaClient
    let embedding = ollama_client.generate_embedding(question).await?;

    // Extract embedding from response
    let question_embedding = embedding;

    // Create a Qdrant client
    let qdrant_client = QdrantClient::new(
        config.qdrant.host.clone(),
        config.qdrant.port,
        config.qdrant.api_key.clone(),
        config.qdrant.vector_size as u64,
        config.qdrant.distance.clone(),
        config.qdrant.limit as u64,
        config.qdrant.score_threshold,
    );

    // Search Qdrant for similar documents using the question embedding
    let search_results = qdrant_client
        .search_points(
            &config.qdrant.collection,
            question_embedding,
            config.qdrant.limit,
            config.qdrant.score_threshold,
            None,
        )
        .await
        .map_err(|e| {
            eprintln!("Failed to search Qdrant for question: {}", e);
            e
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
