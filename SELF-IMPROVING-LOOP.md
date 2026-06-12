# Self-Improving Loop — Compound Knowledge Architecture

**Created:** 2026-06-11  
**Status:** Active  
**Cycle:** 55 harness cycles, 437 build waves, 1539 vectorized entries

---

## The Compound Loop

Every agent run should be smarter than the last. This document describes the infrastructure that makes that possible.

```
┌─────────────────────────────────────────────────────────────────┐
│                     COMPOUND LEARNING CYCLE                     │
│                                                                 │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────────────┐  │
│  │  Agent    │───▶│  Build Waves │───▶│  Pattern Extraction  │  │
│  │  Session  │    │  (437 done)  │    │  (25 patterns)       │  │
│  └────▲──────┘    └──────────────┘    └──────────┬───────────┘  │
│       │                                           │              │
│       │              ┌──────────────┐             │              │
│       │              │  Vector Store│◀────────────┘              │
│       │              │  (1539 vecs) │                            │
│       │              └──────┬───────┘                            │
│       │                     │                                    │
│       └─────────────────────┘                                    │
│         Bootstrap Query: agent searches patterns                 │
│         before starting work                                     │
└─────────────────────────────────────────────────────────────────┘
```

---

## Current State

### What We Have

| Component | Endpoint | Data |
|-----------|----------|------|
| **Harness** | `harness-api.casey-digennaro.workers.dev` | 55 cycles, γ=0.90, EWMA quality=0.71 |
| **Vector Store** | `fleet-vector-api.casey-digennaro.workers.dev` | 1539 entries (1514 crates + 25 patterns) |
| **Build Waves** | `/home/phoenix/repos/RUST-BUILD-WAVE*.md` | 437 reports, 2896 PASS, 52 FAIL |
| **Empty Crate Registry** | `/tmp/empty_librs_results.txt` | 228 empty/missing crates |
| **Field Reports** | `HARNESS-FIELD-REPORT.md` | Cycle-by-cycle quality records |

### What the Knowledge Base Contains

25 extracted patterns covering:
- **Build failures** (6 patterns): manifest parse, private fields, unresolved imports, method not found, borrow checker, format spec changes
- **Success patterns** (3 patterns): fresh scaffold success, high pass rate, typical wave output
- **Development patterns** (3 patterns): vector search reference, split mutable borrow, missing imports
- **Harness patterns** (3 patterns): gamma overcorrection, quality inversion fix, stress test regime
- **Infrastructure patterns** (2 patterns): rate limiting, cache control
- **Quality patterns** (2 patterns): empty lib.rs crisis, workspace complexity
- **Metrics** (3 patterns): ecosystem scale, pass rates, productivity benchmarks
- **Other** (3 patterns): edition 2024 format, graphql syntax, BinaryHeap trait bounds

---

## How Future Agents Should Query

### Before Starting Work

```bash
# 1. Get current allocation
curl -s "https://harness-api.casey-digennaro.workers.dev/allocation"

# 2. Search for relevant patterns BEFORE writing code
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "build fix pattern for <YOUR_TOPIC>", "topK": 5}'

# 3. Get ecosystem stats to understand what exists
curl -s "https://fleet-vector-api.casey-digennaro.workers.dev/stats"
```

### After Completing Work

```bash
# 1. Record the cycle in the harness
curl -s -X POST "https://harness-api.casey-digennaro.workers.dev/cycle" \
  -H "Content-Type: application/json" \
  -d '{"gamma_spent": 0.7, "eta_spent": 0.3, "output_quality": 0.85, "output_quantity": 1704, "exploration_yield": 0.6}'

# 2. Ingest new patterns discovered during the session
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/ingest" \
  -H "Content-Type: application/json" \
  -d '{"crates": [{"name": "pattern-<descriptive-id>", "description": "What you learned", "readme": "Full pattern description with context and fix"}]}'
```

### Key Queries for Common Scenarios

| Scenario | Query |
|----------|-------|
| Build failing | `"build fix error E0433 E0599 E0616"` |
| Empty crate to fill | `"empty lib.rs populate implementation"` |
| Cargo.toml issues | `"manifest parse failure Cargo.toml"` |
| Publish issues | `"crates.io rate limit 429 publish"` |
| Borrow checker | `"borrow checker mutable reference split"` |
| Best practices | `"build success pattern fresh scaffold"` |

---

## Metrics for Measuring Improvement

### Per-Cycle Metrics

| Metric | Source | Target | Current |
|--------|--------|--------|---------|
| Build pass rate | Wave reports | >98% | 98.2% |
| Average quality score | Harness | >0.80 | 0.71 |
| Empty crates remaining | empty_librs scan | <50 | 228 |
| Patterns in knowledge base | Vector store | >50 | 25 |
| Harness cycles | Harness API | Growing | 55 |

### Compound Metrics (Track Over Time)

1. **Pattern Hit Rate**: How often a search returns a relevant pattern that prevents a known failure. Track: successes where agent says "pattern X from knowledge base helped."

2. **First-Attempt Build Rate**: Percentage of crates that pass cargo check on first attempt. Should increase as patterns are applied upstream.

3. **Quality Trend**: EWMA quality over time. Should increase as agents learn from past mistakes.

4. **Time-to-Productive**: Seconds from agent start to first useful output. Should decrease as bootstrap queries provide better orientation.

5. **Failure Novelty Rate**: Percentage of build failures that are NEW (not matching any known pattern). Should decrease over time as patterns are comprehensive.

---

## The Full Cycle Diagram

```
                    ┌─────────────────────────┐
                    │   NEW AGENT SESSION      │
                    │   (fresh context)         │
                    └───────────┬──────────────┘
                                │
                    ┌───────────▼──────────────┐
                    │   BOOTSTRAP QUERY         │
                    │   - Get allocation (γ/η)  │
                    │   - Search patterns       │
                    │   - Check stats           │
                    └───────────┬──────────────┘
                                │
                    ┌───────────▼──────────────┐
                    │   ORIENTED WORK           │
                    │   - Apply known patterns  │
                    │   - Avoid known failures  │
                    │   - Use vector refs       │
                    └───────────┬──────────────┘
                                │
                    ┌───────────▼──────────────┐
                    │   BUILD + TEST            │
                    │   - cargo check           │
                    │   - cargo test            │
                    │   - Record results        │
                    └───────────┬──────────────┘
                                │
                    ┌───────────▼──────────────┐
                    │   EXTRACT PATTERNS        │
                    │   - What worked?          │
                    │   - What failed?          │
                    │   - New error types?      │
                    └───────────┬──────────────┘
                                │
                    ┌───────────▼──────────────┐
                    │   RECORD CYCLE            │
                    │   - Harness API (metrics) │
                    │   - Vector API (patterns) │
                    │   - Build wave report     │
                    └───────────┬──────────────┘
                                │
                    ┌───────────▼──────────────┐
                    │   NEXT AGENT IS SMARTER   │
                    │   (compound improvement)  │
                    └──────────────────────────┘
```

---

## Pattern Categories

### Build Patterns (how to avoid build failures)
- `pattern-manifest-parse` — Cargo.toml syntax issues
- `pattern-private-field-access` — E0616 errors
- `pattern-unresolved-import` — E0433 from API changes
- `pattern-method-not-found` — E0599 version mismatches
- `pattern-borrow-checker` — E0500 ownership issues
- `pattern-edition-2024-format` — Format trait changes
- `pattern-binaryheap-trait-bounds` — Ord requirement
- `pattern-missing-atomics-import` — std::sync::atomic

### Success Patterns (what works)
- `pattern-fresh-scaffold-success` — New crates with no deps
- `pattern-high-pass-rate` — 98.2% overall pass rate
- `pattern-typical-wave-output` — 5 crates, 1700 LOC, 32 tests

### Harness Patterns (how the allocation system behaves)
- `pattern-harness-gamma-overcorrection` — Dramatic γ shifts
- `pattern-harness-quality-inversion` — Death spiral fix
- `pattern-stress-test-regime` — 50-cycle test results

### Infrastructure Patterns (system-level)
- `pattern-rate-limit-429` — crates.io publish limits
- `pattern-cache-control-api` — CDN caching fix
- `pattern-cargo-check-timeout` — 180s timeout handling

### Knowledge Gaps (patterns we should extract next)
- Dependency version conflict patterns
- Test failure patterns
- Cross-crate dependency graph insights
- Which crate domains have the most empty stubs
- Time-of-day effects on build success

---

## File Locations

| File | Purpose |
|------|---------|
| `SELF-IMPROVING-LOOP.md` | This document |
| `bin/bootstrap.sh` | Agent orientation script |
| `HARNESS-FIELD-REPORT.md` | Detailed cycle report |
| `STRESS-TEST-RESULTS.md` | 50-cycle stress test |
| `DESIGN.md` | Harness γ/η design |
| `BUGFIX-NOTES.md` | Bug fixes and their root causes |
| `/tmp/self-improving-knowledge.json` | Raw pattern data (25 entries) |
| `/tmp/empty_librs_results.txt` | Empty crate registry |
| `/home/phoenix/repos/RUST-BUILD-WAVE*.md` | Build wave reports (437) |

---

*This document is alive. Update it when new patterns are discovered or metrics change significantly.*
