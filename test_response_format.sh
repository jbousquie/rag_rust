#!/bin/bash

# Script to test response format when using RAG proxy

set -e  # Exit on any error

echo "Testing RAG proxy response format..."

# Start the RAG proxy in the background
echo "Starting RAG proxy in passthrough mode..."
cargo run --bin rag_proxy -- --passthrough &
SERVER_PID=$!
sleep 5  # Give the server time to start

# Test 1: Send a request with stream=false and check response format
echo "Test 1: Request with stream=false"
RESPONSE=$(curl -s -w "\n%{content_type}\n" -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "Hello, how are you?"
      }
    ],
    "stream": false
  }')

echo "Response with content type:"
echo "$RESPONSE"
echo ""

# Test 2: Send a request without stream parameter (should default to false)
echo "Test 2: Request without stream parameter"
RESPONSE2=$(curl -s -w "\n%{content_type}\n" -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "What is the capital of France?"
      }
    ]
  }')

echo "Response with content type:"
echo "$RESPONSE2"
echo ""

# Cleanup: Kill the server process
kill $SERVER_PID || true
wait $SERVER_PID 2>/dev/null || true

echo "All tests completed!"