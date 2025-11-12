//! Indexing module for processing text chunks and storing embeddings.
//!
//! This module handles the process of generating embeddings for text chunks
//! and storing them in the Qdrant vector database. It serves as the bridge
//! between text processing and database storage.

use crate::Config;
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct EmbeddingRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

/// Indexes text chunks by generating embeddings and storing them in Qdrant
///
/// # Arguments
/// * `config` - Configuration object containing indexing settings
/// * `chunks` - Vector of text chunks to index
/// * `filename` - Name of the source file being indexed
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if successful, error otherwise
pub fn index_chunks(config: &Config, chunks: &[String], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Indexing {} chunks from file: {}", chunks.len(), filename);

    // Generate embeddings for all chunks
    let mut chunk_embeddings = Vec::new();

    // Prepare request to Ollama
    let ollama_endpoint = format!("{}/api/embeddings", config.llm.endpoint);
    let model_name = config.llm.model.clone();

    // Process chunks in batches to avoid overwhelming Ollama
    let batch_size = config.indexing.embeddings_chunk_size;
    for chunk_batch in chunks.chunks(batch_size) {
        let request = EmbeddingRequest {
            model: model_name.clone(),
            input: chunk_batch.to_vec(),
        };

        // Call Ollama to generate embeddings (using blocking client)
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&ollama_endpoint)
            .json(&request)
            .send()
            .map_err(|e| format!("Failed to send request to Ollama: {}", e))?;

        let embedding_response: EmbeddingResponse = response
            .json()
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        // Extract embeddings from response
        for embedding_data in embedding_response.data {
            chunk_embeddings.push(embedding_data.embedding);
        }
    }

    // At this point, we would store the chunks and their embeddings in Qdrant
    // For now, we just print the information
    println!("Generated {} embeddings for file: {}", chunk_embeddings.len(), filename);

    // In a real implementation, we would:
    // 1. Connect to Qdrant
    // 2. Create a collection if it doesn't exist
    // 3. Insert the chunks and their embeddings into Qdrant

    println!("Qdrant integration would be implemented here");

    Ok(())
}