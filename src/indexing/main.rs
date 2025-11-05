use crate::common::Config;
use std::path::Path;

pub async fn main() {
    println!("Starting document indexing...");

    // Load configuration
    let config = Config::load().expect("Failed to load configuration");

    // Get data sources path
    let data_sources_path = Path::new(&config.data_sources.path);

    // Check if data_sources directory exists
    if !data_sources_path.exists() {
        eprintln!(
            "Data sources directory does not exist: {:?}",
            data_sources_path
        );
        return;
    }

    println!("Document indexing completed.");
}
