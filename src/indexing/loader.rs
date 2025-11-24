//! File loading module for reading document content.
//!
//! This module provides functionality to load content from various file types
//! including text files, PDFs, and DOCX documents. It handles both synchronous
//! and asynchronous file reading operations.
//!
//! For PDF files, this module uses the pdf-extract crate to extract text content
//! from PDF documents. The implementation includes robust error handling to
//! prevent panics when processing large or problematic PDF files.
//!
//! For DOCX files, this module uses the docx-rust crate to extract text content
//! from DOCX documents.

use std::fs;
use std::path::Path;
use crate::Config;
use crate::AppError;
use pdf_extract::extract_text;
use docx_rust::DocxFile;
use tracing::warn;

/// Trait for loading document content from different file types
pub trait DocumentLoader {
    /// Loads the document content
    ///
    /// # Returns
    /// * `Result<String, AppError>` - Document content if successful, error otherwise
    fn load(&self, path: &Path) -> Result<String, AppError>;
}

/// Loader for plain text files
pub struct TextLoader;

impl DocumentLoader for TextLoader {
    fn load(&self, path: &Path) -> Result<String, AppError> {
        fs::read_to_string(path).map_err(AppError::Io)
    }
}

/// Loader for PDF files
pub struct PdfLoader;

impl DocumentLoader for PdfLoader {
    fn load(&self, path: &Path) -> Result<String, AppError> {
        // Use catch_unwind to prevent panics from pdf-extract
        let result = std::panic::catch_unwind(|| {
            extract_text(path)
        });

        match result {
            Ok(Ok(content)) => Ok(content),
            Ok(Err(e)) => {
                warn!("Failed to extract PDF content: {}", e);
                Err(AppError::Pdf(format!("PDF extraction failed: {}", e)))
            }
            Err(_) => {
                warn!("PDF extraction panicked");
                Err(AppError::Pdf("PDF extraction panicked".to_string()))
            }
        }
    }
}

/// Loader for DOCX files
pub struct DocxLoader;

impl DocumentLoader for DocxLoader {
    fn load(&self, path: &Path) -> Result<String, AppError> {
        let docx = DocxFile::from_file(path)
            .map_err(|e| AppError::Docx(format!("Failed to open DOCX file: {}", e)))?;

        let docx = docx.parse()
            .map_err(|e| AppError::Docx(format!("Failed to parse DOCX file: {}", e)))?;

        let mut content = String::new();
        for paragraph in docx.document.body.content {
            if let docx_rust::document::BodyContent::Paragraph(p) = paragraph {
                for run in p.content {
                    if let docx_rust::document::ParagraphContent::Run(r) = run {
                        for text in r.content {
                            if let docx_rust::document::RunContent::Text(t) = text {
                                content.push_str(&t.text);
                            }
                        }
                    }
                }
                content.push('\n');
            }
        }

        Ok(content)
    }
}

/// Returns the appropriate loader for a given file extension
fn get_loader(extension: &str) -> Box<dyn DocumentLoader> {
    match extension {
        "pdf" => Box::new(PdfLoader),
        "docx" => Box::new(DocxLoader),
        _ => Box::new(TextLoader),
    }
}

/// Asynchronously loads file content from disk
///
/// # Arguments
/// * `config` - Configuration object containing data sources path
/// * `filename` - Name of the file to load
///
/// # Returns
/// * `Result<String, AppError>` - File content if successful, error otherwise
pub async fn load_file(config: &Config, filename: &str) -> Result<String, AppError> {
    let file_path = Path::new(&config.data_sources.path).join(filename);

    // Get the appropriate loader based on file extension
    let extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    let loader = get_loader(extension);
    
    // Load the file content
    match loader.load(&file_path) {
        Ok(content) => Ok(content),
        Err(e) => {
            warn!("Failed to load file '{}': {}", filename, e);
            // Return empty string for failed loads to allow processing to continue
            Ok(String::new())
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
/// * `Result<String, AppError>` - File content if successful, error otherwise
pub fn load_file_sync(config: &Config, filename: &str) -> Result<String, AppError> {
    let file_path = Path::new(&config.data_sources.path).join(filename);

    // Get the appropriate loader based on file extension
    let extension = file_path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    
    let loader = get_loader(extension);
    
    // Load the file content
    match loader.load(&file_path) {
        Ok(content) => Ok(content),
        Err(e) => {
            warn!("Failed to load file '{}': {}", filename, e);
            // Return empty string for failed loads to allow processing to continue
            Ok(String::new())
        }
    }
}
