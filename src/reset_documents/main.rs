use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use toml;

use rag_rust::qdrant_custom_client::QdrantClient;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    #[serde(rename = "qdrant")]
    qdrant_config: QdrantConfig,

    #[serde(rename = "indexing")]
    indexing_config: IndexingConfig,
}

#[derive(Debug, Deserialize, Serialize)]
struct QdrantConfig {
    host: String,
    port: u16,
    api_key: String,
    collection: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct IndexingConfig {
    file_tracker_path: String,
}

use rag_rust::init_logging;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    init_logging();

    // Read configuration from config.toml
    let config_content = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_content)?;

    // Create Qdrant client from config
    let qdrant_client = QdrantClient::new(
        config.qdrant_config.host,
        config.qdrant_config.port,
        config.qdrant_config.api_key,
        0,             // vector_size not needed for deletion
        String::new(), // distance not needed for deletion
        0,             // limit not needed for deletion
        0.0,           // score_threshold not needed for deletion
    );

    // Delete the Qdrant collection
    info!(
        "Deleting Qdrant collection: {}",
        config.qdrant_config.collection
    );
    match qdrant_client
        .delete_collection(&config.qdrant_config.collection)
        .await
    {
        Ok(success) => {
            if success {
                info!(
                    "Successfully deleted collection: {}",
                    config.qdrant_config.collection
                );
            } else {
                error!(
                    "Failed to delete collection: {}",
                    config.qdrant_config.collection
                );
            }
        }
        Err(e) => {
            error!("Error deleting collection: {}", e);
        }
    }

    // Delete the content of the tracker file
    let tracker_path = Path::new(&config.indexing_config.file_tracker_path);
    if tracker_path.exists() {
        info!("Clearing tracker file: {:?}", tracker_path);
        fs::write(tracker_path, "{}")?;
        info!("Successfully cleared tracker file: {:?}", tracker_path);
    } else {
        info!(
            "Tracker file does not exist, creating empty one: {:?}",
            tracker_path
        );
        fs::write(tracker_path, "{}")?;
        info!(
            "Successfully created empty tracker file: {:?}",
            tracker_path
        );
    }

    Ok(())
}
