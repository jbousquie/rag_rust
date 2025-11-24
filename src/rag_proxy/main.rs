use rag_rust::rag_proxy::server::start_server;
use std::env;

use rag_rust::init_logging;
use tracing::info;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    // Check for passthrough argument
    let args: Vec<String> = env::args().collect();
    let passthrough_mode = args.iter().any(|arg| arg == "--passthrough");

    if passthrough_mode {
        info!("Starting RAG proxy server in passthrough mode...");
    } else {
        info!("Starting RAG proxy server in RAG mode...");
    }

    // Start the server with the appropriate mode
    start_server().await?;

    info!("RAG proxy server started successfully!");
    Ok(())
}
