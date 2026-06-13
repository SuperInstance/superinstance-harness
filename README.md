# SuperInstance Harness — Self-Optimizing γ/η Allocator

A **self-optimizing work harness** that uses the conservation law **γ + η = C** to adaptively balance exploitation (γ) and exploration (η) across work cycles, using ternary signals, EWMA quality tracking, and regret accumulation.

## Why It Matters

Every productive system faces the explore-vs-exploit dilemma: should you keep doing what works (exploitation), or try new approaches that might be better (exploration)? The multi-armed bandit literature studies this tradeoff; reinforcement learning systems implement it via ε-greedy or UCB policies. SuperInstance Harness formalizes it as a **conservation law**: the total capacity budget C is split between γ (exploitation) and η (exploration), and γ + η = C always holds. The harness adjusts the split based on real-time feedback — when exploration yields valuable discoveries, increase η; when output quality is high, increase γ. This creates a self-tuning system that converges on the optimal balance.

## How It Works

### The Conservation Law

```
γ + η = C    (always enforced)
```

- **γ (gamma)**: fraction of capacity spent on exploitation — executing known-good patterns
- **η (eta)**: fraction spent on exploration — research, experimentation, new approaches
- **C**: total capacity budget (default: 1.0)

The harness maintains `{ gamma: f64, eta: f64, capacity: f64 }` and re-normalizes after every adjustment so the sum always equals C.

### Ternary Signals

After each work cycle, the harness emits a `TernarySignal`:

```
Decrease (-1): increase exploration (η ↑, γ ↓)
Maintain  (0): keep current ratio
Increase (+1): increase exploitation (γ ↑, η ↓)
```

Signals are computed from exploration ROI and EWMA quality:

```
If exploration_roi > threshold → Decrease (exploration is valuable, do more)
If output_quality > ewma_quality → Increase (exploitation is working well)
Otherwise → Maintain
```

### EWMA Quality Tracking

Output quality uses an Exponentially Weighted Moving Average:

```
Q_new = α × quality_this_cycle + (1 - α) × Q_previous
```

With α = 0.3 (EWMA_ALPHA), this gives recent cycles 30% weight and history 70% — responsive but not twitchy.

### Regret Accumulation

The harness tracks **cumulative regret** — the gap between actual productivity and hindsight-optimal productivity:

```
regret += max(0, hindsight_productivity - actual_productivity)
```

A growing regret indicates the allocation strategy is suboptimal. The quality gate checks for consecutive low-quality cycles and forces rebalancing after 3.

### Bounds

```
η_floor = 0.1    (never stop exploring entirely)
η_cap  = 0.6    (never spend more than 60% on exploration)
step   = 0.05   (5% adjustment per signal)
```

These prevent the system from getting stuck in pure-exploitation or pure-exploration modes.

### Adjustment Step

Each ternary signal triggers a step adjustment:

```
On Increase: γ += step × C, η -= step × C
On Decrease: γ -= step × C, η += step × C
Then clamp η to [η_floor × C, η_cap × C]
Then re-normalize so γ + η = C
```

### Complexity

| Operation | Time |
|-----------|------|
| Record cycle | O(1) |
| Compute signal | O(1) |
| Rebalance | O(1) |
| Rolling productivity | O(k) for k-cycle window |

## Quick Start

```rust
use superinstance_harness::{HarnessState, CycleRecord};

fn main() {
    let mut harness = HarnessState::new();
    println!("Initial: γ={:.2} η={:.2} C={:.2}", harness.gamma, harness.eta, harness.capacity);

    // Simulate work cycles
    for i in 0..10 {
        let record = CycleRecord {
            timestamp: 1000 + i,
            gamma_spent: harness.gamma,
            eta_spent: harness.eta,
            output_quality: 0.7 + i as f64 * 0.02,
            output_quantity: 3.0,
            exploration_yield: 0.3 + i as f64 * 0.03,
        };

        let signal = harness.record_cycle(record);
        println!("Cycle {}: signal={:?}, γ={:.3}, η={:.3}, quality={:.3}",
            i + 1, signal, harness.gamma, harness.eta, harness.performance.ewma_quality);
    }

    println!("Conservation holds: {}", harness.verify_conservation());
    println!("Total regret: {:.4}", harness.performance.regret);
}
```

## API

### `HarnessState`
- `new() / with_capacity(C)` — create harness (default 70/30 γ/η split)
- `record_cycle(record) -> TernarySignal` — process a cycle, return signal
- `verify_conservation() -> bool` — check γ + η = C
- `gamma_fraction() / eta_fraction() / ratio()` — current allocation
- `rolling_productivity(k) -> f64` — mean productivity over last k cycles
- `rebalance()` — forced reallocation

### `CycleRecord`
- `gamma_spent, eta_spent` — capacity consumed per category
- `output_quality (0-1), output_quantity` — what was produced
- `exploration_yield (0-1)` — fraction of explorations that found something useful
- `productivity() -> f64` — quality × quantity / capacity_used

### `PerformanceMetrics`
- `ewma_quality, exploration_roi, optimal_gamma, regret, cycle_count, low_quality_streak`

### `TernarySignal`
- `Decrease (-1), Maintain (0), Increase (+1)`

## Architecture Notes

This crate IS the γ + η = C conservation law in action. It's the core optimization engine of SuperInstance. The Cocapn delegates fleet-wide allocation decisions to this harness, which tunes the γ/η ratio for each ship and for the fleet as a whole. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Auer, P. et al. (2002). *Finite-time Analysis of the Multiarmed Bandit Problem*. Machine Learning 47. (UCB1.)
- Sutton, R. & Barto, A. (2018). *Reinforcement Learning: An Introduction*, 2nd ed., Ch. 2. MIT Press.
- Bergstra, J. & Bengio, Y. (2012). *Random Search for Hyper-Parameter Optimization*. JMLR 13. (Exploration strategies.)

## License

MIT
