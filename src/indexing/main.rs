//! Main indexing binary for processing documents and generating embeddings.
//!
//! This binary is responsible for reading documents from the data_sources directory,
//! processing them through the indexing pipeline (chunking, embedding generation,
//! and storage in Qdrant), and tracking which files have been processed.
//! The file tracking system ensures that only new or changed files are re-processed,
//! significantly improving performance when re-running the indexing process.

use std::fs;
use std::path::Path;
use rag_rust::Config;
use rag_rust::indexing::{loader, chunker, indexer, file_tracker};
use md5::Digest;
use rag_rust::init_logging;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    // Load configuration
    let config = Config::load()?;

    // Initialize file tracker
    let mut tracker = file_tracker::FileTracker::new();
    let tracker_path = file_tracker::FileTracker::get_tracker_path(&config);
    tracker.load_from_file(&tracker_path)?;

    // Get all files in data_sources directory
    let data_sources_path = Path::new(&config.indexing.path);
    let mut files = Vec::new();

    if data_sources_path.exists() {
        for entry in fs::read_dir(data_sources_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(file_name) = path.file_name() {
                    if let Some(file_name_str) = file_name.to_str() {
                        files.push(file_name_str.to_string());
                    }
                }
            }
        }
    }

    // Filter files that need to be processed (new or changed)
    let files_to_process = tracker.get_changed_files(&files, &config.indexing.path);

    info!("Found {} files to process", files_to_process.len()); // Added this line as per instruction's implied intent

    // Process files
    for file_name in files_to_process {
        info!("Processing file: {}", file_name);

        // Load file content (synchronously)
        let content = loader::load_file_sync(&config, &file_name)?;

        // Chunk content
        let chunks = chunker::chunk_text(&content, config.indexing.chunk_size);

        // Index chunks - make this synchronous
        // The instruction implies changing to async indexer and handling its error with tracing::error
        if let Err(e) = indexer::index_chunks(&config, &chunks, &file_name).await {
            error!("Failed to index chunks for {}: {}", file_name, e);
            continue;
        }

        // Update tracker with new MD5
        let full_path = Path::new(&config.indexing.path).join(&file_name);
        if let Ok(file_content) = fs::read(&full_path) {
            let md5 = format!("{:x}", md5::Md5::digest(&file_content));
            tracker.set_file_md5(file_name, md5);
        }
    }

    // Save updated tracker
    tracker.save_to_file(&tracker_path)?;

    info!("Document indexing completed successfully!");
    Ok(())
}