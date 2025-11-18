#!/bin/bash

# Test script for rag_proxy

echo "Testing rag_proxy with a sample question..."

# Test with a simple question
curl -X POST http://127.0.0.1:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "qwen3coder",
    "messages": [
      {
        "role": "user",
        "content": "Le MiniLab MkII connecté en USB sur son ordinateur peut-il piloter un DAW ?"
      }
    ],
    "stream": false
  }'

echo ""
echo "Test completed."
