#!/bin/bash

# Test script to check proxy behavior with different request formats

echo "=== Testing with standard OpenAI format ==="
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "What is this project about?"
      }
    ],
    "stream": true
  }' \
  -v

echo -e "\n\n=== Testing with minimal request (no stream field) ==="
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "What is this project about?"
      }
    ]
  }' \
  -v

echo -e "\n\n=== Testing with stream=true ==="
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "What is this project about?"
      }
    ],
    "stream": true
  }' \
  -v

echo -e "\n\n=== Testing with stream=false ==="
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "What is this project about?"
      }
    ],
    "stream": false
  }' \
  -v

echo -e "\n\n=== Testing with different content format ==="
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "What is this project about?"
      }
    ],
    "stream": false,
    "temperature": 0.7
  }' \
  -v
