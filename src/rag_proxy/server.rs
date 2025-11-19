//! RAG Proxy Server Module
//!
//! This module implements the HTTP server for the RAG proxy that handles
//! incoming requests and routes them to the appropriate handlers.

use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use std::net::SocketAddr;

use crate::load_config;
use crate::rag_proxy::handler::handle_rag_request;
use tokio::net::TcpListener;

/// Starts the RAG proxy server
///
/// This function initializes and starts the Axum web server that listens for
/// incoming requests on the configured host and port.
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration from config.toml
    let config = load_config();

    // Build the application with routes
    let app = Router::new()
        // Route for chat completions (OpenAI API compatible endpoint)
        .route(
            &config.rag_proxy.chat_completion_endpoint,
            post(handle_rag_request),
        )
        // Health check endpoint
        .route("/health", get(health_check));

    // Create the socket address from configuration
    let addr = SocketAddr::from((
        config.rag_proxy.host.parse::<std::net::IpAddr>()?,
        config.rag_proxy.port,
    ));

    println!("RAG proxy server starting on http://{}", addr);

    // Start the server with the configured address
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint handler
///
/// This function handles requests to the /health endpoint and returns a 200 OK status.
///
/// # Returns
/// * `impl IntoResponse` - HTTP response with status code 200
async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
