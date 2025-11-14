//! Indexing module for processing text chunks and storing embeddings.
//!
//! This module handles the process of generating embeddings for text chunks
//! and storing them in the Qdrant vector database. It serves as the bridge
//! between text processing and database storage.

use crate::Config;
use crate::qdrant_custom_client;
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
pub async fn index_chunks(
    config: &Config,
    chunks: &[String],
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Indexing {} chunks from file: {}", chunks.len(), filename);

    // Generate embeddings for all chunks
    let mut chunk_embeddings = Vec::new();
    let mut chunk_texts = Vec::new();

    // Prepare request to Ollama for embeddings
    let ollama_endpoint = format!("{}/api/embeddings", config.embeddings.endpoint);
    let model_name = config.embeddings.model.clone();

    // Process chunks in batches to avoid overwhelming Ollama
    let batch_size = config.indexing.embeddings_chunk_size;
    for (i, chunk_batch) in chunks.chunks(batch_size).enumerate() {
        println!(
            "Processing batch {} of {} for file: {}",
            i + 1,
            chunks.chunks(batch_size).len(),
            filename
        );

        // Filter out empty chunks to avoid sending empty strings to Ollama
        let non_empty_chunks: Vec<String> = chunk_batch
            .iter()
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
                prompt: chunk.clone(),
            };

            // Call Ollama to generate embeddings (using blocking client)
            let client = reqwest::blocking::Client::new();
            let response = client
                .post(&ollama_endpoint)
                .json(&request)
                .send()
                .map_err(|e| {
                    eprintln!(
                        "Failed to send request to Ollama for file {}: {}",
                        filename, e
                    );
                    format!("Failed to send request to Ollama: {}", e)
                })?;

            // Log the raw response for debugging
            let response_text = response.text().map_err(|e| {
                eprintln!(
                    "Failed to read response text from Ollama for file {}: {}",
                    filename, e
                );
                format!("Failed to read response text from Ollama: {}", e)
            })?;

            println!(
                "Raw Ollama response for file {}: {}",
                filename, response_text
            );

            // Parse the JSON response - Ollama returns a single embedding at a time
            let embedding_response: OllamaEmbeddingResponse = serde_json::from_str(&response_text)
                .map_err(|e| {
                    eprintln!(
                        "Failed to parse Ollama response for file {}: {}",
                        filename, e
                    );
                    format!("Failed to parse Ollama response: {}", e)
                })?;

            // Extract embedding from response
            chunk_embeddings.push(embedding_response.embedding);
            chunk_texts.push(chunk);
        }
    }

    // Connect to Qdrant and store embeddings
    if !chunk_embeddings.is_empty() {
        println!("Connecting to Qdrant for file: {}", filename);

        // Check if Qdrant server is online using our custom client
        let qdrant_client = qdrant_custom_client::QdrantClient::new(
            config.qdrant.host.clone(),
            config.qdrant.port,
            config.qdrant.api_key.clone(),
        );

        match qdrant_client.health_check().await {
            Ok(response) => {
                println!("Qdrant server is online. Status: {}", response.status);
            }
            Err(e) => {
                eprintln!("Failed to connect to Qdrant server: {}", e);
                return Err(format!("Failed to connect to Qdrant server: {}", e).into());
            }
        }

        // Create Qdrant client
    } else {
        println!("No embeddings to store in Qdrant for file: {}", filename);
    }

    println!("Indexing completed for file: {}", filename);
    Ok(())
}

/// Synchronous version of index_chunks for use in blocking contexts
///
/// # Arguments
/// * `config` - Configuration object containing indexing settings
/// * `chunks` - Vector of text chunks to index
/// * `filename` - Name of the source file being indexed
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Ok if successful, error otherwise
pub fn index_chunks_sync(
    config: &Config,
    chunks: &[String],
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Indexing {} chunks from file: {}", chunks.len(), filename);

    // Generate embeddings for all chunks
    let mut chunk_embeddings = Vec::new();
    let mut chunk_texts = Vec::new();

    // Prepare request to Ollama for embeddings
    let ollama_endpoint = format!("{}/api/embeddings", config.embeddings.endpoint);
    let model_name = config.embeddings.model.clone();

    // Process chunks in batches to avoid overwhelming Ollama
    let batch_size = config.indexing.embeddings_chunk_size;
    for (i, chunk_batch) in chunks.chunks(batch_size).enumerate() {
        println!(
            "Processing batch {} of {} for file: {}",
            i + 1,
            chunks.chunks(batch_size).len(),
            filename
        );

        // Filter out empty chunks to avoid sending empty strings to Ollama
        let non_empty_chunks: Vec<String> = chunk_batch
            .iter()
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
                prompt: chunk.clone(),
            };

            // Call Ollama to generate embeddings (using blocking client)
            let client = reqwest::blocking::Client::new();
            let response = client
                .post(&ollama_endpoint)
                .json(&request)
                .send()
                .map_err(|e| {
                    eprintln!(
                        "Failed to send request to Ollama for file {}: {}",
                        filename, e
                    );
                    format!("Failed to send request to Ollama: {}", e)
                })?;

            // Log the raw response for debugging
            let response_text = response.text().map_err(|e| {
                eprintln!(
                    "Failed to read response text from Ollama for file {}: {}",
                    filename, e
                );
                format!("Failed to read response text from Ollama: {}", e)
            })?;

            println!(
                "Raw Ollama response for file {}: {}",
                filename, response_text
            );

            // Parse the JSON response - Ollama returns a single embedding at a time
            let embedding_response: OllamaEmbeddingResponse = serde_json::from_str(&response_text)
                .map_err(|e| {
                    eprintln!(
                        "Failed to parse Ollama response for file {}: {}",
                        filename, e
                    );
                    format!("Failed to parse Ollama response: {}", e)
                })?;

            // Extract embedding from response
            chunk_embeddings.push(embedding_response.embedding);
            chunk_texts.push(chunk);
        }
    }

    // Connect to Qdrant and store embeddings
    if !chunk_embeddings.is_empty() {
        println!("Connecting to Qdrant for file: {}", filename);

        // Check if Qdrant server is online using our custom client
        let qdrant_client = qdrant_custom_client::QdrantClient::new(
            config.qdrant.host.clone(),
            config.qdrant.port,
            config.qdrant.api_key.clone(),
        );

        match qdrant_client.health_check_blocking() {
            Ok(response) => {
                println!("Qdrant server is online. Status: {}", response.status);
            }
            Err(e) => {
                eprintln!("Failed to connect to Qdrant server: {}", e);
                return Err(format!("Failed to connect to Qdrant server: {}", e).into());
            }
        }

        // Check if collection exists
        let collection_name = config.qdrant.collection.clone();
        match qdrant_client.collection_exists_blocking(&collection_name) {
            Ok(exists) => {
                if exists {
                    println!("Collection '{}' exists in Qdrant", collection_name);
                } else {
                    println!("Collection '{}' does not exist in Qdrant", collection_name);
                }
            }
            Err(e) => {
                eprintln!("Failed to check collection existence: {}", e);
                return Err(format!("Failed to check collection existence: {}", e).into());
            }
        }

        // Create Qdrant client
    } else {
        println!("No embeddings to store in Qdrant for file: {}", filename);
    }

    println!("Indexing completed for file: {}", filename);
    Ok(())
}
