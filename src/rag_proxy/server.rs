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
use std::env;
use std::sync::Arc;

use crate::load_config;
use crate::AppError;
use crate::rag_proxy::handler::handle_rag_request;
use crate::rag_proxy::passthrough_handler::handle_passthrough_request;
use tokio::net::TcpListener;

/// Starts the RAG proxy server
///
/// This function initializes and starts the Axum web server that listens for
/// incoming requests on the configured host and port.
///
/// # Returns
/// * `Result<(), AppError>` - Success or error
pub async fn start_server() -> Result<(), AppError> {
    // Check for passthrough argument from environment
    let args: Vec<String> = env::args().collect();
    let passthrough_mode = args.iter().any(|arg| arg == "--passthrough");

    // Load configuration from config.toml
    let config = Arc::new(load_config()?);

    // Build the application with routes
    let app = Router::new();

    let app = if passthrough_mode {
        // Use passthrough handler
        app.route(
            &config.rag_proxy.chat_completion_endpoint,
            post(handle_passthrough_request),
        )
    } else {
        // Use RAG handler
        app.route(
            &config.rag_proxy.chat_completion_endpoint,
            post(handle_rag_request),
        )
    };
    
    let app = app.with_state(config.clone());

    // Add health check endpoint to the app
    let app = app.route("/health", get(health_check));

    // Create the socket address from configuration
    let addr = SocketAddr::from((
        config.rag_proxy.host.parse::<std::net::IpAddr>().map_err(|e| AppError::Config(format!("Invalid IP address: {}", e)))?,
        config.rag_proxy.port,
    ));

    println!("RAG proxy server starting on http://{}", addr);

    // Start the server with the configured address
    let listener = TcpListener::bind(addr).await.map_err(AppError::Io)?;
    axum::serve(listener, app).await.map_err(AppError::Io)?;

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
