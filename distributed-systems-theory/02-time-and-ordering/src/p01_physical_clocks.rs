//! # Exercise: Physical Clocks
//!
//! ## Theory
//!
//! Physical clocks in distributed systems are not perfect. Each clock has a **drift
//! rate** measured in parts per million (ppm). A drift rate of 100 ppm means the
//! clock gains or loses 100 microseconds per second.
//!
//! Over time, clocks with different drift rates diverge, creating inconsistencies
//! in event ordering. This is why distributed systems cannot rely on physical time
//! alone for ordering.
//!
//! ## Proof / Intuition
//!
//! If clock A drifts at +10 ppm and clock B drifts at -10 ppm, their relative
//! drift is 20 ppm. Over 1 hour (3600 seconds):
//!
//! `offset = 20 ppm * 3600 s = 20e-6 * 3600 = 0.072 seconds = 72 ms`
//!
//! Over 24 hours:
//!
//! `offset = 20e-6 * 86400 = 1.728 seconds`
//!
//! This is significant -- messages could appear to arrive before they were sent.
//!
//! ## Implementation Task
//!
//! Implement `PhysicalClock` with:
//! - Configurable drift rate (ppm)
//! - `tick(elapsed_ms)` to advance the clock
//! - `now()` to get the current time
//!
//! ## Verification
//!
//! - Verify drift accumulates over time
//! - Verify different drift rates produce different times

/// A physical clock with configurable drift rate.
///
/// The drift rate is in parts per million (ppm). Positive drift means the clock
/// runs fast (gains time); negative drift means it runs slow (loses time).
#[derive(Debug, Clone)]
pub struct PhysicalClock {
    /// Current logical time in milliseconds.
    time_ms: u64,
    /// Drift rate in parts per million (ppm).
    /// Positive = fast, negative = slow.
    drift_rate_ppm: i64,
    /// Accumulated fractional milliseconds from drift (for precision).
    accumulated_drift: f64,
}

impl PhysicalClock {
    /// Create a new physical clock with the given drift rate.
    ///
    /// # Arguments
    /// * `initial_time_ms` - Starting time in milliseconds
    /// * `drift_rate_ppm` - Drift rate in parts per million (positive = fast, negative = slow)
    pub fn new(initial_time_ms: u64, drift_rate_ppm: i64) -> Self {
        Self {
            time_ms: initial_time_ms,
            drift_rate_ppm,
            accumulated_drift: 0.0,
        }
    }

    /// Advance the clock by the given elapsed real time.
    ///
    /// The clock advances by `elapsed_ms` plus any drift accumulated during
    /// that period.
    pub fn tick(&mut self, elapsed_ms: u64) {
        // Drift = drift_rate * elapsed_time
        // drift_rate is in ppm, so: drift_ms = elapsed_ms * drift_rate / 1_000_000
        let drift = elapsed_ms as f64 * self.drift_rate_ppm as f64 / 1_000_000.0;
        self.accumulated_drift += drift;

        // Apply accumulated drift (take the integer part)
        let drift_to_apply = self.accumulated_drift.floor() as i64;
        self.accumulated_drift -= drift_to_apply as f64;

        if drift_to_apply >= 0 {
            self.time_ms += (elapsed_ms as i64 + drift_to_apply) as u64;
        } else {
            let total = elapsed_ms as i64 + drift_to_apply;
            if total >= 0 {
                self.time_ms += total as u64;
            }
            // If total < 0, clock doesn't go backward (monotonic guarantee)
        }
    }

    /// Get the current clock time in milliseconds.
    pub fn now(&self) -> u64 {
        self.time_ms
    }

    /// Get the drift rate in ppm.
    pub fn drift_rate(&self) -> i64 {
        self.drift_rate_ppm
    }

    /// Calculate the expected drift over a given duration.
    pub fn expected_drift_ms(&self, elapsed_ms: u64) -> f64 {
        elapsed_ms as f64 * self.drift_rate_ppm as f64 / 1_000_000.0
    }
}

/// Calculate the offset between two clocks after a given elapsed time.
pub fn clock_offset(clock_a: &PhysicalClock, clock_b: &PhysicalClock) -> i64 {
    clock_a.now() as i64 - clock_b.now() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_drift_matches_real_time() {
        let mut clock = PhysicalClock::new(0, 0);
        clock.tick(1000);
        assert_eq!(clock.now(), 1000);
        clock.tick(500);
        assert_eq!(clock.now(), 1500);
    }

    #[test]
    fn drift_accumulates_over_time() {
        // +100 ppm means clock gains 100 us per second
        let mut clock = PhysicalClock::new(0, 100);
        let real_time = 1_000_000; // 1000 seconds

        clock.tick(real_time);

        // Expected drift: 1_000_000 * 100 / 1_000_000 = 100 ms
        let expected = real_time as u64 + 100;
        let actual = clock.now();
        assert!(
            (actual as i64 - expected as i64).unsigned_abs() < 10,
            "clock should have drifted by ~100ms, expected {expected}, got {actual}"
        );
    }

    #[test]
    fn different_drift_rates_diverge() {
        let mut fast_clock = PhysicalClock::new(0, 100); // +100 ppm
        let mut slow_clock = PhysicalClock::new(0, -100); // -100 ppm

        let elapsed = 1_000_000; // 1000 seconds
        fast_clock.tick(elapsed);
        slow_clock.tick(elapsed);

        let offset = clock_offset(&fast_clock, &slow_clock);

        // Expected: fast is +100ms ahead, slow is -100ms behind -> 200ms apart
        assert!(
            offset > 150,
            "clocks should have diverged by ~200ms, offset is {offset}ms"
        );
    }

    #[test]
    fn negative_drift_clock_runs_slow() {
        let mut clock = PhysicalClock::new(0, -100); // -100 ppm

        clock.tick(1_000_000); // 1000 seconds

        // Clock should be behind real time by ~100ms
        let expected = 1_000_000u64 - 100;
        let actual = clock.now();
        assert!(
            (actual as i64 - expected as i64).unsigned_abs() < 10,
            "slow clock should be behind, expected ~{expected}, got {actual}"
        );
    }

    #[test]
    fn expected_drift_formula() {
        let clock = PhysicalClock::new(0, 250);
        assert!((clock.expected_drift_ms(1_000_000) - 250.0).abs() < 0.01);
        assert!((clock.expected_drift_ms(2_000_000) - 500.0).abs() < 0.01);
        assert!((clock.expected_drift_ms(100_000) - 25.0).abs() < 0.01);
    }
}
