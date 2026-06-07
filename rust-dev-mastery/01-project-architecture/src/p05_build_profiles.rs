//! # Lesson 5: Build Profiles
//!
//! Cargo build profiles control optimization levels, debug info, and codegen
//! settings. This lesson covers dev vs release profiles, custom profiles,
//! LTO, and codegen-units tuning.

use serde::{Deserialize, Serialize};

/// Represents a Cargo build profile with its settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildProfile {
    pub name: String,
    pub opt_level: OptLevel,
    pub debug: DebugInfo,
    pub lto: LtoMode,
    pub codegen_units: u32,
    pub incremental: bool,
    pub overflow_checks: bool,
    pub debug_assertions: bool,
    pub panic_strategy: PanicStrategy,
    pub strip: StripMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OptLevel {
    /// No optimizations, fastest compile time.
    Zero,
    /// Minimal optimizations.
    One,
    /// Default optimizations.
    Two,
    /// More optimizations, slower compile.
    Three,
    /// Optimize for size (Os).
    Size,
    /// Aggressively optimize for size (Oz).
    SizeMin,
}

impl OptLevel {
    pub fn to_cargo_value(&self) -> &str {
        match self {
            OptLevel::Zero => "0",
            OptLevel::One => "1",
            OptLevel::Two => "2",
            OptLevel::Three => "3",
            OptLevel::Size => "\"s\"",
            OptLevel::SizeMin => "\"z\"",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DebugInfo {
    /// No debug info (fastest, smallest binary).
    None,
    /// Line tables only.
    LineTablesOnly,
    /// Full debug info.
    Full,
}

impl DebugInfo {
    pub fn to_cargo_value(&self) -> &str {
        match self {
            DebugInfo::None => "false",
            DebugInfo::LineTablesOnly => "line-tables-only",
            DebugInfo::Full => "true",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LtoMode {
    /// No link-time optimization.
    Off,
    /// Thin LTO (good balance).
    Thin,
    /// Fat LTO (maximum optimization, slowest).
    Fat,
}

impl LtoMode {
    pub fn to_cargo_value(&self) -> &str {
        match self {
            LtoMode::Off => "false",
            LtoMode::Thin => "\"thin\"",
            LtoMode::Fat => "true",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PanicStrategy {
    /// Unwinding on panic (default).
    Unwind,
    /// Abort on panic (smaller binary).
    Abort,
}

impl PanicStrategy {
    pub fn to_cargo_value(&self) -> &str {
        match self {
            PanicStrategy::Unwind => "\"unwind\"",
            PanicStrategy::Abort => "\"abort\"",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StripMode {
    /// Don't strip anything.
    None,
    /// Strip debug info.
    DebugInfo,
    /// Strip all symbols.
    Symbols,
}

impl StripMode {
    pub fn to_cargo_value(&self) -> &str {
        match self {
            StripMode::None => "\"none\"",
            StripMode::DebugInfo => "\"debuginfo\"",
            StripMode::Symbols => "\"symbols\"",
        }
    }
}

impl BuildProfile {
    /// The standard dev profile (cargo build).
    pub fn dev() -> Self {
        Self {
            name: "dev".to_string(),
            opt_level: OptLevel::Zero,
            debug: DebugInfo::Full,
            lto: LtoMode::Off,
            codegen_units: 256,
            incremental: true,
            overflow_checks: true,
            debug_assertions: true,
            panic_strategy: PanicStrategy::Unwind,
            strip: StripMode::None,
        }
    }

    /// The standard release profile (cargo build --release).
    pub fn release() -> Self {
        Self {
            name: "release".to_string(),
            opt_level: OptLevel::Three,
            debug: DebugInfo::None,
            lto: LtoMode::Off,
            codegen_units: 16,
            incremental: false,
            overflow_checks: false,
            debug_assertions: false,
            panic_strategy: PanicStrategy::Unwind,
            strip: StripMode::None,
        }
    }

    /// A production-optimized profile with maximum optimization.
    pub fn production() -> Self {
        Self {
            name: "release-lto".to_string(),
            opt_level: OptLevel::Three,
            debug: DebugInfo::None,
            lto: LtoMode::Fat,
            codegen_units: 1,
            incremental: false,
            overflow_checks: false,
            debug_assertions: false,
            panic_strategy: PanicStrategy::Abort,
            strip: StripMode::Symbols,
        }
    }

    /// A profile optimized for small binary size (e.g., WASM, embedded).
    pub fn small() -> Self {
        Self {
            name: "release-small".to_string(),
            opt_level: OptLevel::SizeMin,
            debug: DebugInfo::None,
            lto: LtoMode::Fat,
            codegen_units: 1,
            incremental: false,
            overflow_checks: false,
            debug_assertions: false,
            panic_strategy: PanicStrategy::Abort,
            strip: StripMode::Symbols,
        }
    }

    /// A profile for CI: fast compile but with debug assertions.
    pub fn ci() -> Self {
        Self {
            name: "ci".to_string(),
            opt_level: OptLevel::One,
            debug: DebugInfo::LineTablesOnly,
            lto: LtoMode::Off,
            codegen_units: 256,
            incremental: false,
            overflow_checks: true,
            debug_assertions: true,
            panic_strategy: PanicStrategy::Abort,
            strip: StripMode::None,
        }
    }

    /// Generate the Cargo.toml [profile.*] section for this profile.
    pub fn to_toml(&self) -> String {
        let section = if self.name == "dev" || self.name == "release" {
            format!("[profile.{}]", self.name)
        } else {
            format!("[profile.{}]", self.name)
        };

        let mut lines = vec![section];
        lines.push(format!("opt-level = {}", self.opt_level.to_cargo_value()));
        lines.push(format!("debug = {}", self.debug.to_cargo_value()));
        lines.push(format!("lto = {}", self.lto.to_cargo_value()));
        lines.push(format!("codegen-units = {}", self.codegen_units));
        lines.push(format!("incremental = {}", self.incremental));
        lines.push(format!(
            "overflow-checks = {}",
            self.overflow_checks
        ));
        lines.push(format!(
            "debug-assertions = {}",
            self.debug_assertions
        ));
        lines.push(format!(
            "panic = {}",
            self.panic_strategy.to_cargo_value()
        ));
        lines.push(format!("strip = {}", self.strip.to_cargo_value()));

        lines.join("\n")
    }

    /// Estimate relative compile time (1.0 = baseline dev).
    pub fn relative_compile_time(&self) -> f64 {
        let base = 1.0;
        let opt_factor = match self.opt_level {
            OptLevel::Zero => 1.0,
            OptLevel::One => 1.2,
            OptLevel::Two => 1.8,
            OptLevel::Three => 2.5,
            OptLevel::Size => 2.5,
            OptLevel::SizeMin => 3.0,
        };
        let lto_factor = match self.lto {
            LtoMode::Off => 1.0,
            LtoMode::Thin => 1.5,
            LtoMode::Fat => 3.0,
        };
        let codegen_factor = 256.0 / self.codegen_units as f64;
        base * opt_factor * lto_factor * codegen_factor
    }

    /// Estimate relative binary size (1.0 = baseline release).
    pub fn relative_binary_size(&self) -> f64 {
        let base = 1.5; // dev is typically 50% larger
        let opt_factor = match self.opt_level {
            OptLevel::Zero => 1.5,
            OptLevel::One => 1.3,
            OptLevel::Two => 1.0,
            OptLevel::Three => 0.95,
            OptLevel::Size => 0.7,
            OptLevel::SizeMin => 0.6,
        };
        let lto_factor = match self.lto {
            LtoMode::Off => 1.0,
            LtoMode::Thin => 0.9,
            LtoMode::Fat => 0.8,
        };
        let strip_factor = match self.strip {
            StripMode::None => 1.0,
            StripMode::DebugInfo => 0.85,
            StripMode::Symbols => 0.7,
        };
        let debug_factor = match self.debug {
            DebugInfo::None => 1.0,
            DebugInfo::LineTablesOnly => 1.05,
            DebugInfo::Full => 1.3,
        };
        base * opt_factor * lto_factor * strip_factor * debug_factor
    }
}

/// Compare multiple profiles side by side.
pub fn compare_profiles(profiles: &[BuildProfile]) -> ProfileComparison {
    ProfileComparison {
        profiles: profiles.iter().map(|p| ProfileSummary {
            name: p.name.clone(),
            compile_time: p.relative_compile_time(),
            binary_size: p.relative_binary_size(),
            debug_assertions: p.debug_assertions,
            has_debug_info: p.debug != DebugInfo::None,
        }).collect(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileComparison {
    pub profiles: Vec<ProfileSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProfileSummary {
    pub name: String,
    pub compile_time: f64,
    pub binary_size: f64,
    pub debug_assertions: bool,
    pub has_debug_info: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dev_profile() {
        let dev = BuildProfile::dev();
        assert_eq!(dev.name, "dev");
        assert_eq!(dev.opt_level, OptLevel::Zero);
        assert!(dev.debug_assertions);
        assert!(dev.incremental);
    }

    #[test]
    fn test_release_profile() {
        let release = BuildProfile::release();
        assert_eq!(release.name, "release");
        assert_eq!(release.opt_level, OptLevel::Three);
        assert!(!release.debug_assertions);
        assert!(!release.incremental);
    }

    #[test]
    fn test_production_profile() {
        let prod = BuildProfile::production();
        assert_eq!(prod.lto, LtoMode::Fat);
        assert_eq!(prod.codegen_units, 1);
        assert_eq!(prod.panic_strategy, PanicStrategy::Abort);
        assert_eq!(prod.strip, StripMode::Symbols);
    }

    #[test]
    fn test_small_profile() {
        let small = BuildProfile::small();
        assert_eq!(small.opt_level, OptLevel::SizeMin);
        assert_eq!(small.strip, StripMode::Symbols);
    }

    #[test]
    fn test_ci_profile() {
        let ci = BuildProfile::ci();
        assert_eq!(ci.opt_level, OptLevel::One);
        assert!(ci.debug_assertions);
        assert!(!ci.incremental);
        assert_eq!(ci.panic_strategy, PanicStrategy::Abort);
    }

    #[test]
    fn test_opt_level_cargo_values() {
        assert_eq!(OptLevel::Zero.to_cargo_value(), "0");
        assert_eq!(OptLevel::One.to_cargo_value(), "1");
        assert_eq!(OptLevel::Two.to_cargo_value(), "2");
        assert_eq!(OptLevel::Three.to_cargo_value(), "3");
        assert_eq!(OptLevel::Size.to_cargo_value(), "\"s\"");
        assert_eq!(OptLevel::SizeMin.to_cargo_value(), "\"z\"");
    }

    #[test]
    fn test_lto_cargo_values() {
        assert_eq!(LtoMode::Off.to_cargo_value(), "false");
        assert_eq!(LtoMode::Thin.to_cargo_value(), "\"thin\"");
        assert_eq!(LtoMode::Fat.to_cargo_value(), "true");
    }

    #[test]
    fn test_panic_cargo_values() {
        assert_eq!(PanicStrategy::Unwind.to_cargo_value(), "\"unwind\"");
        assert_eq!(PanicStrategy::Abort.to_cargo_value(), "\"abort\"");
    }

    #[test]
    fn test_strip_cargo_values() {
        assert_eq!(StripMode::None.to_cargo_value(), "\"none\"");
        assert_eq!(StripMode::DebugInfo.to_cargo_value(), "\"debuginfo\"");
        assert_eq!(StripMode::Symbols.to_cargo_value(), "\"symbols\"");
    }

    #[test]
    fn test_to_toml() {
        let prod = BuildProfile::production();
        let toml = prod.to_toml();
        assert!(toml.contains("[profile.release-lto]"));
        assert!(toml.contains("opt-level = 3"));
        assert!(toml.contains("lto = true"));
        assert!(toml.contains("codegen-units = 1"));
        assert!(toml.contains("panic = \"abort\""));
        assert!(toml.contains("strip = \"symbols\""));
    }

    #[test]
    fn test_relative_compile_time() {
        let dev = BuildProfile::dev();
        let release = BuildProfile::release();
        let prod = BuildProfile::production();

        // dev should be fastest
        assert!(dev.relative_compile_time() < release.relative_compile_time());
        // production should be slowest
        assert!(release.relative_compile_time() < prod.relative_compile_time());
    }

    #[test]
    fn test_relative_binary_size() {
        let dev = BuildProfile::dev();
        let release = BuildProfile::release();
        let prod = BuildProfile::production();
        let small = BuildProfile::small();

        // dev should be largest
        assert!(dev.relative_binary_size() > release.relative_binary_size());
        // production should be smaller than release
        assert!(prod.relative_binary_size() < release.relative_binary_size());
        // small should be smallest
        assert!(small.relative_binary_size() < prod.relative_binary_size());
    }

    #[test]
    fn test_compare_profiles() {
        let profiles = vec![
            BuildProfile::dev(),
            BuildProfile::release(),
            BuildProfile::production(),
        ];
        let comparison = compare_profiles(&profiles);
        assert_eq!(comparison.profiles.len(), 3);
        assert_eq!(comparison.profiles[0].name, "dev");
        assert!(comparison.profiles[0].debug_assertions);
        assert!(!comparison.profiles[1].debug_assertions);
    }

    #[test]
    fn test_debug_info_values() {
        assert_eq!(DebugInfo::None.to_cargo_value(), "false");
        assert_eq!(DebugInfo::LineTablesOnly.to_cargo_value(), "line-tables-only");
        assert_eq!(DebugInfo::Full.to_cargo_value(), "true");
    }
}
