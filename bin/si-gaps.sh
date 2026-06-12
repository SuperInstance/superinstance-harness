#!/bin/bash
# Find ecosystem gaps for a given domain
DOMAIN="${1:-distributed systems}"
echo "=== Gap Analysis: $DOMAIN ==="
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/gap-analysis" \
  -H "Content-Type: application/json" \
  -d "{\"domain\": \"$DOMAIN\"}"
echo ""
echo "=== Similar crates ==="
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/search" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"$DOMAIN implementation\", \"topK\": 5}"
