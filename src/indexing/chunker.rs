// Découpage (text-splitter-rs ou logique manuelle)

pub fn chunk_text(text: &str, chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    
    // Simple chunking by splitting on newlines and then by word boundaries
    let lines: Vec<&str> = text.lines().collect();
    
    let mut current_chunk = String::new();
    
    for line in lines {
        if current_chunk.len() + line.len() + 1 > chunk_size && !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
            current_chunk = String::new();
        }
        
        if !current_chunk.is_empty() {
            current_chunk.push('\n');
        }
        current_chunk.push_str(line);
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }
    
    chunks
}