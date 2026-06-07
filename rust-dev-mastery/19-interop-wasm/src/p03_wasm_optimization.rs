//! # WASM Optimization
//!
//! WebAssembly binary size and performance optimization is critical for
//! web applications. This module covers strategies for reducing WASM size
//! and improving execution speed.
//!
//! ## Size Optimization:
//!
//! | Technique | Impact |
//! |-----------|--------|
//! | `wasm-opt -O3` | 10-30% reduction |
//! | `strip` symbols | 5-15% reduction |
//! | Feature flags | Varies |
//! | `#[cfg]` attributes | Varies |
//! | Thin LTO | 5-10% reduction |
//!
//! ## Performance Optimization:
//!
//! - Avoid allocations in hot paths
//! - Use `&[u8]` instead of `Vec<u8>` where possible
//! - Minimize JS-WASM boundary crossings
//! - Use SIMD when available

/// WASM size optimization configuration.
#[derive(Debug, Clone)]
pub struct WasmOptConfig {
    pub optimization_level: OptLevel,
    pub strip_debug: bool,
    pub strip_symbols: bool,
    pub enable_simd: bool,
    pub enable_bulk_memory: bool,
    pub enable_reference_types: bool,
    pub thin_lto: bool,
}

#[derive(Debug, Clone)]
pub enum OptLevel {
    Debug,
    Size,
    Speed,
    SizeAndSpeed,
}

impl Default for WasmOptConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptLevel::SizeAndSpeed,
            strip_debug: true,
            strip_symbols: true,
            enable_simd: false,
            enable_bulk_memory: true,
            enable_reference_types: true,
            thin_lto: true,
        }
    }
}

impl WasmOptConfig {
    /// Generate cargo build flags.
    pub fn cargo_flags(&self) -> Vec<String> {
        let mut flags = vec!["build".to_string(), "--target".to_string(), "wasm32-unknown-unknown".to_string()];

        match self.optimization_level {
            OptLevel::Debug => flags.push("--dev".to_string()),
            OptLevel::Size | OptLevel::SizeAndSpeed => flags.push("--release".to_string()),
            OptLevel::Speed => flags.push("--release".to_string()),
        }

        flags
    }

    /// Generate wasm-opt flags.
    pub fn wasm_opt_flags(&self) -> Vec<String> {
        let mut flags = Vec::new();

        match self.optimization_level {
            OptLevel::Size => flags.push("-Os".to_string()),
            OptLevel::Speed => flags.push("-O3".to_string()),
            OptLevel::SizeAndSpeed => flags.push("-O".to_string()),
            OptLevel::Debug => {}
        }

        if self.strip_debug {
            flags.push("--strip-debug".to_string());
        }

        if self.strip_symbols {
            flags.push("--strip-producers".to_string());
        }

        flags
    }

    /// Generate wasm-opt command.
    pub fn wasm_opt_command(&self, input: &str, output: &str) -> String {
        let flags = self.wasm_opt_flags().join(" ");
        format!("wasm-opt {} {} -o {}", flags, input, output)
    }
}

/// WASM binary size analyzer.
pub struct WasmSizeAnalyzer {
    sections: Vec<WasmSection>,
}

#[derive(Debug, Clone)]
pub struct WasmSection {
    pub name: String,
    pub size_bytes: usize,
    pub percentage: f64,
}

impl WasmSizeAnalyzer {
    pub fn new() -> Self {
        Self {
            sections: Vec::new(),
        }
    }

    pub fn add_section(&mut self, name: &str, size: usize) {
        self.sections.push(WasmSection {
            name: name.into(),
            size_bytes: size,
            percentage: 0.0,
        });
        self.recalculate();
    }

    fn recalculate(&mut self) {
        let total: usize = self.sections.iter().map(|s| s.size_bytes).sum();
        for section in &mut self.sections {
            section.percentage = if total > 0 {
                (section.size_bytes as f64 / total as f64) * 100.0
            } else {
                0.0
            };
        }
    }

    pub fn total_size(&self) -> usize {
        self.sections.iter().map(|s| s.size_bytes).sum()
    }

    pub fn largest_section(&self) -> Option<&WasmSection> {
        self.sections.iter().max_by_key(|s| s.size_bytes)
    }

    pub fn report(&self) -> String {
        let mut output = String::from("WASM Size Report:\n");
        output.push_str(&format!("Total: {} bytes\n\n", self.total_size()));

        let mut sorted = self.sections.clone();
        sorted.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

        for section in &sorted {
            output.push_str(&format!(
                "  {}: {} bytes ({:.1}%)\n",
                section.name, section.size_bytes, section.percentage
            ));
        }
        output
    }
}

/// WASM performance counter for measuring JS-WASM boundary crossings.
pub struct WasmPerfCounter {
    crossings: u64,
    total_time_ns: u64,
    max_time_ns: u64,
}

impl WasmPerfCounter {
    pub fn new() -> Self {
        Self {
            crossings: 0,
            total_time_ns: 0,
            max_time_ns: 0,
        }
    }

    /// Record a boundary crossing.
    pub fn record_crossing(&mut self, duration_ns: u64) {
        self.crossings += 1;
        self.total_time_ns += duration_ns;
        self.max_time_ns = self.max_time_ns.max(duration_ns);
    }

    pub fn crossings(&self) -> u64 {
        self.crossings
    }

    pub fn average_time_ns(&self) -> f64 {
        if self.crossings == 0 {
            return 0.0;
        }
        self.total_time_ns as f64 / self.crossings as f64
    }

    pub fn max_time_ns(&self) -> u64 {
        self.max_time_ns
    }
}

/// Batch processor to minimize JS-WASM boundary crossings.
pub struct BatchProcessor<T> {
    batch: Vec<T>,
    batch_size: usize,
}

impl<T> BatchProcessor<T> {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch: Vec::with_capacity(batch_size),
            batch_size,
        }
    }

    pub fn add(&mut self, item: T) -> Option<Vec<T>> {
        self.batch.push(item);
        if self.batch.len() >= self.batch_size {
            Some(std::mem::replace(
                &mut self.batch,
                Vec::with_capacity(self.batch_size),
            ))
        } else {
            None
        }
    }

    pub fn flush(&mut self) -> Vec<T> {
        std::mem::replace(&mut self.batch, Vec::with_capacity(self.batch_size))
    }

    pub fn len(&self) -> usize {
        self.batch.len()
    }
}

/// Feature flag configuration for WASM builds.
#[derive(Debug, Clone)]
pub struct WasmFeatures {
    pub simd: bool,
    pub threads: bool,
    pub bulk_memory: bool,
    pub reference_types: bool,
    pub mutable_globals: bool,
    pub sign_extensions: bool,
    pub saturating_float_to_int: bool,
}

impl Default for WasmFeatures {
    fn default() -> Self {
        Self {
            simd: false,
            threads: false,
            bulk_memory: true,
            reference_types: true,
            mutable_globals: true,
            sign_extensions: true,
            saturating_float_to_int: true,
        }
    }
}

impl WasmFeatures {
    /// Generate RUSTFLAGS for feature selection.
    pub fn rustflags(&self) -> Vec<String> {
        let mut flags = Vec::new();
        if self.simd {
            flags.push("-C target-feature=+simd128".into());
        }
        if self.bulk_memory {
            flags.push("-C target-feature=+bulk-memory".into());
        }
        if self.reference_types {
            flags.push("-C target-feature=+reference-types".into());
        }
        flags
    }

    /// Check browser compatibility score (0-100).
    pub fn browser_compatibility(&self) -> u32 {
        let mut score = 100;
        if self.simd { score -= 5; } // Wide but not universal
        if self.threads { score -= 15; } // Requires SharedArrayBuffer
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_opt_config_default() {
        let config = WasmOptConfig::default();
        assert!(config.strip_debug);
        assert!(config.strip_symbols);
        assert!(config.thin_lto);
    }

    #[test]
    fn test_wasm_opt_config_cargo_flags() {
        let config = WasmOptConfig::default();
        let flags = config.cargo_flags();
        assert!(flags.contains(&"wasm32-unknown-unknown".to_string()));
        assert!(flags.contains(&"--release".to_string()));
    }

    #[test]
    fn test_wasm_opt_config_wasm_opt_flags() {
        let config = WasmOptConfig::default();
        let flags = config.wasm_opt_flags();
        assert!(flags.contains(&"--strip-debug".to_string()));
    }

    #[test]
    fn test_wasm_opt_command() {
        let config = WasmOptConfig::default();
        let cmd = config.wasm_opt_command("input.wasm", "output.wasm");
        assert!(cmd.contains("wasm-opt"));
        assert!(cmd.contains("input.wasm"));
        assert!(cmd.contains("output.wasm"));
    }

    #[test]
    fn test_wasm_size_analyzer() {
        let mut analyzer = WasmSizeAnalyzer::new();
        analyzer.add_section("code", 50000);
        analyzer.add_section("data", 20000);
        analyzer.add_section("name", 10000);

        assert_eq!(analyzer.total_size(), 80000);
        let largest = analyzer.largest_section().unwrap();
        assert_eq!(largest.name, "code");
    }

    #[test]
    fn test_wasm_size_analyzer_report() {
        let mut analyzer = WasmSizeAnalyzer::new();
        analyzer.add_section("code", 100);
        analyzer.add_section("data", 50);

        let report = analyzer.report();
        assert!(report.contains("150"));
        assert!(report.contains("code"));
    }

    #[test]
    fn test_wasm_perf_counter() {
        let mut perf = WasmPerfCounter::new();
        perf.record_crossing(100);
        perf.record_crossing(200);
        perf.record_crossing(300);

        assert_eq!(perf.crossings(), 3);
        assert!((perf.average_time_ns() - 200.0).abs() < f64::EPSILON);
        assert_eq!(perf.max_time_ns(), 300);
    }

    #[test]
    fn test_wasm_perf_counter_empty() {
        let perf = WasmPerfCounter::new();
        assert_eq!(perf.crossings(), 0);
        assert_eq!(perf.average_time_ns(), 0.0);
    }

    #[test]
    fn test_batch_processor() {
        let mut bp = BatchProcessor::new(3);
        assert!(bp.add(1).is_none());
        assert!(bp.add(2).is_none());

        let batch = bp.add(3).unwrap();
        assert_eq!(batch, vec![1, 2, 3]);
        assert_eq!(bp.len(), 0);
    }

    #[test]
    fn test_batch_processor_flush() {
        let mut bp = BatchProcessor::new(10);
        bp.add(1);
        bp.add(2);

        let batch = bp.flush();
        assert_eq!(batch, vec![1, 2]);
        assert_eq!(bp.len(), 0);
    }

    #[test]
    fn test_wasm_features_default() {
        let features = WasmFeatures::default();
        assert!(!features.simd);
        assert!(!features.threads);
        assert!(features.bulk_memory);
    }

    #[test]
    fn test_wasm_features_rustflags() {
        let mut features = WasmFeatures::default();
        features.simd = true;
        let flags = features.rustflags();
        assert!(flags.iter().any(|f| f.contains("simd128")));
    }

    #[test]
    fn test_wasm_features_compatibility() {
        let features = WasmFeatures::default();
        assert!(features.browser_compatibility() >= 85);

        let mut heavy = WasmFeatures::default();
        heavy.simd = true;
        heavy.threads = true;
        assert!(heavy.browser_compatibility() < features.browser_compatibility());
    }

    #[test]
    fn test_wasm_size_analyzer_percentages() {
        let mut analyzer = WasmSizeAnalyzer::new();
        analyzer.add_section("a", 75);
        analyzer.add_section("b", 25);

        let sections = &analyzer.sections;
        assert!((sections[0].percentage - 75.0).abs() < f64::EPSILON);
        assert!((sections[1].percentage - 25.0).abs() < f64::EPSILON);
    }
}
