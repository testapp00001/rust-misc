//! # Exercise: TrueTime-style Interval Clock
//!
//! ## Theory
//!
//! Google's TrueTime, used in Spanner, exposes clock uncertainty directly.
//! Instead of returning a single point in time, TrueTime returns an interval
//! [earliest, latest] such that the true current time is guaranteed to lie
//! within that interval with high probability (typically 99.999%).
//!
//! The uncertainty comes from:
//!   1. GPS signal uncertainty (atomic reference, very small)
//!   2. Crystal oscillator drift between GPS/atomic sync points
//!   3. Network delay to the time masters
//!
//! TrueTime maintains a set of "time masters" - machines with atomic clocks
//! and GPS receivers. A local daemon polls these masters and maintains an
//! uncertainty interval that accounts for:
//!   - The spread of reported times across masters
//!   - The worst-case drift since the last successful poll
//!
//! Typical uncertainty is 1-7ms, with 4ms being common in healthy clusters.
//! During GPS outages, uncertainty grows at the crystal drift rate (~200 ppm
//! for server-grade hardware), reaching ~7ms after about 30 seconds.
//!
//! ## Proof / Intuition
//!
//! Let t_true be the actual current time. TrueTime guarantees:
//!   earliest <= t_true <= latest
//!
//! where latest - earliest = 2 * uncertainty_ms.
//!
//! This is possible because:
//!   - Atomic clocks drift ~10^-12 per second
//!   - GPS provides absolute time to ~100ns
//!   - The daemon interpolates between sync points and bounds the error
//!
//! The key insight is that uncertainty is *known* - you can make decisions
//! based on it. This is fundamentally different from NTP, where the error
//! is unknown.
//!
//! ## Implementation Task
//!
//! Implement `TrueTime` with the following:
//!
//! 1. `new(base_time: u64)` - create with default 7ms uncertainty.
//! 2. `new_with_uncertainty(base_time: u64, uncertainty_ms: u64)` -
//!    create with custom uncertainty.
//! 3. `now(&self, real_time: u64) -> (u64, u64)` - return (earliest, latest)
//!    interval around real_time.
//! 4. `contains(&self, real_time: u64, interval: (u64, u64)) -> bool` -
//!    check if the true time at real_time falls within the given interval.
//! 5. `uncertainty(&self) -> u64` - return the uncertainty bound.
//!
//! ## Verification
//!
//! The tests verify:
//! - The interval always contains the real time.
//! - The interval width equals 2 * uncertainty.
//! - The uncertainty bounds are respected.

/// A TrueTime-style clock that exposes uncertainty as an interval.
///
/// Instead of claiming to know the exact current time, TrueTime returns
/// an interval [earliest, latest] and guarantees that the true time
/// lies within that interval.
#[derive(Debug, Clone)]
pub struct TrueTime {
    /// Uncertainty in milliseconds on each side of the interval.
    /// The total interval width is 2 * uncertainty_ms.
    pub uncertainty_ms: u64,
    /// Base time for the clock (used as reference).
    pub base_time: u64,
}

impl TrueTime {
    /// Create a new TrueTime with default uncertainty (7ms).
    pub fn new(base_time: u64) -> Self {
        TrueTime {
            uncertainty_ms: 7,
            base_time,
        }
    }

    /// Create a new TrueTime with custom uncertainty.
    pub fn new_with_uncertainty(base_time: u64, uncertainty_ms: u64) -> Self {
        TrueTime {
            uncertainty_ms,
            base_time,
        }
    }

    /// Return the uncertainty interval around the given real time.
    ///
    /// Returns (earliest, latest) where:
    ///   earliest = real_time - uncertainty_ms (saturating at 0)
    ///   latest = real_time + uncertainty_ms
    ///
    /// The true time is guaranteed to lie within this interval.
    pub fn now(&self, real_time: u64) -> (u64, u64) {
        let earliest = real_time.saturating_sub(self.uncertainty_ms);
        let latest = real_time + self.uncertainty_ms;
        (earliest, latest)
    }

    /// Check if the true time (at real_time) falls within the given interval.
    ///
    /// This is useful for implementing commit wait: a transaction can
    /// proceed when its timestamp interval does not overlap with the
    /// "uncertainty window" of any future timestamp.
    pub fn contains(&self, real_time: u64, interval: (u64, u64)) -> bool {
        let (earliest, latest) = interval;
        real_time >= earliest && real_time <= latest
    }

    /// Return the uncertainty bound in milliseconds.
    pub fn uncertainty(&self) -> u64 {
        self.uncertainty_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_contains_real_time() {
        let tt = TrueTime::new_with_uncertainty(1000, 5);

        // Real time at 1000, interval should be [995, 1005]
        let (earliest, latest) = tt.now(1000);
        assert!(tt.contains(1000, (earliest, latest)));
        assert_eq!(earliest, 995);
        assert_eq!(latest, 1005);
    }

    #[test]
    fn test_interval_width() {
        let tt = TrueTime::new_with_uncertainty(0, 7);

        let (earliest, latest) = tt.now(1000);
        let width = latest - earliest;
        assert_eq!(width, 14); // 2 * uncertainty_ms
    }

    #[test]
    fn test_default_uncertainty() {
        let tt = TrueTime::new(0);
        assert_eq!(tt.uncertainty(), 7);

        let (earliest, latest) = tt.now(100);
        assert_eq!(earliest, 93);
        assert_eq!(latest, 107);
    }

    #[test]
    fn test_custom_uncertainty() {
        let tt = TrueTime::new_with_uncertainty(0, 3);

        let (earliest, latest) = tt.now(100);
        assert_eq!(earliest, 97);
        assert_eq!(latest, 103);
        assert_eq!(tt.uncertainty(), 3);
    }

    #[test]
    fn test_saturating_at_zero() {
        let tt = TrueTime::new_with_uncertainty(0, 10);

        // Real time = 5, uncertainty = 10
        // earliest = 5 - 10 = saturate to 0
        let (earliest, _latest) = tt.now(5);
        assert_eq!(earliest, 0);
    }

    #[test]
    fn test_contains_within_bounds() {
        let tt = TrueTime::new_with_uncertainty(0, 5);

        let (earliest, latest) = tt.now(100);

        // Values inside the interval
        assert!(tt.contains(98, (earliest, latest)));
        assert!(tt.contains(100, (earliest, latest)));
        assert!(tt.contains(102, (earliest, latest)));
        assert!(tt.contains(earliest, (earliest, latest)));
        assert!(tt.contains(latest, (earliest, latest)));
    }

    #[test]
    fn test_contains_outside_bounds() {
        let tt = TrueTime::new_with_uncertainty(0, 5);

        let (earliest, latest) = tt.now(100);

        // Values outside the interval
        assert!(!tt.contains(90, (earliest, latest)));
        assert!(!tt.contains(110, (earliest, latest)));
        assert!(!tt.contains(0, (earliest, latest)));
    }

    #[test]
    fn test_multiple_intervals_contain_real_times() {
        let tt = TrueTime::new_with_uncertainty(0, 3);

        // Check that intervals contain the real time for various values
        for real_time in [10, 50, 100, 500, 1000, 9999] {
            let (earliest, latest) = tt.now(real_time);
            assert!(
                tt.contains(real_time, (earliest, latest)),
                "Interval [{}, {}] should contain real_time {}",
                earliest,
                latest,
                real_time
            );
        }
    }

    #[test]
    fn test_zero_uncertainty() {
        let tt = TrueTime::new_with_uncertainty(0, 0);

        let (earliest, latest) = tt.now(100);
        assert_eq!(earliest, 100);
        assert_eq!(latest, 100);
        assert_eq!(latest - earliest, 0);
    }

    #[test]
    fn test_large_uncertainty() {
        let tt = TrueTime::new_with_uncertainty(0, 1000);

        let (earliest, latest) = tt.now(5000);
        assert_eq!(earliest, 4000);
        assert_eq!(latest, 6000);
        assert_eq!(latest - earliest, 2000);
    }
}
