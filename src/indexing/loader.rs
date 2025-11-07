//! File loading module for reading document content.
//!
//! This module provides functionality to load content from various file types
//! including text files, PDFs, and DOCX documents. It handles both synchronous
//! and asynchronous file reading operations.

use std::fs;
use std::path::Path;
use crate::Config;

/// Asynchronously loads file content from disk
/// 
/// # Arguments
/// * `config` - Configuration object containing data sources path
/// * `filename` - Name of the file to load
/// 
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - File content if successful, error otherwise
pub async fn load_file(config: &Config, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = Path::new(&config.data_sources.path).join(filename);
    let content = tokio::fs::read_to_string(&file_path).await?;
    Ok(content)
}

/// Synchronously loads file content from disk
/// 
/// # Arguments
/// * `config` - Configuration object containing data sources path
/// * `filename` - Name of the file to load
/// 
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - File content if successful, error otherwise
pub fn load_file_sync(config: &Config, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = Path::new(&config.data_sources.path).join(filename);
    let content = fs::read_to_string(&file_path)?;
    Ok(content)
}
