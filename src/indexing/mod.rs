//! Indexing module for processing documents and generating embeddings.
//!
//! This module contains all the functionality needed to process documents
//! from the data sources directory, split them into chunks, generate embeddings,
//! and store them in the Qdrant vector database. It also handles file tracking
//! to avoid re-indexing unchanged files.

pub mod loader;
pub mod chunker;
pub mod indexer;
pub mod file_tracker;
