//! File loading module for reading document content.
//!
//! This module provides functionality to load content from various file types
//! including text files, PDFs, and DOCX documents. It handles both synchronous
//! and asynchronous file reading operations.
//!
//! For PDF files, this module uses the pdf-extract crate to extract text content
//! from PDF documents. The implementation includes robust error handling to
//! prevent panics when processing large or problematic PDF files.

use std::fs;
use std::path::Path;
use crate::Config;
use pdf_extract::extract_text;

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
    
    // Check file extension to determine loading method
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("pdf") => {
            // Load PDF content using pdf-extract crate
            match load_pdf_file(&file_path).await {
                Ok(content) => Ok(content),
                Err(e) => {
                    eprintln!("Warning: Failed to read PDF file '{}': {}", filename, e);
                    // Return empty string instead of panicking
                    Ok(String::new())
                }
            }
        }
        _ => {
            // Default to text file loading
            let content = tokio::fs::read_to_string(&file_path).await?;
            Ok(content)
        }
    }
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
    
    // Check file extension to determine loading method
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("pdf") => {
            // Load PDF content using pdf-extract crate
            match load_pdf_file_sync(&file_path) {
                Ok(content) => Ok(content),
                Err(e) => {
                    eprintln!("Warning: Failed to read PDF file '{}': {}", filename, e);
                    // Return empty string instead of panicking
                    Ok(String::new())
                }
            }
        }
        _ => {
            // Default to text file loading
            let content = fs::read_to_string(&file_path)?;
            Ok(content)
        }
    }
}

/// Asynchronously loads PDF file content using pdf-extract crate
///
/// # Arguments
/// * `file_path` - Path to the PDF file
///
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - PDF content if successful, error otherwise
async fn load_pdf_file(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Extract text from PDF using pdf-extract crate
    // Use spawn_blocking since pdf-extract is synchronous
    let path = file_path.to_path_buf();
    match tokio::task::spawn_blocking({
        let path = path.clone();
        move || {
            // Use std::panic::catch_unwind to catch any panics from pdf-extract
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                extract_text(&path)
            }))
        }
    }).await {
        Ok(Ok(Ok(text))) => Ok(text),
        Ok(Ok(Err(e))) => {
            eprintln!("Warning: Failed to extract text from PDF file '{:?}': {}", file_path, e);
            // Return empty string instead of erroring
            Ok(String::new())
        },
        Ok(Err(panic_err)) => {
            eprintln!("Warning: PDF extraction panicked for file '{:?}': {:?}", file_path, panic_err);
            // Return empty string instead of panicking
            Ok(String::new())
        },
        Err(join_err) => {
            eprintln!("Warning: Task join error for PDF file '{:?}': {:?}", file_path, join_err);
            // Return empty string
            Ok(String::new())
        }
    }
}

/// Synchronously loads PDF file content using pdf-extract crate
///
/// # Arguments
/// * `file_path` - Path to the PDF file
///
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - PDF content if successful, error otherwise
fn load_pdf_file_sync(file_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    // Extract text from PDF using pdf-extract crate
    // Use std::panic::catch_unwind to catch any panics from pdf-extract
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        extract_text(file_path)
    }));

    match result {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(e)) => {
            eprintln!("Warning: Failed to extract text from PDF file '{:?}': {}", file_path, e);
            // Return empty string instead of erroring
            Ok(String::new())
        },
        Err(panic_err) => {
            eprintln!("Warning: PDF extraction panicked for file '{:?}': {:?}", file_path, panic_err);
            // Return empty string instead of panicking
            Ok(String::new())
        }
    }
}