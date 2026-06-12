//! # Exercise: Clock Drift Simulation
//!
//! ## Theory
//!
//! In a real distributed system, each machine has its own physical clock with a
//! unique drift rate. Even with NTP synchronization, clocks diverge between
//! synchronizations. This exercise simulates multiple machines with different
//! drift rates and measures how their clocks diverge over time.
//!
//! ## Proof / Intuition
//!
//! The maximum offset between any two clocks in a system with n machines is:
//!
//! `max_offset = max(|drift_i - drift_j|) * elapsed_time`
//!
//! For a system with 100 ppm drift variation and 1 hour between NTP syncs:
//!
//! `max_offset = 200e-6 * 3600 = 720 ms`
//!
//! This means events within 720ms of each other could be ordered differently
//! by different machines.
//!
//! ## Implementation Task
//!
//! - Create multiple `PhysicalClock` instances with different drift rates
//! - Simulate time passing and measure maximum offset
//! - Show that even small drift rates cause significant divergence
//!
//! ## Verification
//!
//! - Verify clocks diverge over time
//! - Verify drift is proportional to elapsed time
//! - Verify maximum offset matches theoretical prediction

use super::p01_physical_clocks::PhysicalClock;

/// A simulated machine with a physical clock.
#[derive(Debug, Clone)]
pub struct SimulatedMachine {
    /// Machine identifier.
    pub id: usize,
    /// The machine's physical clock.
    pub clock: PhysicalClock,
}

impl SimulatedMachine {
    /// Create a new simulated machine with a given drift rate.
    pub fn new(id: usize, drift_rate_ppm: i64) -> Self {
        Self {
            id,
            clock: PhysicalClock::new(0, drift_rate_ppm),
        }
    }
}

/// Simulate a distributed system with multiple machines.
pub struct ClockDriftSimulation {
    /// All machines in the simulation.
    machines: Vec<SimulatedMachine>,
    /// Current real elapsed time in milliseconds.
    elapsed_ms: u64,
}

impl ClockDriftSimulation {
    /// Create a new simulation with the given drift rates.
    pub fn new(drift_rates_ppm: &[i64]) -> Self {
        let machines = drift_rates_ppm
            .iter()
            .enumerate()
            .map(|(i, &rate)| SimulatedMachine::new(i, rate))
            .collect();

        Self {
            machines,
            elapsed_ms: 0,
        }
    }

    /// Advance all clocks by the given real time.
    pub fn advance(&mut self, elapsed_ms: u64) {
        self.elapsed_ms += elapsed_ms;
        for machine in &mut self.machines {
            machine.clock.tick(elapsed_ms);
        }
    }

    /// Get the maximum clock offset between any two machines.
    pub fn max_offset(&self) -> i64 {
        let times: Vec<i64> = self.machines.iter().map(|m| m.clock.now() as i64).collect();
        let min_time = times.iter().min().unwrap();
        let max_time = times.iter().max().unwrap();
        max_time - min_time
    }

    /// Get the minimum clock offset between any two machines.
    pub fn min_offset(&self) -> i64 {
        let times: Vec<i64> = self.machines.iter().map(|m| m.clock.now() as i64).collect();
        let mut min_offset = i64::MAX;
        for i in 0..times.len() {
            for j in (i + 1)..times.len() {
                let offset = (times[i] - times[j]).unsigned_abs() as i64;
                min_offset = min_offset.min(offset);
            }
        }
        min_offset
    }

    /// Get the current elapsed real time.
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed_ms
    }

    /// Get the number of machines.
    pub fn machine_count(&self) -> usize {
        self.machines.len()
    }

    /// Get a reference to all machines.
    pub fn machines(&self) -> &[SimulatedMachine] {
        &self.machines
    }

    /// Calculate the theoretical maximum drift given the drift rates and elapsed time.
    pub fn theoretical_max_offset(&self) -> f64 {
        let rates: Vec<i64> = self.machines.iter().map(|m| m.clock.drift_rate()).collect();
        let mut max_drift = 0i64;
        for i in 0..rates.len() {
            for j in (i + 1)..rates.len() {
                let diff = (rates[i] - rates[j]).unsigned_abs() as i64;
                max_drift = max_drift.max(diff);
            }
        }
        max_drift as f64 * self.elapsed_ms as f64 / 1_000_000.0
    }
}

/// Calculate how long it takes for clocks to diverge by a given threshold.
pub fn time_to_diverge(drift_rates_ppm: &[i64], threshold_ms: i64) -> u64 {
    let mut max_drift = 0i64;
    for i in 0..drift_rates_ppm.len() {
        for j in (i + 1)..drift_rates_ppm.len() {
            let diff = (drift_rates_ppm[i] - drift_rates_ppm[j]).unsigned_abs() as i64;
            max_drift = max_drift.max(diff);
        }
    }

    if max_drift == 0 {
        return u64::MAX; // Never diverges
    }

    // threshold_ms = max_drift * elapsed / 1_000_000
    // elapsed = threshold_ms * 1_000_000 / max_drift
    (threshold_ms as u64 * 1_000_000) / max_drift as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clocks_diverge_over_time() {
        let mut sim = ClockDriftSimulation::new(&[-100, 0, 100]);

        // Initially no divergence
        assert_eq!(sim.max_offset(), 0);

        // After 1000 seconds, diverge
        sim.advance(1_000_000);
        assert!(
            sim.max_offset() > 0,
            "clocks should have diverged after 1000 seconds"
        );
    }

    #[test]
    fn drift_proportional_to_time() {
        let mut sim1 = ClockDriftSimulation::new(&[-100, 100]);
        let mut sim2 = ClockDriftSimulation::new(&[-100, 100]);

        sim1.advance(1_000_000); // 1000 seconds
        sim2.advance(2_000_000); // 2000 seconds

        // Offset should roughly double
        let offset1 = sim1.max_offset();
        let offset2 = sim2.max_offset();
        let ratio = offset2 as f64 / offset1 as f64;

        assert!(
            ratio > 1.8 && ratio < 2.2,
            "offset should be roughly proportional to time, ratio is {ratio}"
        );
    }

    #[test]
    fn max_offset_matches_theoretical() {
        let mut sim = ClockDriftSimulation::new(&[-50, 0, 50, 100]);
        sim.advance(1_000_000); // 1000 seconds

        let actual = sim.max_offset() as f64;
        let theoretical = sim.theoretical_max_offset();

        // Allow 10% tolerance due to integer rounding
        let tolerance = theoretical * 0.1;
        assert!(
            (actual - theoretical).abs() < tolerance,
            "actual offset {actual} should be close to theoretical {theoretical}"
        );
    }

    #[test]
    fn zero_drift_no_divergence() {
        let mut sim = ClockDriftSimulation::new(&[0, 0, 0]);
        sim.advance(10_000_000); // 10,000 seconds

        assert_eq!(sim.max_offset(), 0);
    }

    #[test]
    fn time_to_diverge_formula() {
        // With 200 ppm max drift, 100ms threshold:
        // time = 100ms * 1_000_000 / 200 = 500_000 ms = 500 seconds
        let time = time_to_diverge(&[-100, 100], 100);
        assert!(
            time > 490_000 && time < 510_000,
            "expected ~500_000ms, got {time}ms"
        );
    }

    #[test]
    fn many_machines_diverge() {
        let drift_rates: Vec<i64> = (-5..=5).collect(); // 11 machines
        let mut sim = ClockDriftSimulation::new(&drift_rates);
        sim.advance(3_600_000); // 1 hour

        assert!(
            sim.max_offset() > 0,
            "multiple machines should diverge over 1 hour"
        );
    }
}
