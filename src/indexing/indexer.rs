//! Indexing module for processing text chunks and storing embeddings.
//!
//! This module handles the process of generating embeddings for text chunks
//! and storing them in the Qdrant vector database. It serves as the bridge
//! between text processing and database storage.

use crate::Config;
use crate::AppError;
use crate::qdrant_custom_client::{QdrantClient, Point};
use crate::clients::ollama::OllamaClient;
use tracing::{info, error, warn};
use serde_json;
use std::time::Instant;

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
    info!("Indexing {} chunks from file: {}", chunks.len(), filename);

    // Start timing the indexing process
    let start_time = Instant::now();

    // Initialize Qdrant client
    let qdrant_client = QdrantClient::new(
        config.qdrant.host.clone(),
        config.qdrant.port,
        config.qdrant.api_key.clone(),
        config.qdrant.vector_size as u64,
        config.qdrant.distance.clone(),
        config.qdrant.limit as u64,
        config.qdrant.score_threshold,
    );

    // Check Qdrant health
    match qdrant_client.health_check().await {
        Ok(response) => {
            info!("Qdrant server is online. Status: {}", response.status);
        }
        Err(e) => {
            error!("Failed to connect to Qdrant server: {}", e);
            return Err(e);
        }
    }

    // Check if collection exists, create if not
    let collection_name = config.qdrant.collection.clone();
    match qdrant_client.collection_exists(&collection_name).await {
        Ok(exists) => {
            if exists {
                info!("Collection '{}' exists in Qdrant", collection_name);
            } else {
                warn!("Collection '{}' does not exist in Qdrant", collection_name);
                info!("Creating collection '{}'...", collection_name);
                match qdrant_client.create_collection(&collection_name).await {
                    Ok(created) => {
                        if created {
                            info!("Collection '{}' created successfully", collection_name);
                        } else {
                            error!("Failed to create collection '{}'", collection_name);
                            return Err(AppError::Qdrant(format!(
                                "Failed to create collection '{}'",
                                collection_name
                            )));
                        }
                    }
                    Err(e) => {
                        error!("Failed to create collection '{}': {}", collection_name, e);
                        return Err(e);
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to check collection existence: {}", e);
            return Err(e);
        }
    }

    // Create Ollama client
    let ollama_client = OllamaClient::new(config);

    // Process chunks in batches
    let batch_size = config.indexing.embeddings_chunk_size;
    for (batch_idx, batch) in chunks.chunks(batch_size).enumerate() {
        info!("Processing batch {}/{} for file: {}", batch_idx + 1, (chunks.len() + batch_size - 1) / batch_size, filename);

        let mut points = Vec::new();

        for (chunk_idx, chunk) in batch.iter().enumerate() {
            // Filter out empty chunks
            if chunk.trim().is_empty() {
                warn!("Skipping empty chunk in batch {} for file: {}", batch_idx + 1, filename);
                continue;
            }

            // Generate embedding for the chunk using OllamaClient
            let embedding = match ollama_client.generate_embedding(chunk).await {
                Ok(emb) => emb,
                Err(e) => {
                    error!("Failed to generate embedding for chunk {}: {}", chunk_idx, e);
                    continue;
                }
            };

            // Generate a simple ID for the point
            let point_id = format!("{}_{}", filename, batch_idx * batch_size + chunk_idx);

            // Create a point for Qdrant
            let point = Point {
                id: serde_json::Value::String(point_id),
                vector: embedding,
                payload: Some(serde_json::json!({
                    "text": chunk,
                    "source_file": filename,
                    "chunk_index": batch_idx * batch_size + chunk_idx
                })),
            };

            points.push(point);
        }

        // Upsert points to Qdrant
        if !points.is_empty() {
            // Convert points to the format expected by upsert_points
            let chunks_data: Vec<(String, Vec<f32>)> = points.iter()
                .map(|p| {
                    let text = p.payload.as_ref()
                        .and_then(|payload| payload.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    (text, p.vector.clone())
                })
                .collect();

            match qdrant_client.upsert_points(&collection_name, filename.to_string(), chunks_data).await {
                Ok(success) => {
                    if success {
                        info!(
                            "Successfully upserted {} points into collection '{}' for file: {}",
                            points.len(),
                            collection_name,
                            filename
                        );
                    } else {
                        error!(
                            "Failed to upsert points into collection '{}' for file: {}",
                            collection_name,
                            filename
                        );
                        return Err(AppError::Qdrant(format!(
                            "Failed to upsert points into collection '{}'",
                            collection_name
                        )));
                    }
                }
                Err(e) => {
                    error!(
                        "Failed to upsert points into collection '{}' for file {}: {}",
                        collection_name, filename, e
                    );
                    return Err(e);
                }
            }
        } else {
            info!("No embeddings to store in Qdrant for batch {} of file: {}", batch_idx + 1, filename);
        }
    }

    // Calculate and display the duration
    let duration = start_time.elapsed();
    let seconds = duration.as_secs();
    let minutes = seconds / 60;
    let remaining_seconds = seconds % 60;
    info!(
        "Indexing completed for file: {} (Duration: {}m {}s)",
        filename, minutes, remaining_seconds
    );
    Ok(())
}
