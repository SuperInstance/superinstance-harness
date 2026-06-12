#!/bin/bash
# SuperInstance Agent Bootstrap
# Run this at the start of any agent session to get oriented.
#
# Usage: bash /home/phoenix/repos/superinstance-harness/bin/bootstrap.sh [OPTIONAL_QUERY]
#
# What it does:
#   1. Shows current harness allocation (γ exploitation vs η exploration)
#   2. Searches knowledge base for relevant patterns
#   3. Shows ecosystem stats (total crates, index info)
#   4. Shows harness performance metrics
#   5. Counts empty crates remaining

set -euo pipefail

QUERY="${1:-build fix success pattern recent}"

HARNESS="https://harness-api.casey-digennaro.workers.dev"
VECTOR="https://fleet-vector-api.casey-digennaro.workers.dev"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║          SuperInstance Agent Bootstrap                  ║"
echo "║          Ecosystem Orientation & Knowledge Load         ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

echo "=== 📊 Work Allocation (γ exploit / η explore) ==="
curl -sf "$HARNESS/allocation" | python3 -m json.tool 2>/dev/null || echo "(harness unreachable)"
echo ""

echo "=== 🧠 Knowledge Base: Relevant Patterns ==="
echo "Query: $QUERY"
curl -sf -X POST "$VECTOR/search" \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"$QUERY\", \"topK\": 5}" | python3 -c "
import sys, json
try:
    data = json.load(sys.stdin)
    for r in data.get('results', [])[:5]:
        score = r.get('score', 0)
        name = r.get('name', '?')
        desc = r.get('description', '')[:120]
        print(f'  [{score:.3f}] {name}')
        print(f'         {desc}')
        print()
except:
    print('  (search failed)')
" 2>/dev/null || echo "(vector search unreachable)"
echo ""

echo "=== 📈 Ecosystem Stats ==="
curl -sf "$VECTOR/stats" | python3 -m json.tool 2>/dev/null || echo "(vector API unreachable)"
echo ""

echo "=== ⚡ Harness Performance Metrics ==="
curl -sf "$HARNESS/metrics" | python3 -m json.tool 2>/dev/null || echo "(harness unreachable)"
echo ""

echo "=== 📦 Empty Crates Remaining ==="
if [ -f /tmp/empty_librs_results.txt ]; then
    EMPTY=$(grep -c "EMPTY_LIBRS\|MISSING_LIBRS\|MISSING_SRC" /tmp/empty_librs_results.txt 2>/dev/null || echo "?")
    echo "  $EMPTY empty/missing crates registered in /tmp/empty_librs_results.txt"
else
    echo "  (no empty crate registry found - run a scan first)"
fi
echo ""

echo "=== 📋 Quick Reference ==="
echo "  Harness API:     $HARNESS"
echo "  Vector API:      $VECTOR"
echo "  Build Waves:     $(ls /home/phoenix/repos/RUST-BUILD-WAVE*.md 2>/dev/null | wc -l) reports"
echo "  Total Repos:     $(ls /home/phoenix/repos/ 2>/dev/null | wc -l)"
echo "  Knowledge Base:  25 patterns indexed"
echo ""
echo "Key endpoints:"
echo "  POST $HARNESS/cycle      — Record a work cycle"
echo "  POST $VECTOR/ingest       — Add new patterns/crates"
echo "  POST $VECTOR/search       — Semantic search"
echo "  POST $VECTOR/recommend    — Get crate recommendations"
echo "  GET  $VECTOR/gap-analysis — Find ecosystem gaps"
echo ""
echo "Bootstrap complete. You are oriented. 🚀"
