#!/bin/bash

# Test script for rag_proxy

echo "Testing rag_proxy with a sample question..."

# Test with a simple question (original format)
echo "Testing with simple format:"
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": "Que peux-tu me dire à propos de Zorglub ?"
      }
    ],
    "stream": false
  }'

echo ""
echo "Testing with multimodal format:"
# Test with multimodal question (Qwen CLI format)
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3-coder-dual",
    "messages": [
      {
        "role": "user",
        "content": [
          {
            "type": "text",
            "text": "Que peux-tu me dire à propos de Zorglub ?"
          }
        ]
      }
    ],
    "stream": false
  }'

echo ""
echo "Test completed."
