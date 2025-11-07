use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct FileTracker {
    files: HashMap<String, String>, // filename -> md5
}

impl FileTracker {
    pub fn new() -> Self {
        FileTracker {
            files: HashMap::new(),
        }
    }

    pub fn load_from_file(&mut self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if Path::new(file_path).exists() {
            let content = fs::read_to_string(file_path)?;
            if !content.trim().is_empty() {
                let tracker: FileTracker = serde_json::from_str(&content)?;
                self.files = tracker.files;
            }
        }
        Ok(())
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(file_path)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub fn get_file_md5(&self, filename: &str) -> Option<&String> {
        self.files.get(filename)
    }

    pub fn set_file_md5(&mut self, filename: String, md5: String) {
        self.files.insert(filename, md5);
    }

    pub fn remove_file(&mut self, filename: &str) {
        self.files.remove(filename);
    }

    pub fn is_file_changed(&self, filename: &str, content: &[u8]) -> bool {
        let current_md5 = format!("{:x}", Md5::digest(content));
        match self.files.get(filename) {
            Some(stored_md5) => stored_md5 != &current_md5,
            None => true, // File not tracked yet
        }
    }

    pub fn get_changed_files(&self, files: &[String]) -> Vec<String> {
        files
            .iter()
            .filter(|filename| {
                // Check if file exists and has changed
                if let Ok(content) = fs::read(filename) {
                    self.is_file_changed(filename, &content)
                } else {
                    // If file doesn't exist, it's considered as changed
                    true
                }
            })
            .cloned()
            .collect()
    }
    
    // Method to get the file tracker path from configuration
    pub fn get_tracker_path(config: &Config) -> String {
        config.indexing.file_tracker_path.clone()
    }
}