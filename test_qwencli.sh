#!/bin/bash

# Test script to verify QwenCLI compatibility with our RAG proxy

echo "Testing RAG proxy with QwenCLI compatibility..."

# Start the RAG proxy in normal mode
echo "Starting RAG proxy server..."
cargo run --bin rag_proxy &

# Give the server time to start
sleep 3

# Test with a simple question that should work with QwenCLI
echo "Testing with a simple question..."
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "What is Rust programming language?"
      }
    ],
    "stream": false
  }' | jq .

# Test with a more complex question that includes a system message
echo "Testing with a question that includes a system message..."
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "system",
        "content": "You are a helpful assistant."
      },
      {
        "role": "user",
        "content": "Explain Rust memory management?"
      }
    ],
    "stream": false
  }' | jq .

# Stop the server
kill $(jobs -p)

echo "Test completed."