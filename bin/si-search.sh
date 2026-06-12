#!/bin/bash
# SuperInstance semantic search — use this for any development task
# Usage: si-search.sh "query" [topK]

QUERY="${1:-ternary signal processing}"
TOPK="${2:-10}"

echo "=== SuperInstance Semantic Search ==="
echo "Query: $QUERY"
echo "Top: $TOPK"
echo ""

curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/search" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"$QUERY\", \"topK\": $TOPK}" | python3 -c "
import sys, json
data = json.load(sys.stdin)
if 'results' in data:
    for i, r in enumerate(data['results'], 1):
        name = r.get('id', r.get('name', 'unknown'))
        score = r.get('score', 0)
        desc = r.get('metadata', {}).get('description', '(no description)')
        print(f'{i}. {name} (score: {score:.3f})')
        if desc:
            print(f'   {desc[:120]}')
        print()
else:
    print(json.dumps(data, indent=2))
"
