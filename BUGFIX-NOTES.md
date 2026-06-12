# Bugfix Notes — 2026-06-11

## Bug 1: Harness Self-Reinforcing Feedback Loop

### Root Cause
`compute_signal()` only considered exploration ROI thresholds (`roi > 0.6`, `roi < 0.3`) when deciding allocation direction. It completely ignored `output_quality`. This created a death spiral:

1. Agent spends heavily on exploitation (high γ) → quality drops (producing garbage)
2. Low quality depresses exploration ROI (because total yield is low)
3. Low ROI triggers `TernarySignal::Increase` → allocates even MORE to exploitation
4. Quality drops further → repeat

The harness was rewarding spending rather than results. High γ → more γ, even when output was worthless.

### Fix Applied
**Two changes in both `worker/src/index.ts` and `src/lib.rs`:**

1. **Quality-weighted signal** (`compute_signal`): When `ewma_quality < 0.5`, the signal logic inverts:
   - High gamma fraction (> 0.6) + low quality → **Decrease** exploitation (back off the bad activity)
   - Low ROI + low quality → **Decrease** exploitation (try exploring instead)
   - This prevents the self-reinforcing loop: spending on a low-quality activity now causes the harness to back off

2. **Quality gate** (`low_quality_streak` tracking): A new counter tracks consecutive cycles where `output_quality < ewma_quality`. After 3 consecutive low-quality cycles, the harness forces a rebalance to 50/50 (γ = η = 0.5 × capacity), breaking any entrenched feedback loop. The streak counter resets after rebalance or when quality recovers.

### How to Verify After Deployment
1. **Signal inversion test**: Send cycles with high `gamma_spent` (0.8+), low `output_quality` (0.1-0.3), low `exploration_yield` (0.1). Previously this would increase γ. Now it should return `signal: 'Decrease'` or `signal: 'Rebalance (quality gate)'`.
2. **Quality gate test**: Send 3+ consecutive cycles where `output_quality` is below the running EWMA. After the 3rd cycle, response should include `rebalance: true` and allocation should reset to ~50/50.
3. **Normal operation preserved**: Send cycles with quality > 0.6 and varying ROI. Behavior should match previous (high ROI + high quality → explore more, low ROI → exploit more).
4. **Run the Rust test suite**: `cargo test` — new test `test_low_quality_inverts_signal` validates the fix.

---

## Bug 2: Recommend Endpoint Cache (fleet-vector-api)

### Root Cause
The `json()` response helper in `fleet-vector-api/src/index.ts` returned responses with no `Cache-Control` header:

```js
function json(data: unknown, status = 200): Response {
  return new Response(JSON.stringify(data, null, 2), {
    status, headers: { 'Content-Type': 'application/json', ...corsHeaders },
  });
}
```

Without explicit `Cache-Control: no-store`, Cloudflare's edge CDN can cache responses based on zone rules. If the zone has a "Cache Everything" page rule (common for API performance), POST responses get cached by URL path — meaning every POST to `/recommend` returns the first cached response regardless of the request body (`context` field).

### Fix Applied
Added `Cache-Control: no-store, no-cache, must-revalidate, proxy-revalidate` to the `json()` helper's response headers. This applies to ALL endpoints (not just `/recommend`), which is correct — all Vectorize API responses are dynamic and should never be served from cache.

### How to Verify After Deployment
1. **Cache header check**: `curl -I -X POST https://fleet-vector-api.casey-digennaro.workers.dev/recommend -H 'Content-Type: application/json' -d '{"context":"test"}'` — response headers should include `Cache-Control: no-store, no-cache, must-revalidate, proxy-revalidate`.
2. **Distinct results test**: Send two requests with different contexts and verify different results:
   ```bash
   curl -s -X POST .../recommend -d '{"context":"http server"}' | jq '.recommendations[0].name'
   curl -s -X POST .../recommend -d '{"context":"database ORM"}' | jq '.recommendations[0].name'
   ```
   These should return different crate names.
3. **Run existing tests**: `npm test` in fleet-vector-api repo.
