#!/bin/bash
# Pre-task search — run before starting any development work
# Usage: pre-task-search.sh "task description"

TASK="$1"
if [ -z "$TASK" ]; then
    echo "Usage: pre-task-search.sh 'task description'"
    exit 1
fi

echo "=== Pre-Task Search: $TASK ==="

echo -e "\n1. Existing similar crates:"
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/search" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"$TASK\", \"topK\": 5}"

echo -e "\n2. Build patterns:"
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/search" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"build pattern $TASK\", \"topK\": 3}"

echo -e "\n3. Recommendations:"
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/recommend" \
  -H "Content-Type: application/json" \
  -d "{\"context\": \"$TASK\", \"topK\": 3}"
