//! Main indexing binary for processing documents and generating embeddings.
//!
//! This binary is responsible for reading documents from the data_sources directory,
//! processing them through the indexing pipeline (chunking, embedding generation,
//! and storage in Qdrant), and tracking which files have been processed.

use std::fs;
use std::path::Path;
use rag_rust::Config;
use rag_rust::indexing::{loader, chunker, indexer, file_tracker};
use md5::Digest;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting document indexing...");

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
    let files_to_process = tracker.get_changed_files(&files);

    // Process files
    for file_name in files_to_process {
        println!("Processing file: {}", file_name);

        // Load file content (synchronously)
        let content = loader::load_file_sync(&config, &file_name)?;

        // Chunk content
        let chunks = chunker::chunk_text(&content, 512);

        // Index chunks
        indexer::index_chunks(&config, &chunks, &file_name)?;

        // Update tracker with new MD5
        if let Ok(file_content) = fs::read(&file_name) {
            let md5 = format!("{:x}", md5::Md5::digest(&file_content));
            tracker.set_file_md5(file_name, md5);
        }
    }

    // Save updated tracker
    tracker.save_to_file(&tracker_path)?;

    println!("Document indexing completed successfully!");
    Ok(())
}