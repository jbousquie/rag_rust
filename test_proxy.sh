#!/bin/bash

# Test script to check proxy behavior with different clients

echo "Testing proxy with curl (should work)..."
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

echo -e "\n\nTesting proxy with a simple HTTP client (should work)..."
python3 -c "
import requests
import json

url = 'http://127.0.0.1:3000/v1/chat/completions'
headers = {'Content-Type': 'application/json'}
data = {
    'model': 'qwen3-coder-dual',
    'messages': [
        {
            'role': 'user',
            'content': 'What is this project about?'
        }
    ],
    'stream': False
}

response = requests.post(url, headers=headers, json=data, timeout=30)
print('Status Code:', response.status_code)
print('Response:', response.text[:200] + '...' if len(response.text) > 200 else response.text)
"