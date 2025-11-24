//! Indexing module for processing text chunks and storing embeddings.
//!
//! This module handles the process of generating embeddings for text chunks
//! and storing them in the Qdrant vector database. It serves as the bridge
//! between text processing and database storage.

use crate::Config;
use crate::AppError;
use crate::qdrant_custom_client;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;
use std::time::Instant;

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
/// * `Result<(), AppError>` - Ok if successful, error otherwise
pub async fn index_chunks(
    config: &Config,
    chunks: &[String],
    filename: &str,
) -> Result<(), AppError> {
    println!("Indexing {} chunks from file: {}", chunks.len(), filename);

    // Start timing the indexing process
    let start_time = Instant::now();

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
                    AppError::Reqwest(e)
                })?;

            // Log the raw response for debugging
            let response_text = response.text().map_err(|e| {
                eprintln!(
                    "Failed to read response text from Ollama for file {}: {}",
                    filename, e
                );
                AppError::Reqwest(e)
            })?;
            /*/
            println!(
                "Raw Ollama response for file {}: {}",
                filename, response_text
            );
            */
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
            config.qdrant.vector_size as u64,
            config.qdrant.distance.clone(),
            config.qdrant.limit as u64,
            config.qdrant.score_threshold,
        );

        match qdrant_client.health_check().await {
            Ok(response) => {
                println!("Qdrant server is online. Status: {}", response.status);
            }
            Err(e) => {
                eprintln!("Failed to connect to Qdrant server: {}", e);
                return Err(e);
            }
        }

        // Create points for upsert
        let collection_name = config.qdrant.collection.clone();
        let mut points = Vec::new();
        for (i, (embedding, text)) in chunk_embeddings.iter().zip(chunk_texts.iter()).enumerate() {
            let point = qdrant_custom_client::Point {
                id: serde_json::Value::String(format!("{}_{}", filename, i)),
                vector: embedding.clone(),
                payload: Some(serde_json::json!({
                    "text": text,
                    "source_file": filename,
                    "chunk_index": i
                })),
            };
            points.push(point);
        }

        // Upsert points into Qdrant
        match qdrant_client
            .upsert_points(
                &collection_name,
                filename.to_string(),
                chunk_texts
                    .iter()
                    .zip(chunk_embeddings.iter())
                    .map(|(text, embedding)| (text.clone(), embedding.clone()))
                    .collect(),
            )
            .await
        {
            Ok(success) => {
                if success {
                    println!(
                        "Successfully upserted {} points into collection '{}'",
                        chunk_embeddings.len(),
                        collection_name
                    );
                } else {
                    eprintln!(
                        "Failed to upsert points into collection '{}'",
                        collection_name
                    );
                    return Err(AppError::Qdrant(format!(
                        "Failed to upsert points into collection '{}'",
                        collection_name
                    )));
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to upsert points into collection '{}': {}",
                    collection_name, e
                );
                return Err(AppError::Qdrant(format!(
                    "Failed to upsert points into collection '{}': {}",
                    collection_name, e
                )));
            }
        }
    } else {
        println!("No embeddings to store in Qdrant for file: {}", filename);
    }

    // Calculate and display the duration
    let duration = start_time.elapsed();
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    println!(
        "Indexing completed for file: {} (Duration: {}m {}s)",
        filename, minutes, remaining_seconds
    );
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
/// * `Result<(), AppError>` - Ok if successful, error otherwise
pub fn index_chunks_sync(
    config: &Config,
    chunks: &[String],
    filename: &str,
) -> Result<(), AppError> {
    println!("Indexing {} chunks from file: {}", chunks.len(), filename);

    // Start timing the indexing process
    let start_time = Instant::now();

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
                    AppError::Reqwest(e)
                })?;

            // Log the raw response for debugging
            let response_text = response.text().map_err(|e| {
                eprintln!(
                    "Failed to read response text from Ollama for file {}: {}",
                    filename, e
                );
                AppError::Reqwest(e)
            })?;
            /*
            println!(
                "Raw Ollama response for file {}: {}",
                filename, response_text
            );
            */

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
            config.qdrant.vector_size as u64,
            config.qdrant.distance.clone(),
            config.qdrant.limit as u64,
            config.qdrant.score_threshold as f32,
        );

        match qdrant_client.health_check_blocking() {
            Ok(response) => {
                println!("Qdrant server is online. Status: {}", response.status);
            }
            Err(e) => {
                eprintln!("Failed to connect to Qdrant server: {}", e);
                return Err(e);
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
                    // Create the collection
                    println!("Creating collection '{}'...", collection_name);
                    match qdrant_client.create_collection_blocking(&collection_name) {
                        Ok(created) => {
                            if created {
                                println!("Collection '{}' created successfully", collection_name);
                            } else {
                                eprintln!("Failed to create collection '{}'", collection_name);
                                return Err(AppError::Qdrant(format!(
                                    "Failed to create collection '{}'",
                                    collection_name
                                )));
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to create collection '{}': {}", collection_name, e);
                            return Err(e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to check collection existence: {}", e);
                return Err(e);
            }
        }

        // Create points for upsert
        let mut points = Vec::new();
        for (i, (embedding, text)) in chunk_embeddings.iter().zip(chunk_texts.iter()).enumerate() {
            let point = qdrant_custom_client::Point {
                id: serde_json::Value::String(format!("{}_{}", filename, i)),
                vector: embedding.clone(),
                payload: Some(serde_json::json!({
                    "text": text,
                    "source_file": filename,
                    "chunk_index": i
                })),
            };
            points.push(point);
        }

        // Upsert points into Qdrant
        match qdrant_client.upsert_points_blocking(
            &collection_name,
            filename.to_string(),
            chunk_texts
                .iter()
                .zip(chunk_embeddings.iter())
                .map(|(text, embedding)| (text.clone(), embedding.clone()))
                .collect(),
        ) {
            Ok(success) => {
                if success {
                    println!(
                        "Successfully upserted {} points into collection '{}'",
                        chunk_embeddings.len(),
                        collection_name
                    );
                } else {
                    eprintln!(
                        "Failed to upsert points into collection '{}'",
                        collection_name
                    );
                    return Err(AppError::Qdrant(format!(
                        "Failed to upsert points into collection '{}'",
                        collection_name
                    )));
                }
            }
            Err(e) => {
                eprintln!(
                    "Failed to upsert points into collection '{}': {}",
                    collection_name, e
                );
                return Err(AppError::Qdrant(format!(
                    "Failed to upsert points into collection '{}': {}",
                    collection_name, e
                )));
            }
        }
    } else {
        println!("No embeddings to store in Qdrant for file: {}", filename);
    }

    // Calculate and display the duration
    let duration = start_time.elapsed();
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    println!(
        "Indexing completed for file: {} (Duration: {}m {}s)",
        filename, minutes, remaining_seconds
    );
    Ok(())
}
