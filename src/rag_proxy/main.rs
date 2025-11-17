use rag_rust::rag_proxy::server::start_server;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting RAG proxy server...");

    // Start the server
    start_server().await?;

    println!("RAG proxy server started successfully!");
    Ok(())
}
