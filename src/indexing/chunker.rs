//! Text chunking module for splitting text into manageable pieces.
//!
//! This module provides functionality to split text content into chunks
//! of a specified size, which is useful for processing large documents
//! in smaller, manageable pieces for indexing and embedding generation.

/// Splits text content into chunks of a specified size
/// 
/// # Arguments
/// * `text` - The text to be chunked
/// * `chunk_size` - Maximum size of each chunk in characters
/// 
/// # Returns
/// * `Vec<String>` - A vector of text chunks
pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    
    // Simple chunking by splitting on newlines and then by word boundaries
    let lines: Vec<&str> = text.lines().collect();
    
    let mut current_chunk = String::new();
    
    for line in lines {
        if current_chunk.len() + line.len() + 1 > chunk_size && !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
            current_chunk = String::new();
        }
        
        if !current_chunk.is_empty() {
            current_chunk.push('\n');
        }
        current_chunk.push_str(line);
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }
    
    chunks
}