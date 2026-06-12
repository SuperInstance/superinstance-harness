use superinstance_harness::{CycleRecord, HarnessState};

fn main() {
    let mut harness = HarnessState::new();
    println!("=== superinstance-harness ===\n");
    println!("Initial state: γ={:.3} η={:.3} C={:.3}", harness.gamma, harness.eta, harness.capacity);
    println!("Conservation: γ + η = {:.3} = C? {}\n", harness.gamma + harness.eta, harness.verify_conservation());

    // Simulate 10 cycles
    for i in 0..10 {
        let record = CycleRecord {
            timestamp: 1000 + i,
            gamma_spent: harness.gamma,
            eta_spent: harness.eta,
            output_quality: 0.7 + (i as f64 * 0.02),
            output_quantity: 3.0,
            exploration_yield: 0.3 + (i as f64 * 0.03),
        };

        let signal = harness.record_cycle(record);
        let alloc = harness.allocation();
        println!(
            "Cycle {:2}: γ={:.3} η={:.3} | quality_ewma={:.3} roi={:.3} | signal={:?} | regret={:.4}",
            i + 1,
            alloc.gamma,
            alloc.eta,
            harness.performance.ewma_quality,
            harness.performance.exploration_roi,
            signal,
            harness.performance.regret,
        );
    }

    println!("\nFinal state:");
    println!("  γ fraction: {:.1}%", harness.gamma_fraction() * 100.0);
    println!("  η fraction: {:.1}%", harness.eta_fraction() * 100.0);
    println!("  Rolling productivity (5): {:.3}", harness.rolling_productivity(5));
    println!("  Total regret: {:.4}", harness.performance.regret);
    println!("  Conservation holds: {}", harness.verify_conservation());
}
