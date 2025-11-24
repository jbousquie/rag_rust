//! RAG Proxy Module
//!
//! This module contains all the components necessary to implement the RAG (Retrieval-Augmented Generation) proxy server.
//! It handles incoming HTTP requests, processes them through the RAG pipeline (retrieval + LLM calling),
//! and returns responses in OpenAI API compatible format.

pub mod handler;
pub mod passthrough_handler;
pub mod retriever;
pub mod server;
