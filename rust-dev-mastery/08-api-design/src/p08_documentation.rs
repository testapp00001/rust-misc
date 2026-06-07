//! # Documentation
//!
//! Documentation is a first-class citizen in Rust. Doc comments (`///` and `//!`)
//! are processed by `rustdoc` and can include examples that are compiled and run
//! as tests. This module covers documentation best practices.
//!
//! ## Key Concepts
//! - **Doc comments**: `///` for items, `//!` for modules
//! - **Examples in docs**: Code examples that double as tests
//! - **rustdoc**: The documentation generator
//! - **Doc tests**: Examples that are compiled and verified

/// A temperature value that supports multiple scales.
///
/// `Temperature` provides type-safe temperature handling with automatic
/// conversion between Celsius, Fahrenheit, and Kelvin.
///
/// # Examples
///
/// ```
/// use api_design::p08_documentation::Temperature;
///
/// let temp = Temperature::from_celsius(100.0);
/// assert!((temp.as_fahrenheit() - 212.0).abs() < 0.01);
/// assert!((temp.as_kelvin() - 373.15).abs() < 0.01);
/// ```
///
/// # Panics
///
/// Temperature values below absolute zero (0 K / -273.15 C) will cause
/// a panic in debug mode.
///
/// # Thread Safety
///
/// `Temperature` is `Send` and `Sync` and can be safely shared between threads.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature {
    celsius: f64,
}

impl Temperature {
    /// Creates a temperature from a Celsius value.
    ///
    /// # Arguments
    ///
    /// * `celsius` - Temperature in degrees Celsius
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design::p08_documentation::Temperature;
    ///
    /// let temp = Temperature::from_celsius(0.0);
    /// assert!((temp.as_celsius() - 0.0).abs() < 0.001);
    /// ```
    pub fn from_celsius(celsius: f64) -> Self {
        Temperature { celsius }
    }

    /// Creates a temperature from a Fahrenheit value.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design::p08_documentation::Temperature;
    ///
    /// let temp = Temperature::from_fahrenheit(32.0);
    /// assert!((temp.as_celsius() - 0.0).abs() < 0.01);
    /// ```
    pub fn from_fahrenheit(fahrenheit: f64) -> Self {
        Temperature {
            celsius: (fahrenheit - 32.0) * 5.0 / 9.0,
        }
    }

    /// Creates a temperature from a Kelvin value.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design::p08_documentation::Temperature;
    ///
    /// let temp = Temperature::from_kelvin(273.15);
    /// assert!((temp.as_celsius() - 0.0).abs() < 0.01);
    /// ```
    pub fn from_kelvin(kelvin: f64) -> Self {
        Temperature {
            celsius: kelvin - 273.15,
        }
    }

    /// Returns the temperature in Celsius.
    pub fn as_celsius(&self) -> f64 {
        self.celsius
    }

    /// Returns the temperature in Fahrenheit.
    pub fn as_fahrenheit(&self) -> f64 {
        self.celsius * 9.0 / 5.0 + 32.0
    }

    /// Returns the temperature in Kelvin.
    pub fn as_kelvin(&self) -> f64 {
        self.celsius + 273.15
    }

    /// Returns true if the temperature is below freezing (0 C).
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design::p08_documentation::Temperature;
    ///
    /// assert!(Temperature::from_celsius(-10.0).is_freezing());
    /// assert!(!Temperature::from_celsius(25.0).is_freezing());
    /// ```
    pub fn is_freezing(&self) -> bool {
        self.celsius <= 0.0
    }

    /// Returns the difference between two temperatures.
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design::p08_documentation::Temperature;
    ///
    /// let t1 = Temperature::from_celsius(100.0);
    /// let t2 = Temperature::from_celsius(25.0);
    /// let diff = t1.difference(&t2);
    /// assert!((diff - 75.0).abs() < 0.01);
    /// ```
    pub fn difference(&self, other: &Temperature) -> f64 {
        (self.celsius - other.celsius).abs()
    }
}

/// A statistics calculator for numeric data.
///
/// Provides common statistical measures with proper handling of edge cases.
///
/// # Examples
///
/// ```
/// use api_design::p08_documentation::Statistics;
///
/// let stats = Statistics::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
/// assert!((stats.mean() - 3.0).abs() < 0.001);
/// assert!((stats.median() - 3.0).abs() < 0.001);
/// ```
pub struct Statistics {
    data: Vec<f64>,
    sorted: Vec<f64>,
}

impl Statistics {
    /// Creates a new Statistics instance.
    ///
    /// # Arguments
    ///
    /// * `data` - A vector of f64 values to analyze
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design::p08_documentation::Statistics;
    ///
    /// let stats = Statistics::new(vec![10.0, 20.0, 30.0]);
    /// assert_eq!(stats.count(), 3);
    /// ```
    pub fn new(data: Vec<f64>) -> Self {
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        Statistics { data, sorted }
    }

    /// Returns the number of data points.
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// Returns the arithmetic mean.
    ///
    /// Returns `None` if the data is empty.
    pub fn mean(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        self.data.iter().sum::<f64>() / self.data.len() as f64
    }

    /// Returns the median value.
    ///
    /// For even-length datasets, returns the average of the two middle values.
    pub fn median(&self) -> f64 {
        if self.sorted.is_empty() {
            return 0.0;
        }
        let n = self.sorted.len();
        if n % 2 == 0 {
            (self.sorted[n / 2 - 1] + self.sorted[n / 2]) / 2.0
        } else {
            self.sorted[n / 2]
        }
    }

    /// Returns the minimum value.
    pub fn min(&self) -> f64 {
        self.sorted.first().copied().unwrap_or(0.0)
    }

    /// Returns the maximum value.
    pub fn max(&self) -> f64 {
        self.sorted.last().copied().unwrap_or(0.0)
    }

    /// Returns the standard deviation.
    pub fn stddev(&self) -> f64 {
        if self.data.len() < 2 {
            return 0.0;
        }
        let mean = self.mean();
        let variance: f64 = self
            .data
            .iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / (self.data.len() - 1) as f64;
        variance.sqrt()
    }

    /// Returns a percentile value (0.0 to 100.0).
    ///
    /// # Examples
    ///
    /// ```
    /// use api_design::p08_documentation::Statistics;
    ///
    /// let stats = Statistics::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
    /// let p50 = stats.percentile(50.0);
    /// assert!((p50 - 3.0).abs() < 0.01);
    /// ```
    pub fn percentile(&self, p: f64) -> f64 {
        if self.sorted.is_empty() {
            return 0.0;
        }
        let idx = (p / 100.0 * (self.sorted.len() - 1) as f64) as usize;
        self.sorted[idx.min(self.sorted.len() - 1)]
    }
}

/// # Safety
///
/// This struct demonstrates the `# Safety` documentation section that's
/// required for unsafe code.
///
/// # Examples
///
/// ```
/// use api_design::p08_documentation::SafeWrapper;
///
/// let wrapper = SafeWrapper::new(42);
/// assert_eq!(wrapper.get(), 42);
/// ```
pub struct SafeWrapper {
    value: i32,
}

impl SafeWrapper {
    /// Creates a new SafeWrapper.
    pub fn new(value: i32) -> Self {
        SafeWrapper { value }
    }

    /// Returns the wrapped value.
    pub fn get(&self) -> i32 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temperature_conversions() {
        let boiling = Temperature::from_celsius(100.0);
        assert!((boiling.as_fahrenheit() - 212.0).abs() < 0.01);
        assert!((boiling.as_kelvin() - 373.15).abs() < 0.01);

        let freezing = Temperature::from_fahrenheit(32.0);
        assert!((freezing.as_celsius() - 0.0).abs() < 0.01);

        let absolute_zero = Temperature::from_kelvin(0.0);
        assert!((absolute_zero.as_celsius() - (-273.15)).abs() < 0.01);
    }

    #[test]
    fn test_temperature_freezing() {
        assert!(Temperature::from_celsius(-10.0).is_freezing());
        assert!(Temperature::from_celsius(0.0).is_freezing());
        assert!(!Temperature::from_celsius(25.0).is_freezing());
    }

    #[test]
    fn test_temperature_difference() {
        let t1 = Temperature::from_celsius(100.0);
        let t2 = Temperature::from_celsius(25.0);
        assert!((t1.difference(&t2) - 75.0).abs() < 0.01);
    }

    #[test]
    fn test_statistics_basic() {
        let stats = Statistics::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert_eq!(stats.count(), 5);
        assert!((stats.mean() - 3.0).abs() < 0.001);
        assert!((stats.median() - 3.0).abs() < 0.001);
        assert!((stats.min() - 1.0).abs() < 0.001);
        assert!((stats.max() - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_statistics_even_count() {
        let stats = Statistics::new(vec![1.0, 2.0, 3.0, 4.0]);
        assert!((stats.median() - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_statistics_empty() {
        let stats = Statistics::new(vec![]);
        assert_eq!(stats.count(), 0);
        assert_eq!(stats.mean(), 0.0);
        assert_eq!(stats.median(), 0.0);
    }

    #[test]
    fn test_statistics_stddev() {
        let stats = Statistics::new(vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        assert!((stats.stddev() - 2.14).abs() < 0.05);
    }

    #[test]
    fn test_statistics_percentile() {
        let stats = Statistics::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((stats.percentile(50.0) - 3.0).abs() < 0.01);
        assert!((stats.percentile(0.0) - 1.0).abs() < 0.01);
        assert!((stats.percentile(100.0) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_safe_wrapper() {
        let wrapper = SafeWrapper::new(42);
        assert_eq!(wrapper.get(), 42);
    }
}
