//! Indexing module for processing text chunks and storing embeddings.
//!
//! This module handles the process of generating embeddings for text chunks
//! and storing them in the Qdrant vector database. It serves as the bridge
//! between text processing and database storage.

use crate::Config;
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

    // Prepare request to Ollama for embeddings
    let ollama_endpoint = format!("{}/api/embeddings", config.embeddings.endpoint);
    let model_name = config.embeddings.model.clone();

    // Process chunks in batches to avoid overwhelming Ollama
    let batch_size = config.indexing.embeddings_chunk_size;
    for (i, chunk_batch) in chunks.chunks(batch_size).enumerate() {
        println!("Processing batch {} of {} for file: {}", i + 1, chunks.chunks(batch_size).len(), filename);

        // Filter out empty chunks to avoid sending empty strings to Ollama
        let non_empty_chunks: Vec<String> = chunk_batch.iter()
            .filter(|chunk| !chunk.trim().is_empty())
            .cloned()
            .collect();

        if non_empty_chunks.is_empty() {
            println!("Skipping empty chunks for file: {}", filename);
            continue;
        }

        // Process each chunk individually since Ollama's embeddings API expects a single prompt
        for chunk in non_empty_chunks {
            let request = EmbeddingRequest {
                model: model_name.clone(),
                prompt: chunk,
            };

            // Call Ollama to generate embeddings (using blocking client)
            let client = reqwest::blocking::Client::new();
            let response = client
                .post(&ollama_endpoint)
                .json(&request)
                .send()
                .map_err(|e| {
                    eprintln!("Failed to send request to Ollama for file {}: {}", filename, e);
                    format!("Failed to send request to Ollama: {}", e)
                })?;

            // Log the raw response for debugging
            let response_text = response.text().map_err(|e| {
                eprintln!("Failed to read response text from Ollama for file {}: {}", filename, e);
                format!("Failed to read response text from Ollama: {}", e)
            })?;

            println!("Raw Ollama response for file {}: {}", filename, response_text);

            // Parse the JSON response - Ollama returns a single embedding at a time
            let embedding_response: OllamaEmbeddingResponse = serde_json::from_str(&response_text)
                .map_err(|e| {
                    eprintln!("Failed to parse Ollama response for file {}: {}", filename, e);
                    format!("Failed to parse Ollama response: {}", e)
                })?;

            // Extract embedding from response
            chunk_embeddings.push(embedding_response.embedding);
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