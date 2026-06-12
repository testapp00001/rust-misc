//! # Exercise: Clock Drift Model
//!
//! ## Theory
//!
//! Real clocks do not tick at exactly the correct rate. Crystal oscillators,
//! the most common timekeeping mechanism, have a "drift rate" measured in
//! parts-per-million (ppm). A drift rate of +10 ppm means the clock gains
//! 10 microseconds per second, or about 0.864 seconds per day.
//!
//! Typical crystal oscillator drift rates:
//!   - Standard quartz: +/-10 to +/-100 ppm
//!   - Temperature-compensated (TCXO): +/-1 to +/-5 ppm
//!   - Oven-controlled (OCXO): +/-0.01 to +/-0.1 ppm
//!   - Atomic clocks: ~10^-12 ppm (effectively zero for most purposes)
//!
//! The drift model is:
//!   C(t) = C(t0) + (t - t0) + drift_rate * (t - t0) / 1_000_000
//!
//! where C(t) is the clock reading at real time t, C(t0) is the clock reading
//! at the reference time t0, and drift_rate is in ppm.
//!
//! This linear model is a simplification. Real drift varies with temperature,
//! voltage, and aging. But it captures the dominant behavior for short time
//! scales (hours to days).
//!
//! ## Proof / Intuition
//!
//! Consider a clock with drift rate d ppm. After T seconds:
//!   drift_error = d * T / 1_000_000 seconds
//!
//! For d = 50 ppm (typical) and T = 86400 (one day):
//!   drift_error = 50 * 86400 / 1_000_000 = 4.32 seconds
//!
//! This is why NTP needs to resync periodically: the clock drifts away
//! from true time. The resync interval must be chosen so that the
//! accumulated drift stays within the required accuracy bounds.
//!
//! ## Implementation Task
//!
//! Implement `DriftingClock` with the following:
//!
//! 1. `new(base_time: u64, drift_rate_ppm: i64)` - create a clock with given
//!    initial time and drift rate.
//! 2. `current_time(&self, real_time: u64) -> u64` - compute the clock reading
//!    at the given real time, applying the drift model.
//! 3. `drift_over_duration(&self, duration: u64) -> i64` - compute the total
//!    drift error accumulated over the given duration.
//! 4. `is_fast(&self) -> bool` - true if drift_rate > 0 (clock runs fast).
//! 5. `is_slow(&self) -> bool` - true if drift_rate < 0 (clock runs slow).
//!
//! ## Verification
//!
//! The tests verify:
//! - Drift accumulates linearly with elapsed time.
//! - A zero-drift clock stays perfectly accurate.
//! - The drift sign correctly indicates fast vs slow clocks.
/// A clock that drifts at a constant rate from true time.
///
/// The drift is modeled as a linear function of elapsed time, with the
/// rate specified in parts-per-million (ppm).
#[derive(Debug, Clone)]
pub struct DriftingClock {
    /// Drift rate in parts-per-million. Positive = clock runs fast.
    pub drift_rate: i64,
    /// The clock reading at the base real time.
    pub base_time: u64,
    /// The reference (real) time at which base_time was observed.
    pub base_real_time: u64,
}

impl DriftingClock {
    /// Create a new drifting clock.
    ///
    /// - `base_time`: the clock's reading at the reference moment.
    /// - `drift_rate_ppm`: drift rate in parts-per-million. Positive means
    ///   the clock runs fast (gains time), negative means it runs slow.
    pub fn new(base_time: u64, drift_rate_ppm: i64) -> Self {
        DriftingClock {
            drift_rate: drift_rate_ppm,
            base_time,
            base_real_time: base_time,
        }
    }

    /// Compute the clock reading at a given real time.
    ///
    /// The model is:
    ///   result = base_time + elapsed + drift_rate * elapsed / 1_000_000
    ///
    /// where elapsed = real_time - base_real_time.
    ///
    /// This returns the clock's reading, which may differ from real_time
    /// due to the drift.
    pub fn current_time(&self, real_time: u64) -> u64 {
        let elapsed = (real_time as i64) - (self.base_real_time as i64);
        let drift = self.drift_rate * elapsed / 1_000_000;
        ((self.base_time as i64) + elapsed + drift) as u64
    }

    /// Compute the drift error accumulated over a given duration.
    ///
    /// Returns the difference between the drifted clock and true time
    /// after `duration` seconds, in the same units as the clock.
    ///
    /// Positive return = clock is ahead of true time.
    /// Negative return = clock is behind true time.
    pub fn drift_over_duration(&self, duration: u64) -> i64 {
        self.drift_rate * (duration as i64) / 1_000_000
    }

    /// Returns true if the clock runs fast (positive drift rate).
    pub fn is_fast(&self) -> bool {
        self.drift_rate > 0
    }

    /// Returns true if the clock runs slow (negative drift rate).
    pub fn is_slow(&self) -> bool {
        self.drift_rate < 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_accumulates_over_time() {
        let clock = DriftingClock::new(1000, 50); // 50 ppm fast

        // After 1 second, clock should be at 1001 (1 second elapsed)
        // plus drift: 50 * 1 / 1_000_000 = 0 (integer division rounds down)
        let t = clock.current_time(1001);
        assert_eq!(t, 1001);

        // After 20000 seconds, drift = 50 * 20000 / 1_000_000 = 1
        let t = clock.current_time(3000);
        // elapsed = 2000, base_time = 1000, elapsed + base = 3000
        // drift = 50 * 2000 / 1_000_000 = 0 (still rounds to 0)
        assert_eq!(t, 3000);
    }

    #[test]
    fn test_drift_proportional_to_elapsed() {
        let clock = DriftingClock::new(0, 100); // 100 ppm fast

        // After 10000 seconds: drift = 100 * 10000 / 1_000_000 = 1
        let t = clock.current_time(10000);
        assert_eq!(t, 10001); // 10000 elapsed + 1 drift

        // After 20000 seconds: drift = 100 * 20000 / 1_000_000 = 2
        let t = clock.current_time(20000);
        assert_eq!(t, 20002);

        // After 100000 seconds: drift = 100 * 100000 / 1_000_000 = 10
        let t = clock.current_time(100000);
        assert_eq!(t, 100010);
    }

    #[test]
    fn test_zero_drift_stays_accurate() {
        let clock = DriftingClock::new(5000, 0);

        // With zero drift, clock always matches real time
        assert_eq!(clock.current_time(5000), 5000);
        assert_eq!(clock.current_time(6000), 6000);
        assert_eq!(clock.current_time(10000), 10000);
    }

    #[test]
    fn test_drift_over_duration() {
        let clock = DriftingClock::new(0, 50);

        // 50 ppm over 1_000_000 seconds = 50 seconds of drift
        assert_eq!(clock.drift_over_duration(1_000_000), 50);

        // 50 ppm over 2_000_000 seconds = 100 seconds
        assert_eq!(clock.drift_over_duration(2_000_000), 100);

        // 50 ppm over 100_000 seconds = 5 seconds
        assert_eq!(clock.drift_over_duration(100_000), 5);
    }

    #[test]
    fn test_is_fast_and_is_slow() {
        let fast = DriftingClock::new(0, 50);
        assert!(fast.is_fast());
        assert!(!fast.is_slow());

        let slow = DriftingClock::new(0, -50);
        assert!(!slow.is_fast());
        assert!(slow.is_slow());

        let perfect = DriftingClock::new(0, 0);
        assert!(!perfect.is_fast());
        assert!(!perfect.is_slow());
    }

    #[test]
    fn test_negative_drift() {
        let clock = DriftingClock::new(10000, -50); // 50 ppm slow

        // After 1_000_000 seconds: drift = -50 * 1_000_000 / 1_000_000 = -50
        // clock reading = 10000 + 1_000_000 - 50 = 1_009_950
        let t = clock.current_time(1_010_000);
        // elapsed = 1_000_000, drift = -50
        assert_eq!(t, 1_009_950);
    }

    #[test]
    fn test_large_drift_rate() {
        // Extreme drift: 1000 ppm = 0.1% error
        let clock = DriftingClock::new(0, 1000);

        // After 10000 seconds: drift = 1000 * 10000 / 1_000_000 = 10
        let t = clock.current_time(10000);
        assert_eq!(t, 10010);
    }

    #[test]
    fn test_drift_over_zero_duration() {
        let clock = DriftingClock::new(1000, 50);
        assert_eq!(clock.drift_over_duration(0), 0);
    }

    #[test]
    fn test_drift_is_linear() {
        let clock = DriftingClock::new(0, 100);

        // Drift should be linear: drift(T1 + T2) = drift(T1) + drift(T2)
        let d1 = clock.drift_over_duration(500_000);
        let d2 = clock.drift_over_duration(500_000);
        let d_combined = clock.drift_over_duration(1_000_000);

        assert_eq!(d1 + d2, d_combined);
    }
}
