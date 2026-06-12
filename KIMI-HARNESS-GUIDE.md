# Kimi Harness Guide — Self-Optimizing Development with γ + η = C

## What This Is

A guide for Kimi CLI (or any AI coding assistant) to use the SuperInstance self-optimizing harness for adaptive development. The harness tracks how you spend your capacity between exploitation (γ) and exploration (η), learns from results, and tells you what to do next.

## The Theory

**γ + η = C** (conservation law)
- **γ (exploitation)** = shipping, building, publishing, executing known-good patterns
- **η (exploration)** = research, experimentation, trying new approaches, reading docs
- **C = 1.0** = your total capacity per cycle (normalized)

The harness adaptively adjusts the γ/η split based on:
1. **EWMA quality tracking** — exponentially weighted moving average of your output quality
2. **Exploration ROI** — useful discoveries per unit exploration spent
3. **Ternary signals** — {-1, 0, +1} = decrease/maintain/increase exploitation

## API Endpoints

Base URL: `https://harness-api.casey-digennaro.workers.dev`

### 1. Start a Work Cycle — `GET /allocation`

```bash
curl -s "https://harness-api.casey-digennaro.workers.dev/allocation"
```

Returns:
```json
{
  "gamma": 0.67,        // 67% exploitation recommended
  "eta": 0.33,          // 33% exploration recommended
  "signal": "Maintain", // Ternary signal: Increase/Maintain/Decrease exploitation
  "signal_value": 0     // -1, 0, or +1
}
```

**Do this first.** It tells you how to split your time this cycle.

### 2. Record Results — `POST /cycle`

```bash
curl -s -X POST "https://harness-api.casey-digennaro.workers.dev/cycle" \
  -H "Content-Type: application/json" \
  -d '{
    "gamma_spent": 0.7,
    "eta_spent": 0.3,
    "output_quality": 0.85,
    "output_quantity": 5,
    "exploration_yield": 0.4
  }'
```

Fields:
- `gamma_spent` (0-1): fraction of time on exploitation (building, shipping, testing)
- `eta_spent` (0-1): fraction of time on exploration (reading, researching, experimenting)
- `output_quality` (0-1): how good was the output? (build pass rate, test coverage, code quality)
- `output_quantity` (number): items produced (crates published, tests written, files fixed)
- `exploration_yield` (0-1): what fraction of exploration was useful? (did you find something valuable?)

Returns the new allocation + signal for next cycle.

### 3. External Feedback — `POST /feedback`

```bash
curl -s -X POST "https://harness-api.casey-digennaro.workers.dev/feedback" \
  -H "Content-Type: application/json" \
  -d '{"quality_signal": 0.8, "context": "build passes improved"}'
```

Signal from outside the cycle — user feedback, CI results, review scores. Range -1 to 1.

### 4. Check Metrics — `GET /metrics`

```bash
curl -s "https://harness-api.casey-digennaro.workers.dev/metrics"
```

Returns EWMA quality, exploration ROI, optimal γ, regret, cycle count.

## How to Use It — The Kimi Loop

### Step 1: Check In

At the start of each work session:

```bash
curl -s "https://harness-api.casey-digennaro.workers.dev/allocation"
```

Interpret the signal:
- **"Increase" (signal +1)** → spend more time shipping/building. Execute. Don't read docs.
- **"Maintain" (signal 0)** → balanced. Keep doing what you're doing.
- **"Decrease" (signal -1)** → you're over-exploiting. Explore more. Read, research, prototype.

### Step 2: Plan Your Work

Given γ/η split, allocate your tasks:

**γ tasks (exploitation — do these first when γ > 0.6):**
- Build, test, fix compile errors
- Publish crates/npm packages
- Push to git
- Run CI
- Write tests for existing code
- Deploy workers

**η tasks (exploration — do these first when η > 0.4):**
- Read code you've never seen before
- Research architecture patterns
- Write design docs
- Prototype new approaches
- Semantic search for cross-repo patterns
- Review synthesis documents

### Step 3: Use Semantic Search for Exploration

The fleet-vector-api gives you semantic search across 1,012 crates:

```bash
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/search" \
  -H "Content-Type: application/json" \
  -d '{"query": "distributed consensus protocol", "topK": 10}'
```

Use this to:
- Find relevant code before building
- Discover cross-repo patterns
- Identify gaps (crates that should exist but don't)

### Step 4: Use the RAG Agent for Deep Questions

```bash
curl -s -X POST "https://superinstance-agent.casey-digennaro.workers.dev/ask" \
  -H "Content-Type: application/json" \
  -d '{"question": "What crates implement spectral methods for ternary signals?"}'
```

Returns an LLM-generated answer with citations from the corpus.

### Step 5: Get Recommendations

```bash
curl -s -X POST "https://superinstance-agent.casey-digennaro.workers.dev/recommend" \
  -H "Content-Type: application/json" \
  -d '{"task": "building a self-optimizing AI agent harness"}'
```

Returns ranked crate suggestions with integration guidance.

### Step 6: Record Your Cycle

At the end of each significant chunk of work:

```bash
curl -s -X POST "https://harness-api.casey-digennaro.workers.dev/cycle" \
  -H "Content-Type: application/json" \
  -d '{
    "gamma_spent": <fraction of time building>,
    "eta_spent": <fraction of time exploring>,
    "output_quality": <0-1 quality score>,
    "output_quantity": <items produced>,
    "exploration_yield": <fraction of exploration that was useful>
  }'
```

Be honest. The harness only works if you feed it real data.

## Example Session: Publishing Wave 6

```bash
# 1. Check in
ALLOC=$(curl -s "https://harness-api.casey-digennaro.workers.dev/allocation")
# gamma=0.67, eta=0.33, signal="Maintain"

# 2. Plan: 67% exploitation = publish crates, 33% exploration = find new candidates

# 3. Exploit: publish crates (γ work)
cd /home/phoenix/repos/cudaclaw && cargo publish --no-verify
# ... publish more crates ...

# 4. Explore: find underrepresented domains (η work)
curl -s -X POST "https://fleet-vector-api.casey-digennaro.workers.dev/search" \
  -d '{"query": "neural network ternary quantization", "topK": 5}'
# Discovery: only 3 crates in this space → opportunity for wave 7

# 5. Record cycle
curl -s -X POST "https://harness-api.casey-digennaro.workers.dev/cycle" \
  -d '{"gamma_spent": 0.7, "eta_spent": 0.3, "output_quality": 0.9, "output_quantity": 8, "exploration_yield": 0.5}'
# → new allocation: gamma=0.69, eta=0.31, signal="Increase"
# Harness says: exploitation is working, do more of it
```

## Kimi CLI One-Liners

```bash
# Check allocation
kimi -y --print -p "Run: curl -s https://harness-api.casey-digennaro.workers.dev/allocation. What should I focus on this cycle?"

# Record a cycle
kimi -y --print -p "Record work cycle: curl -s -X POST https://harness-api.casey-digennaro.workers.dev/cycle -H 'Content-Type: application/json' -d '{\"gamma_spent\": 0.8, \"eta_spent\": 0.2, \"output_quality\": 0.9, \"output_quantity\": 12, \"exploration_yield\": 0.3}'"

# Find crates for a task
kimi -y --print -p "Search our crate corpus: curl -s -X POST https://fleet-vector-api.casey-digennaro.workers.dev/search -H 'Content-Type: application/json' -d '{\"query\": \"self-optimizing agent harness\", \"topK\": 5}'"

# Ask the RAG agent
kimi -y --print -p "Ask: curl -s -X POST https://superinstance-agent.casey-digennaro.workers.dev/ask -H 'Content-Type: application/json' -d '{\"question\": \"What is the conservation law and how does it relate to work allocation?\"}'"
```

## Ecosystem Map

| Service | URL | Purpose |
|---------|-----|---------|
| **harness-api** | `https://harness-api.casey-digennaro.workers.dev` | γ/η allocation, cycle tracking, metrics |
| **fleet-vector-api** | `https://fleet-vector-api.casey-digennaro.workers.dev` | Semantic search across 1,012 crates |
| **superinstance-agent** | `https://superinstance-agent.casey-digennaro.workers.dev` | RAG Q&A + recommendations |
| **fleet-edge-worker** | (deployed) | Edge compute |
| **fleet-auth** | (deployed) | Auth service |
| **fleet-metrics-cron** | (deployed) | 5-min metrics collection |

## Quality Scoring Guide

When recording `output_quality`, use:
- **0.9-1.0**: Build passes, tests pass, docs written, code is clean
- **0.7-0.9**: Build passes, maybe some test gaps
- **0.5-0.7**: Build passes but messy, or partial completion
- **0.3-0.5**: Build fails, but good exploration/learning
- **0.0-0.3**: Nothing useful produced

## Exploration Yield Guide

When recording `exploration_yield`, use:
- **0.8-1.0**: Found exactly what you needed, immediately actionable
- **0.5-0.8**: Found useful patterns, needs synthesis
- **0.2-0.5**: Some interesting leads, not immediately useful
- **0.0-0.2**: Dead ends, but negative results are still data

## The Feedback Loop

```
┌─────────────────────────────────────────────┐
│  GET /allocation                            │
│  "gamma: 0.67, eta: 0.33, signal: Maintain" │
└──────────────────┬──────────────────────────┘
                   │
          ┌────────▼────────┐
          │   DO THE WORK   │
          │  γ: build/ship  │
          │  η: read/search │
          └────────┬────────┘
                   │
┌──────────────────▼──────────────────────────┐
│  POST /cycle                                │
│  "quality: 0.85, quantity: 5, yield: 0.4"   │
└──────────────────┬──────────────────────────┘
                   │
          ┌────────▼────────┐
          │  HARNESS LEARNS │
          │  Adjusts γ/η    │
          │  Issues signal  │
          └────────┬────────┘
                   │
                   └──→ Back to GET /allocation
```

## Anti-Patterns

1. **Always reporting quality=1.0** — the harness learns nothing. Be honest.
2. **Ignoring the signal** — if it says "Decrease exploitation," stop shipping and start reading.
3. **Not recording cycles** — the harness needs data to adapt.
4. **Only doing γ work** — you'll hit diminishing returns. Exploration feeds exploitation.
5. **Only doing η work** — you'll never ship. Exploitation validates exploration.

## Philosophy

This isn't just a tool — it's a formalization of how good developers already work. The conservation law says you can't do everything. The harness helps you decide *what* to do, *when*, and *how much*. The ternary signal language is deliberately simple: {-1, 0, +1}. You don't need fine-grained control. You need clear direction.

**Build the forge while forging.** The harness improves itself as you use it.
