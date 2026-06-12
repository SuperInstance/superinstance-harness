//! # superinstance-harness
//!
//! A self-optimizing harness that uses the conservation law γ + η = C to
//! adaptively allocate between exploitation (γ) and exploration (η).
//!
//! ## Core Theory
//!
//! The conservation law states that total capacity C is split between:
//! - **γ (gamma)**: exploitation — using known-good patterns, executing, shipping
//! - **η (eta)**: exploration — research, experimentation, trying new approaches
//!
//! γ + η = C (always enforced)
//!
//! The harness adaptively adjusts the γ/η ratio based on exploration ROI
//! using ternary signals: {-1, 0, +1} meaning decrease/maintain/increase exploitation.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Default EWMA smoothing factor (α)
pub const EWMA_ALPHA: f64 = 0.3;

/// Minimum exploration floor — never go below this
pub const ETA_FLOOR: f64 = 0.1;

/// Maximum exploration cap
pub const ETA_CAP: f64 = 0.6;

/// Adjustment step size per ternary signal
pub const ADJUSTMENT_STEP: f64 = 0.05;

/// Default total capacity
pub const DEFAULT_CAPACITY: f64 = 1.0;

/// Ternary signal for allocation decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i8)]
pub enum TernarySignal {
    /// Decrease exploitation (increase exploration)
    Decrease = -1,
    /// Maintain current allocation
    Maintain = 0,
    /// Increase exploitation (decrease exploration)
    Increase = 1,
}

impl TernarySignal {
    /// Create from i8 value, clamping to valid range
    pub fn from_i8(v: i8) -> Self {
        match v.cmp(&0) {
            Ordering::Less => TernarySignal::Decrease,
            Ordering::Equal => TernarySignal::Maintain,
            Ordering::Greater => TernarySignal::Increase,
        }
    }
}

/// Record of a completed work cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CycleRecord {
    /// Unix timestamp of cycle completion
    pub timestamp: i64,
    /// Amount of capacity spent on exploitation
    pub gamma_spent: f64,
    /// Amount of capacity spent on exploration
    pub eta_spent: f64,
    /// Quality of output (0.0 - 1.0)
    pub output_quality: f64,
    /// Number of items/tasks produced
    pub output_quantity: f64,
    /// Useful discoveries / total explorations (0.0 - 1.0)
    pub exploration_yield: f64,
}

impl CycleRecord {
    /// Total capacity consumed this cycle
    pub fn total_spent(&self) -> f64 {
        self.gamma_spent + self.eta_spent
    }

    /// Effective productivity: quality × quantity / capacity_used
    pub fn productivity(&self) -> f64 {
        let spent = self.total_spent();
        if spent <= 0.0 {
            return 0.0;
        }
        (self.output_quality * self.output_quantity) / spent
    }

    /// Exploration ROI: normalized to [0,1] based on yield
    pub fn exploration_roi(&self) -> f64 {
        self.exploration_yield
    }
}

/// Aggregate performance metrics maintained across cycles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Exponentially-weighted moving average of output quality
    pub ewma_quality: f64,
    /// Return on exploration investment (recent)
    pub exploration_roi: f64,
    /// Current recommended gamma value
    pub optimal_gamma: f64,
    /// Cumulative regret vs. optimal hindsight allocation
    pub regret: f64,
    /// Total cycles processed
    pub cycle_count: u64,
    /// Consecutive cycles where output_quality < ewma_quality (for quality gate)
    pub low_quality_streak: u32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        PerformanceMetrics {
            ewma_quality: 0.5,
            exploration_roi: 0.5,
            optimal_gamma: 0.7,
            regret: 0.0,
            cycle_count: 0,
            low_quality_streak: 0,
        }
    }
}

/// The core harness state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessState {
    /// Exploitation weight (0.0 - 1.0)
    pub gamma: f64,
    /// Exploration weight (0.0 - 1.0)
    pub eta: f64,
    /// Total capacity budget
    pub capacity: f64,
    /// History of completed cycles
    pub history: Vec<CycleRecord>,
    /// Current performance metrics
    pub performance: PerformanceMetrics,
}

impl HarnessState {
    /// Create a new harness with default allocation (70/30 exploitation/exploration)
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    /// Create with a specific capacity budget
    pub fn with_capacity(capacity: f64) -> Self {
        let gamma = 0.7 * capacity;
        let eta = 0.3 * capacity;
        HarnessState {
            gamma,
            eta,
            capacity,
            history: Vec::new(),
            performance: PerformanceMetrics::default(),
        }
    }

    /// Enforce the conservation law: γ + η = C
    pub fn enforce_conservation(&mut self) {
        // Clamp capacity
        if self.capacity <= 0.0 {
            self.capacity = DEFAULT_CAPACITY;
        }
        let total = self.gamma + self.eta;
        if total <= 0.0 {
            // Reset to defaults
            self.gamma = 0.7 * self.capacity;
            self.eta = 0.3 * self.capacity;
        } else {
            // Normalize to capacity
            let scale = self.capacity / total;
            self.gamma *= scale;
            self.eta *= scale;
        }
    }

    /// Verify conservation law holds
    pub fn verify_conservation(&self) -> bool {
        let total = self.gamma + self.eta;
        (total - self.capacity).abs() < 1e-9
    }

    /// Get the current γ/η ratio
    pub fn ratio(&self) -> f64 {
        if self.eta <= 0.0 {
            return f64::INFINITY;
        }
        self.gamma / self.eta
    }

    /// Get the γ fraction (gamma / capacity)
    pub fn gamma_fraction(&self) -> f64 {
        if self.capacity <= 0.0 {
            return 0.0;
        }
        self.gamma / self.capacity
    }

    /// Get the η fraction (eta / capacity)
    pub fn eta_fraction(&self) -> f64 {
        if self.capacity <= 0.0 {
            return 0.0;
        }
        self.eta / self.capacity
    }

    /// Record a completed cycle and update metrics
    pub fn record_cycle(&mut self, record: CycleRecord) -> TernarySignal {
        // Update EWMA quality
        let alpha = EWMA_ALPHA;
        self.performance.ewma_quality =
            alpha * record.output_quality + (1.0 - alpha) * self.performance.ewma_quality;

        // Update exploration ROI
        let cycle_roi = record.exploration_roi();
        self.performance.exploration_roi =
            alpha * cycle_roi + (1.0 - alpha) * self.performance.exploration_roi;

        // Compute optimal hindsight allocation for this cycle
        let _hindsight_gamma = if record.exploration_yield > 0.5 {
            // Exploration was fruitful → should have explored more
            record.gamma_spent * 0.9
        } else {
            // Exploration was poor → should have exploited more
            record.gamma_spent * 1.1
        };

        // Accumulate regret
        let actual_value = record.productivity();
        let hindsight_value = if record.total_spent() > 0.0 {
            let h_productivity = record.output_quality * record.output_quantity * 1.1;
            h_productivity / record.total_spent()
        } else {
            0.0
        };
        self.performance.regret += (hindsight_value - actual_value).max(0.0);

        // Track quality streak for quality gate
        if record.output_quality < self.performance.ewma_quality {
            self.performance.low_quality_streak += 1;
        } else {
            self.performance.low_quality_streak = 0;
        }

        // Store history
        self.history.push(record);
        self.performance.cycle_count += 1;

        // Quality gate: force rebalance if quality declining for 3+ consecutive cycles
        if self.quality_gate_triggered() {
            self.rebalance();
            return TernarySignal::Maintain; // Signal rebalance happened, no incremental change
        }

        // Generate signal for next cycle
        let signal = self.compute_signal();
        self.apply_signal(signal);
        signal
    }

    /// Compute ternary signal based on current metrics, weighted by quality
    pub fn compute_signal(&self) -> TernarySignal {
        let roi = self.performance.exploration_roi;
        let quality = self.performance.ewma_quality;
        let gamma_fraction = self.gamma_fraction();

        // Quality-weighted signal: if quality < 0.5, invert the reward direction.
        // High gamma + low quality = decrease exploitation (stop doing the bad thing).
        if quality < 0.5 {
            if gamma_fraction > 0.6 {
                return TernarySignal::Decrease; // Spending heavily but producing garbage — back off
            }
            if roi < 0.3 {
                return TernarySignal::Decrease; // Low ROI + low quality — try exploring
            }
            return TernarySignal::Maintain;
        }

        // If exploration ROI is high and quality is good → explore more (decrease exploitation)
        if roi > 0.6 && quality > 0.6 {
            TernarySignal::Decrease
        }
        // If exploration ROI is low → exploit more (increase exploitation)
        else if roi < 0.3 {
            TernarySignal::Increase
        }
        // Otherwise maintain
        else {
            TernarySignal::Maintain
        }
    }

    /// Check if the quality gate should trigger a forced rebalance.
    /// Returns true when output_quality has been below EWMA for 3+ consecutive cycles.
    pub fn quality_gate_triggered(&self) -> bool {
        self.performance.low_quality_streak >= 3
    }

    /// Force a rebalance: reset to 50/50 allocation to break feedback loops
    pub fn rebalance(&mut self) {
        self.gamma = 0.5 * self.capacity;
        self.eta = 0.5 * self.capacity;
        self.performance.low_quality_streak = 0;
        self.enforce_conservation();
        self.performance.optimal_gamma = self.gamma;
    }

    /// Apply a ternary signal to adjust allocation
    pub fn apply_signal(&mut self, signal: TernarySignal) {
        match signal {
            TernarySignal::Decrease => {
                // Decrease exploitation → increase exploration
                self.gamma = (self.gamma - ADJUSTMENT_STEP * self.capacity).max(0.0);
            }
            TernarySignal::Increase => {
                // Increase exploitation → decrease exploration
                self.eta = (self.eta - ADJUSTMENT_STEP * self.capacity).max(0.0);
            }
            TernarySignal::Maintain => {
                // No change
            }
        }

        // Enforce floors and caps
        self.eta = self.eta.clamp(
            ETA_FLOOR * self.capacity,
            ETA_CAP * self.capacity,
        );

        // Enforce conservation law
        self.enforce_conservation();

        // Update optimal_gamma in metrics
        self.performance.optimal_gamma = self.gamma;
    }

    /// Apply external feedback signal
    pub fn apply_feedback(&mut self, quality_signal: f64) {
        // quality_signal: -1.0 to 1.0 indicating external assessment
        // Positive = things going well → can afford more exploration
        // Negative = things going poorly → focus on exploitation
        if quality_signal > 0.2 {
            self.apply_signal(TernarySignal::Decrease);
        } else if quality_signal < -0.2 {
            self.apply_signal(TernarySignal::Increase);
        }
    }

    /// Get recommended allocation for the next cycle
    pub fn allocation(&self) -> Allocation {
        Allocation {
            gamma: self.gamma,
            eta: self.eta,
            gamma_budget: self.gamma,
            eta_budget: self.eta,
            signal: self.compute_signal(),
        }
    }

    /// Serialize state to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize state from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Get recent N cycles from history
    pub fn recent_cycles(&self, n: usize) -> &[CycleRecord] {
        let start = self.history.len().saturating_sub(n);
        &self.history[start..]
    }

    /// Compute rolling average productivity over last N cycles
    pub fn rolling_productivity(&self, n: usize) -> f64 {
        let recent = self.recent_cycles(n);
        if recent.is_empty() {
            return 0.0;
        }
        recent.iter().map(|c| c.productivity()).sum::<f64>() / recent.len() as f64
    }

    /// Reset harness to initial state (keeping capacity)
    pub fn reset(&mut self) {
        let cap = self.capacity;
        *self = Self::with_capacity(cap);
    }
}

/// Recommended allocation for next cycle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    /// Exploitation weight
    pub gamma: f64,
    /// Exploration weight
    pub eta: f64,
    /// Recommended exploitation budget for next cycle
    pub gamma_budget: f64,
    /// Recommended exploration budget for next cycle
    pub eta_budget: f64,
    /// Current signal direction
    pub signal: TernarySignal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conservation_law_default() {
        let h = HarnessState::new();
        assert!(h.verify_conservation(), "γ + η must equal C");
        assert!((h.gamma + h.eta - h.capacity).abs() < 1e-9);
    }

    #[test]
    fn test_conservation_after_signal() {
        let mut h = HarnessState::new();
        h.apply_signal(TernarySignal::Decrease);
        assert!(h.verify_conservation(), "Conservation after Decrease");
        h.apply_signal(TernarySignal::Increase);
        assert!(h.verify_conservation(), "Conservation after Increase");
        h.apply_signal(TernarySignal::Maintain);
        assert!(h.verify_conservation(), "Conservation after Maintain");
    }

    #[test]
    fn test_eta_floor_and_cap() {
        let mut h = HarnessState::new();
        // Hammer increase signal many times
        for _ in 0..100 {
            h.apply_signal(TernarySignal::Increase);
        }
        assert!(h.eta >= ETA_FLOOR * h.capacity - 1e-9, "eta must respect floor");
        assert!(h.eta <= ETA_CAP * h.capacity + 1e-9, "eta must respect cap");
        assert!(h.verify_conservation());
    }

    #[test]
    fn test_ternary_signal_from_i8() {
        assert_eq!(TernarySignal::from_i8(-5), TernarySignal::Decrease);
        assert_eq!(TernarySignal::from_i8(-1), TernarySignal::Decrease);
        assert_eq!(TernarySignal::from_i8(0), TernarySignal::Maintain);
        assert_eq!(TernarySignal::from_i8(1), TernarySignal::Increase);
        assert_eq!(TernarySignal::from_i8(42), TernarySignal::Increase);
    }

    #[test]
    fn test_cycle_record_productivity() {
        let record = CycleRecord {
            timestamp: 1000,
            gamma_spent: 0.7,
            eta_spent: 0.3,
            output_quality: 0.8,
            output_quantity: 5.0,
            exploration_yield: 0.4,
        };
        // productivity = quality * quantity / total_spent = 0.8 * 5.0 / 1.0 = 4.0
        assert!((record.productivity() - 4.0).abs() < 1e-9);
        // exploration_roi = yield (already 0-1)
        assert!((record.exploration_roi() - 0.4).abs() < 1e-9);
    }

    #[test]
    fn test_record_cycle_updates_metrics() {
        let mut h = HarnessState::new();
        let initial_count = h.performance.cycle_count;

        // Record multiple cycles with high exploration yield to drive ROI up
        for i in 0..10 {
            let record = CycleRecord {
                timestamp: 1000 + i,
                gamma_spent: 0.5,
                eta_spent: 0.5,
                output_quality: 0.95,
                output_quantity: 3.0,
                exploration_yield: 0.95,
            };
            let signal = h.record_cycle(record);
            if i == 9 {
                // After many cycles with high yield (0.95 > 0.6) and quality, signal Decrease
                assert_eq!(signal, TernarySignal::Decrease);
            }
        }
        assert_eq!(h.performance.cycle_count, initial_count + 10);
        assert_eq!(h.history.len(), 10);
    }

    #[test]
    fn test_low_quality_inverts_signal() {
        let mut h = HarnessState::new();
        // Start with high gamma (0.7) and force quality below 0.5
        // by recording cycles with terrible output quality
        for i in 0..15 {
            let record = CycleRecord {
                timestamp: 1000 + i,
                gamma_spent: h.gamma,
                eta_spent: h.eta,
                output_quality: 0.1, // Very low quality
                output_quantity: 1.0,
                exploration_yield: 0.1, // Low ROI
            };
            h.record_cycle(record);
        }

        // With low quality (< 0.5) and high gamma fraction, the signal should
        // invert to Decrease (back off exploitation), not increase it
        let signal = h.compute_signal();
        assert_eq!(signal, TernarySignal::Decrease, "Low quality + high gamma should decrease exploitation");
        assert!(h.verify_conservation());
    }
        let mut h = HarnessState::new();

        // Simulate cycles with low exploration yield → should converge to more exploitation
        for i in 0..20 {
            let record = CycleRecord {
                timestamp: 1000 + i,
                gamma_spent: h.gamma,
                eta_spent: h.eta,
                output_quality: 0.8,
                output_quantity: 4.0,
                exploration_yield: 0.1, // low exploration yield
            };
            h.record_cycle(record);
        }

        // After many low-yield exploration cycles, gamma should be high
        assert!(h.gamma_fraction() > 0.6, "Should converge toward exploitation");
        assert!(h.verify_conservation());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut h = HarnessState::new();
        let record = CycleRecord {
            timestamp: 1000,
            gamma_spent: 0.7,
            eta_spent: 0.3,
            output_quality: 0.8,
            output_quantity: 3.0,
            exploration_yield: 0.5,
        };
        h.record_cycle(record);

        let json = h.to_json().unwrap();
        let h2: HarnessState = HarnessState::from_json(&json).unwrap();

        assert!((h.gamma - h2.gamma).abs() < 1e-9);
        assert!((h.eta - h2.eta).abs() < 1e-9);
        assert_eq!(h.history.len(), h2.history.len());
        assert_eq!(h.performance.cycle_count, h2.performance.cycle_count);
    }

    #[test]
    fn test_external_feedback() {
        let mut h = HarnessState::new();
        let initial_gamma = h.gamma;

        // Positive feedback → more exploration (gamma decreases)
        h.apply_feedback(0.5);
        assert!(h.gamma < initial_gamma);

        // Negative feedback → more exploitation
        h.apply_feedback(-0.5);
        assert!(h.verify_conservation());
    }

    #[test]
    fn test_rolling_productivity() {
        let mut h = HarnessState::new();
        for i in 0..10 {
            let record = CycleRecord {
                timestamp: 1000 + i,
                gamma_spent: 0.7,
                eta_spent: 0.3,
                output_quality: 0.5,
                output_quantity: 2.0,
                exploration_yield: 0.3,
            };
            h.record_cycle(record);
        }
        // productivity = 0.5 * 2.0 / 1.0 = 1.0
        let rp = h.rolling_productivity(5);
        assert!((rp - 1.0).abs() < 1e-9, "Rolling productivity should be 1.0, got {}", rp);
    }

    #[test]
    fn test_allocation_endpoint_data() {
        let h = HarnessState::new();
        let alloc = h.allocation();
        assert!((alloc.gamma + alloc.eta - h.capacity).abs() < 1e-9);
        assert_eq!(alloc.gamma_budget, alloc.gamma);
        assert_eq!(alloc.eta_budget, alloc.eta);
    }

    #[test]
    fn test_zero_capacity_recovery() {
        let mut h = HarnessState::new();
        h.capacity = 0.0;
        h.gamma = 0.0;
        h.eta = 0.0;
        h.enforce_conservation();
        assert_eq!(h.capacity, DEFAULT_CAPACITY);
        assert!(h.verify_conservation());
    }

    #[test]
    fn test_reset() {
        let mut h = HarnessState::new();
        let record = CycleRecord {
            timestamp: 1000,
            gamma_spent: 0.5,
            eta_spent: 0.5,
            output_quality: 0.9,
            output_quantity: 10.0,
            exploration_yield: 0.8,
        };
        h.record_cycle(record);
        assert!(!h.history.is_empty());

        h.reset();
        assert!(h.history.is_empty());
        assert_eq!(h.performance.cycle_count, 0);
        assert!(h.verify_conservation());
    }
}
