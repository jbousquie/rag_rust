//! File tracker module for tracking indexed files and their MD5 checksums.
//!
//! This module provides functionality to track which files have been indexed,
//! and to determine if files have changed since their last indexing.
//! It uses a JSON file to persist the tracking information between runs.

use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use crate::Config;

/// File tracker structure for managing file indexing status
#[derive(Debug, Serialize, Deserialize)]
pub struct FileTracker {
    files: HashMap<String, String>, // filename -> md5
}

impl FileTracker {
    /// Creates a new empty file tracker
    pub fn new() -> Self {
        FileTracker {
            files: HashMap::new(),
        }
    }

    /// Loads tracking information from a JSON file
    /// 
    /// # Arguments
    /// * `file_path` - Path to the JSON file containing tracking information
    /// 
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if successful, error otherwise
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

    /// Saves tracking information to a JSON file
    /// 
    /// # Arguments
    /// * `file_path` - Path to the JSON file where tracking information will be saved
    /// 
    /// # Returns
    /// * `Result<(), Box<dyn std::error::Error>>` - Ok if successful, error otherwise
    pub fn save_to_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        let mut file = fs::File::create(file_path)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }

    /// Gets the MD5 checksum for a specific file
    /// 
    /// # Arguments
    /// * `filename` - Name of the file to look up
    /// 
    /// # Returns
    /// * `Option<&String>` - The MD5 checksum if the file is tracked, None otherwise
    pub fn get_file_md5(&self, filename: &str) -> Option<&String> {
        self.files.get(filename)
    }

    /// Sets the MD5 checksum for a specific file
    /// 
    /// # Arguments
    /// * `filename` - Name of the file to set
    /// * `md5` - MD5 checksum to associate with the file
    pub fn set_file_md5(&mut self, filename: String, md5: String) {
        self.files.insert(filename, md5);
    }

    /// Removes a file from tracking
    /// 
    /// # Arguments
    /// * `filename` - Name of the file to remove from tracking
    pub fn remove_file(&mut self, filename: &str) {
        self.files.remove(filename);
    }

    /// Checks if a file has changed since it was last tracked
    /// 
    /// # Arguments
    /// * `filename` - Name of the file to check
    /// * `content` - Current content of the file
    /// 
    /// # Returns
    /// * `bool` - True if the file has changed, false if it's the same or not tracked
    pub fn is_file_changed(&self, filename: &str, content: &[u8]) -> bool {
        let current_md5 = format!("{:x}", Md5::digest(content));
        match self.files.get(filename) {
            Some(stored_md5) => stored_md5 != &current_md5,
            None => true, // File not tracked yet
        }
    }

    /// Gets a list of files that have changed since last tracking
    /// 
    /// # Arguments
    /// * `files` - List of file names to check
    /// 
    /// # Returns
    /// * `Vec<String>` - List of file names that have changed
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
    
    /// Gets the file tracker path from configuration
    /// 
    /// # Arguments
    /// * `config` - Configuration object containing the file tracker path
    /// 
    /// # Returns
    /// * `String` - The path to the file tracker file
    pub fn get_tracker_path(config: &Config) -> String {
        config.indexing.file_tracker_path.clone()
    }
}