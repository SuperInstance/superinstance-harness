# superinstance-harness — Design Document

## The Conservation Law: γ + η = C

### Core Insight

All productive systems operate under finite capacity — time, compute, tokens, attention. This capacity C is split between two fundamental modes:

- **γ (gamma) — Exploitation**: Using proven patterns, executing known workflows, shipping output. High γ means reliable, predictable results.
- **η (eta) — Exploration**: Research, experimentation, trying new approaches, discovering patterns. High η means more novelty, more risk, more potential breakthroughs.

The conservation law **γ + η = C** is invariant. You cannot increase one without decreasing the other. The question isn't *whether* to explore or exploit — it's *how much* of each, *when*.

### Why This Matters

Most systems use static allocation (e.g., "spend 20% on R&D"). This is suboptimal because:

1. **Exploration ROI varies over time** — Early in a project, exploration yields high returns (you know little). Later, it yields less (you've found the good patterns).
2. **Environmental shifts change the optimum** — A new tool, framework, or requirement can suddenly make exploration valuable again.
3. **Static allocation wastes capacity** — If you're at 20% exploration but exploration ROI is near zero, you're wasting 20% of your budget.

The harness adapts the γ/η split in real-time based on observed outcomes.

## Architecture

```
┌─────────────────────────────────────────────┐
│                  HarnessState                │
│                                             │
│  γ ───────┐                                 │
│           ├── C (conservation enforced)      │
│  η ───────┘                                 │
│                                             │
│  PerformanceMetrics                         │
│  ├── ewma_quality  (smoothed output score)   │
│  ├── exploration_roi (recent discovery rate) │
│  ├── optimal_gamma  (recommended γ)          │
│  └── regret  (cumulative suboptimality)      │
│                                             │
│  CycleHistory → drives adaptation            │
└─────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────┐
│        Ternary Signal       │
│  {-1: Decrease exploitation}│
│  { 0: Maintain allocation } │
│  {+1: Increase exploitation}│
└─────────────────────────────┘
```

## Adaptive Algorithm

### Step 1: Record Cycle Output

Each work cycle produces a `CycleRecord`:
- How much γ and η were spent
- Output quality (0-1)
- Output quantity (items produced)
- Exploration yield (useful discoveries / total explorations)

### Step 2: Update Metrics (EWMA)

We use Exponentially Weighted Moving Averages to track:

```
ewma_quality = α × new_quality + (1 - α) × old_ewma_quality
exploration_roi = α × new_yield + (1 - α) × old_roi
```

With α = 0.3, this gives ~85% weight to the last 5 cycles while never fully forgetting history.

### Step 3: Compute Ternary Signal

The decision language is deliberately minimal — three states:

```
if exploration_roi > 0.6 AND ewma_quality > 0.6 → Decrease (explore more)
if exploration_roi < 0.3                         → Increase (exploit more)
otherwise                                        → Maintain
```

**Why ternary?** Three states is the minimum that captures directional intent without overfitting to noise. Binary (increase/decrease) would force constant oscillation. Continuous values would be noisy. Ternary with hysteresis bands (0.3–0.6 dead zone) provides stable, interpretable decisions.

### Step 4: Apply Signal (Adjust Allocation)

Each signal shifts the γ/η balance by a fixed step (5% of capacity):

| Signal | γ change | η change | Effect |
|--------|----------|----------|--------|
| Decrease (-1) | γ -= step | η += step | More exploration |
| Maintain (0) | — | — | Stay the course |
| Increase (+1) | γ += step | η -= step | More exploitation |

### Step 5: Enforce Constraints

After every adjustment:
1. **Floor**: η ≥ 10% of C (never stop exploring entirely)
2. **Cap**: η ≤ 60% of C (don't go overboard on exploration)
3. **Conservation**: γ + η = C (always re-normalize)

## Components

### Rust Crate (`superinstance-harness`)

The core algorithm implementation. Pure computation, no I/O dependencies.

- `HarnessState` — Main state machine
- `CycleRecord` — Per-cycle data
- `PerformanceMetrics` — Aggregate tracking
- `TernarySignal` — Decision language
- `Allocation` — Recommended next-cycle split

**Tests (13)**: Conservation law invariants, signal computation, EWMA updates, convergence, serialization roundtrip, feedback, floor/cap enforcement, productivity metrics, reset recovery.

### Cloudflare Worker (`harness-api`)

REST API wrapping the harness algorithm with persistent state.

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/cycle` | POST | Record completed cycle, get next allocation |
| `/allocation` | GET | Current recommended γ/η split |
| `/metrics` | GET | Performance metrics + recent cycles |
| `/feedback` | POST | External feedback signal |

**Storage**: D1 for cycle history (analytics), KV for current state (fast reads).

## Convergence Behavior

The algorithm converges to different equilibria depending on the environment:

- **Stable environment** (exploration consistently low-yield): Converges to high γ (~80-90%), minimal exploration. Efficient execution.
- **Volatile environment** (exploration consistently high-yield): Converges to moderate γ (~40-50%), sustained exploration. Adaptive behavior.
- **Mixed environment**: Oscillates around the optimal split, with the dead zone preventing jitter.

Regret (cumulative suboptimality) grows sublinearly — the algorithm learns from its mistakes.

## Integration Points

```
harness-api ──→ fleet-vector-api  (crate similarity for context)
            ──→ fleet-metrics-cron (performance signals)
            ──→ knowledge-cron     (pattern detection from cycle data)
```

Future: The harness can feed its own metrics back into fleet-metrics-cron, creating a meta-optimization loop — the harness optimizing its own optimization.
