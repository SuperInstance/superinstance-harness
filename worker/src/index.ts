import { Hono } from 'hono';
import { cors } from 'hono/cors';

// ── Types ──────────────────────────────────────────────────────────────
interface Env {
  DB: D1Database;
  STATE: KVNamespace;
}

interface CycleInput {
  timestamp?: number;
  gamma_spent: number;
  eta_spent: number;
  output_quality: number;
  output_quantity: number;
  exploration_yield: number;
}

interface HarnessMetrics {
  ewma_quality: number;
  exploration_roi: number;
  optimal_gamma: number;
  regret: number;
  cycle_count: number;
  low_quality_streak: number;
}

interface HarnessState {
  gamma: number;
  eta: number;
  capacity: number;
  performance: HarnessMetrics;
}

// ── Constants ──────────────────────────────────────────────────────────
const EWMA_ALPHA = 0.3;
const ETA_FLOOR = 0.1;
const ETA_CAP = 0.6;
const ADJUSTMENT_STEP = 0.05;
const DEFAULT_CAPACITY = 1.0;

// ── Helpers ────────────────────────────────────────────────────────────
function defaultState(): HarnessState {
  return {
    gamma: 0.7 * DEFAULT_CAPACITY,
    eta: 0.3 * DEFAULT_CAPACITY,
    capacity: DEFAULT_CAPACITY,
    performance: {
      ewma_quality: 0.5,
      exploration_roi: 0.5,
      optimal_gamma: 0.7,
      regret: 0,
      cycle_count: 0,
    },
  };
}

function enforceConservation(s: HarnessState): void {
  if (s.capacity <= 0) s.capacity = DEFAULT_CAPACITY;
  const total = s.gamma + s.eta;
  if (total <= 0) {
    s.gamma = 0.7 * s.capacity;
    s.eta = 0.3 * s.capacity;
  } else {
    const scale = s.capacity / total;
    s.gamma *= scale;
    s.eta *= scale;
  }
}

function computeSignal(s: HarnessState): -1 | 0 | 1 {
  const roi = s.performance.exploration_roi;
  const quality = s.performance.ewma_quality;
  const gammaFraction = s.gamma / s.capacity;

  // Quality-weighted signal: if quality < 0.5, invert the reward
  // High gamma + low quality = decrease exploitation (stop doing the bad thing)
  if (quality < 0.5) {
    if (gammaFraction > 0.6) return -1; // Spending heavily on exploitation but producing garbage — back off
    if (roi < 0.3) return -1;            // Low ROI + low quality — try exploring instead
    return 0;
  }

  if (roi > 0.6 && quality > 0.6) return -1; // Decrease exploitation
  if (roi < 0.3) return 1;                    // Increase exploitation
  return 0;                                     // Maintain
}

function qualityGateRebalance(s: HarnessState): boolean {
  // Force rebalance if quality drops below its own EWMA for 3+ consecutive cycles
  return s.performance.low_quality_streak >= 3;
}

function applySignal(s: HarnessState, signal: -1 | 0 | 1): void {
  if (signal === -1) {
    s.gamma = Math.max(0, s.gamma - ADJUSTMENT_STEP * s.capacity);
  } else if (signal === 1) {
    s.eta = Math.max(0, s.eta - ADJUSTMENT_STEP * s.capacity);
  }
  s.eta = Math.max(ETA_FLOOR * s.capacity, Math.min(ETA_CAP * s.capacity, s.eta));
  enforceConservation(s);
  s.performance.optimal_gamma = s.gamma;
}

// ── App ────────────────────────────────────────────────────────────────
const app = new Hono<{ Bindings: Env }>();

app.use('*', cors());

// GET / — health
app.get('/', (c) => c.json({ service: 'harness-api', version: '0.1.0', status: 'ok' }));

// POST /cycle — record a completed work cycle
app.post('/cycle', async (c) => {
  const body = await c.req.json<CycleInput>();
  const timestamp = body.timestamp ?? Date.now();

  // Validate
  if (
    typeof body.gamma_spent !== 'number' ||
    typeof body.eta_spent !== 'number' ||
    typeof body.output_quality !== 'number' ||
    typeof body.output_quantity !== 'number' ||
    typeof body.exploration_yield !== 'number'
  ) {
    return c.json({ error: 'Missing or invalid fields' }, 400);
  }

  // Insert into D1
  await c.env.DB.prepare(
    `INSERT INTO cycles (timestamp, gamma_spent, eta_spent, output_quality, output_quantity, exploration_yield)
     VALUES (?, ?, ?, ?, ?, ?)`
  ).bind(
    timestamp,
    body.gamma_spent,
    body.eta_spent,
    body.output_quality,
    body.output_quantity,
    body.exploration_yield
  ).run();

  // Load state from KV
  const raw = await c.env.STATE.get('harness_state');
  const state: HarnessState = raw ? JSON.parse(raw) : defaultState();

  // Update EWMA metrics
  const a = EWMA_ALPHA;
  state.performance.ewma_quality = a * body.output_quality + (1 - a) * state.performance.ewma_quality;
  state.performance.exploration_roi = a * body.exploration_yield + (1 - a) * state.performance.exploration_roi;

  // Compute regret
  const productivity = (body.output_quality * body.output_quantity) / Math.max(body.gamma_spent + body.eta_spent, 0.001);
  const hindsight = productivity * 1.1;
  state.performance.regret += Math.max(0, hindsight - productivity);

  state.performance.cycle_count++;

  // Track quality streak for quality gate
  if (body.output_quality < state.performance.ewma_quality) {
    state.performance.low_quality_streak = (state.performance.low_quality_streak || 0) + 1;
  } else {
    state.performance.low_quality_streak = 0;
  }

  // Quality gate: force rebalance if quality declining for 3+ consecutive cycles
  if (qualityGateRebalance(state)) {
    // Reset to balanced allocation (50/50) to break feedback loop
    state.gamma = 0.5 * state.capacity;
    state.eta = 0.5 * state.capacity;
    state.performance.low_quality_streak = 0;
    enforceConservation(state);
    state.performance.optimal_gamma = state.gamma;
    await c.env.STATE.put('harness_state', JSON.stringify(state));
    return c.json({
      ok: true,
      signal: 'Rebalance (quality gate)',
      signal_value: 0,
      rebalance: true,
      new_allocation: { gamma: state.gamma, eta: state.eta },
      metrics: state.performance,
    });
  }

  // Compute signal and apply
  const signal = computeSignal(state);
  applySignal(state, signal);

  // Save state
  await c.env.STATE.put('harness_state', JSON.stringify(state));

  return c.json({
    ok: true,
    signal: signal === -1 ? 'Decrease' : signal === 0 ? 'Maintain' : 'Increase',
    signal_value: signal,
    new_allocation: { gamma: state.gamma, eta: state.eta },
    metrics: state.performance,
  });
});

// GET /allocation — recommended γ/η for next cycle
app.get('/allocation', async (c) => {
  const raw = await c.env.STATE.get('harness_state');
  const state: HarnessState = raw ? JSON.parse(raw) : defaultState();
  const signal = computeSignal(state);

  return c.json({
    gamma: state.gamma,
    eta: state.eta,
    capacity: state.capacity,
    gamma_fraction: state.gamma / state.capacity,
    eta_fraction: state.eta / state.capacity,
    signal: signal === -1 ? 'Decrease' : signal === 0 ? 'Maintain' : 'Increase',
    signal_value: signal,
  });
});

// GET /metrics — current performance metrics
app.get('/metrics', async (c) => {
  const raw = await c.env.STATE.get('harness_state');
  const state: HarnessState = raw ? JSON.parse(raw) : defaultState();

  // Get recent cycles from D1
  const recent = await c.env.DB.prepare(
    'SELECT * FROM cycles ORDER BY timestamp DESC LIMIT 20'
  ).all();

  return c.json({
    performance: state.performance,
    allocation: { gamma: state.gamma, eta: state.eta, capacity: state.capacity },
    recent_cycles: recent.results?.length ?? 0,
  });
});

// POST /feedback — external feedback signal
app.post('/feedback', async (c) => {
  const body = await c.req.json<{ quality_signal: number }>();
  if (typeof body.quality_signal !== 'number') {
    return c.json({ error: 'quality_signal required (-1 to 1)' }, 400);
  }

  const raw = await c.env.STATE.get('harness_state');
  const state: HarnessState = raw ? JSON.parse(raw) : defaultState();

  let signal: -1 | 0 | 1;
  if (body.quality_signal > 0.2) signal = -1;      // Positive → explore more
  else if (body.quality_signal < -0.2) signal = 1;  // Negative → exploit more
  else signal = 0;

  applySignal(state, signal);
  await c.env.STATE.put('harness_state', JSON.stringify(state));

  return c.json({
    ok: true,
    applied_signal: signal === -1 ? 'Decrease' : signal === 0 ? 'Maintain' : 'Increase',
    new_allocation: { gamma: state.gamma, eta: state.eta },
  });
});

export default app;
