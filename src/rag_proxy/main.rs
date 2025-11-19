use rag_rust::rag_proxy::server::start_server;
use std::env;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for passthrough argument
    let args: Vec<String> = env::args().collect();
    let passthrough_mode = args.iter().any(|arg| arg == "--passthrough");

    if passthrough_mode {
        println!("Starting RAG proxy server in passthrough mode...");
    } else {
        println!("Starting RAG proxy server in RAG mode...");
    }

    // Start the server with the appropriate mode
    start_server().await?;

    println!("RAG proxy server started successfully!");
    Ok(())
}
