// Chargement des fichiers (tokio::fs, pdf_extract, docx-rs)

use std::fs;
use std::path::Path;
use crate::Config;

pub async fn load_file(config: &Config, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = Path::new(&config.data_sources.path).join(filename);
    let content = tokio::fs::read_to_string(&file_path).await?;
    Ok(content)
}

pub fn load_file_sync(config: &Config, filename: &str) -> Result<String, Box<dyn std::error::Error>> {
    let file_path = Path::new(&config.data_sources.path).join(filename);
    let content = fs::read_to_string(&file_path)?;
    Ok(content)
}
