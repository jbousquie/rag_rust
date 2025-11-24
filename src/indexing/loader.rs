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
        Some("docx") => {
            // Load DOCX content using docx-rust crate
            match load_docx_file(&file_path).await {
                Ok(content) => Ok(content),
                Err(e) => {
                    eprintln!("Warning: Failed to read DOCX file '{}': {}", filename, e);
                    // Return empty string instead of panicking
                    Ok(String::new())
                }
            }
        }
        _ => {
            // Default to text file loading
            let content = tokio::fs::read_to_string(&file_path).await.map_err(AppError::Io)?;
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
/// * `Result<String, AppError>` - File content if successful, error otherwise
pub fn load_file_sync(config: &Config, filename: &str) -> Result<String, AppError> {
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
        Some("docx") => {
            // Load DOCX content using docx-rust crate
            match load_docx_file_sync(&file_path) {
                Ok(content) => Ok(content),
                Err(e) => {
                    eprintln!("Warning: Failed to read DOCX file '{}': {}", filename, e);
                    // Return empty string instead of panicking
                    Ok(String::new())
                }
            }
        }
        _ => {
            // Default to text file loading
            let content = fs::read_to_string(&file_path).map_err(AppError::Io)?;
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
/// * `Result<String, AppError>` - PDF content if successful, error otherwise
async fn load_pdf_file(file_path: &Path) -> Result<String, AppError> {
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

/// Asynchronously loads DOCX file content using docx-rust crate
///
/// # Arguments
/// * `file_path` - Path to the DOCX file
///
/// # Returns
/// * `Result<String, AppError>` - DOCX content if successful, error otherwise
async fn load_docx_file(file_path: &Path) -> Result<String, AppError> {
    // Extract text from DOCX using docx-rust crate
    // Use spawn_blocking since docx-rust is synchronous
    let path = file_path.to_path_buf();
    let path_clone = path.clone();  // Clone path to use in error handling outside the closure
    let result = tokio::task::spawn_blocking(move || -> Result<String, AppError> {
        // Load DOCX file using docx-rust API and extract text
        match DocxFile::from_file(&path) {
            Ok(docx_file) => {
                match docx_file.parse() {
                    Ok(docx) => {
                        // Build the content string by extracting text from paragraphs
                        let mut content = String::new();

                        // Access the content of the body
                        for content_child in &docx.document.body.content {
                            if let docx_rust::document::BodyContent::Paragraph(p) = content_child {
                                let mut paragraph_content = String::new();
                                for run_child in &p.content {
                                    if let docx_rust::document::ParagraphContent::Run(run) = run_child {
                                        // Access the text property - it's likely a direct field or method
                                        let run_text = run.text();
                                        paragraph_content.push_str(&run_text);
                                    }
                                }
                                content.push_str(&paragraph_content);
                                content.push('\n');
                            }
                        }

                        Ok(content)
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse DOCX file '{:?}': {:?}", path, e);
                        // Return empty string instead of erroring to allow processing to continue
                        Ok(String::new())
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to load DOCX file '{:?}': {:?}", path, e);
                Ok(String::new())
            }
        }
    })
    .await;

    match result {
        Ok(content_result) => content_result,
        Err(join_err) => {
            eprintln!("Warning: Task join error for DOCX file '{:?}': {:?}", path_clone, join_err);
            Err(AppError::Docx(format!("Join error: {:?}", join_err)))
        }
    }
}

/// Synchronously loads PDF file content using pdf-extract crate
///
/// # Arguments
/// * `file_path` - Path to the PDF file
///
/// # Returns
/// * `Result<String, AppError>` - PDF content if successful, error otherwise
fn load_pdf_file_sync(file_path: &Path) -> Result<String, AppError> {
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

/// Synchronously loads DOCX file content using docx-rust crate
///
/// # Arguments
/// * `file_path` - Path to the DOCX file
///
/// # Returns
/// * `Result<String, AppError>` - DOCX content if successful, error otherwise
fn load_docx_file_sync(file_path: &Path) -> Result<String, AppError> {
    // Load DOCX file using docx-rust API and extract text
    match DocxFile::from_file(file_path) {
        Ok(docx_file) => {
            match docx_file.parse() {
                Ok(docx) => {
                    // Build the content string by extracting text from paragraphs
                    let mut content = String::new();

                    // Access the content of the body
                    for content_child in &docx.document.body.content {
                        if let docx_rust::document::BodyContent::Paragraph(p) = content_child {
                            let mut paragraph_content = String::new();
                            for run_child in &p.content {
                                if let docx_rust::document::ParagraphContent::Run(run) = run_child {
                                    // Access the text property - it's likely a direct field or method
                                    let run_text = run.text();
                                    paragraph_content.push_str(&run_text);
                                }
                            }
                            content.push_str(&paragraph_content);
                            content.push('\n');
                        }
                    }

                    Ok(content)
                }
                Err(e) => {
                    eprintln!("Warning: Failed to parse DOCX file '{:?}': {:?}", file_path, e);
                    // Return empty string instead of erroring to allow processing to continue
                    Ok(String::new())
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to load DOCX file '{:?}': {:?}", file_path, e);
            Ok(String::new())
        }
    }
}