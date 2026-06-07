//! # GitHub Actions for Rust Projects
//!
//! GitHub Actions is the de facto CI/CD platform for Rust open-source projects.
//! This module covers workflow design, caching strategies, matrix builds, and
//! artifact management with real-world patterns.
//!
//! ## Typical Rust CI Workflow (YAML reference):
//!
//! ```yaml
//! name: CI
//! on:
//!   push:
//!     branches: [main]
//!   pull_request:
//!     branches: [main]
//!
//! env:
//!   CARGO_TERM_COLOR: always
//!   RUSTFLAGS: "-D warnings"
//!
//! jobs:
//!   test:
//!     runs-on: ubuntu-latest
//!     strategy:
//!       matrix:
//!         rust: [stable, nightly]
//!     steps:
//!       - uses: actions/checkout@v4
//!       - uses: dtolnay/rust-toolchain@master
//!         with:
//!           toolchain: ${{ matrix.rust }}
//!           components: clippy, rustfmt
//!       - uses: actions/cache@v4
//!         with:
//!           path: |
//!             ~/.cargo/bin/
//!             ~/.cargo/registry/index/
//!             ~/.cargo/registry/cache/
//!             ~/.cargo/git/db/
//!             target/
//!           key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
//!       - run: cargo fmt --all -- --check
//!       - run: cargo clippy --all-targets --all-features -- -D warnings
//!       - run: cargo test --all-features
//!       - run: cargo doc --no-deps
//! ```

use serde::{Deserialize, Serialize};

/// Represents a GitHub Actions workflow configuration.
/// This models the structure for programmatic workflow generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub on: TriggerConfig,
    pub env: std::collections::HashMap<String, String>,
    pub jobs: std::collections::HashMap<String, Job>,
}

/// Trigger configuration for workflows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub push: Option<BranchFilter>,
    pub pull_request: Option<BranchFilter>,
    pub schedule: Option<Vec<CronSchedule>>,
    pub workflow_dispatch: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchFilter {
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronSchedule {
    pub cron: String,
}

/// A single job in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub runs_on: String,
    pub strategy: Option<MatrixStrategy>,
    pub steps: Vec<Step>,
    pub needs: Option<Vec<String>>,
    pub if_condition: Option<String>,
}

/// Matrix strategy for testing across multiple configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixStrategy {
    pub matrix: serde_json::Value,
    pub fail_fast: Option<bool>,
}

/// A single step in a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    pub name: Option<String>,
    #[serde(rename = "uses")]
    pub uses_action: Option<String>,
    pub run: Option<String>,
    pub with: Option<std::collections::HashMap<String, String>>,
    pub env: Option<std::collections::HashMap<String, String>>,
    pub if_condition: Option<String>,
    pub id: Option<String>,
}

/// Builder for constructing GitHub Actions workflows programmatically.
/// Useful for monorepos or projects that need dynamic CI generation.
pub struct WorkflowBuilder {
    name: String,
    triggers: TriggerConfig,
    env: std::collections::HashMap<String, String>,
    jobs: std::collections::HashMap<String, Job>,
}

impl WorkflowBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            triggers: TriggerConfig {
                push: None,
                pull_request: None,
                schedule: None,
                workflow_dispatch: None,
            },
            env: std::collections::HashMap::new(),
            jobs: std::collections::HashMap::new(),
        }
    }

    /// Configure push triggers for specific branches.
    pub fn on_push(mut self, branches: Vec<&str>) -> Self {
        self.triggers.push = Some(BranchFilter {
            branches: branches.into_iter().map(String::from).collect(),
        });
        self
    }

    /// Configure pull request triggers.
    pub fn on_pull_request(mut self, branches: Vec<&str>) -> Self {
        self.triggers.pull_request = Some(BranchFilter {
            branches: branches.into_iter().map(String::from).collect(),
        });
        self
    }

    /// Add a cron schedule trigger.
    pub fn on_schedule(mut self, cron: &str) -> Self {
        self.triggers
            .schedule
            .get_or_insert_with(Vec::new)
            .push(CronSchedule {
                cron: cron.to_string(),
            });
        self
    }

    /// Set a global environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Add a job to the workflow.
    pub fn job(mut self, name: impl Into<String>, job: Job) -> Self {
        self.jobs.insert(name.into(), job);
        self
    }

    /// Build the workflow.
    pub fn build(self) -> Workflow {
        Workflow {
            name: self.name,
            on: self.triggers,
            env: self.env,
            jobs: self.jobs,
        }
    }
}

/// Cache configuration for Rust projects on GitHub Actions.
/// Proper caching can reduce CI times by 50-80%.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Paths to cache (cargo registry, target dir, etc.)
    pub paths: Vec<String>,
    /// Cache key template (usually includes hash of Cargo.lock)
    pub key_template: String,
    /// Restore keys for partial cache hits
    pub restore_keys: Vec<String>,
}

impl CacheConfig {
    /// Standard Rust cargo cache configuration.
    pub fn cargo() -> Self {
        Self {
            paths: vec![
                "~/.cargo/bin/".to_string(),
                "~/.cargo/registry/index/".to_string(),
                "~/.cargo/registry/cache/".to_string(),
                "~/.cargo/git/db/".to_string(),
                "target/".to_string(),
            ],
            key_template: "{os}-cargo-{cargo_lock_hash}".to_string(),
            restore_keys: vec!["{os}-cargo-".to_string()],
        }
    }

    /// Cache configuration for rustup toolchain.
    pub fn rustup() -> Self {
        Self {
            paths: vec![
                "~/.rustup/".to_string(),
                "~/.cargo/".to_string(),
            ],
            key_template: "{os}-rustup-{toolchain}".to_string(),
            restore_keys: vec!["{os}-rustup-".to_string()],
        }
    }

    /// Generate the cache step configuration.
    pub fn to_cache_step(&self, os: &str, cargo_lock_hash: &str) -> Step {
        let key = self
            .key_template
            .replace("{os}", os)
            .replace("{cargo_lock_hash}", cargo_lock_hash);
        let mut with = std::collections::HashMap::new();
        with.insert("path".to_string(), self.paths.join("\n"));
        with.insert("key".to_string(), key);
        with.insert(
            "restore-keys".to_string(),
            self.restore_keys.join("\n"),
        );

        Step {
            name: Some("Cache cargo registry and build".to_string()),
            uses_action: Some("actions/cache@v4".to_string()),
            run: None,
            with: Some(with),
            env: None,
            if_condition: None,
            id: None,
        }
    }
}

/// Matrix build configuration for testing across multiple Rust versions
/// and operating systems.
#[derive(Debug, Clone)]
pub struct MatrixConfig {
    pub rust_versions: Vec<String>,
    pub os_list: Vec<String>,
    pub include_extra: Vec<MatrixExtra>,
    pub exclude: Vec<MatrixExclude>,
}

#[derive(Debug, Clone)]
pub struct MatrixExtra {
    pub rust: String,
    pub os: String,
    pub extra_vars: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct MatrixExclude {
    pub rust: String,
    pub os: String,
}

impl MatrixConfig {
    /// Standard CI matrix: stable + nightly on Linux/macOS/Windows.
    pub fn standard() -> Self {
        Self {
            rust_versions: vec!["stable".to_string(), "nightly".to_string()],
            os_list: vec![
                "ubuntu-latest".to_string(),
                "macos-latest".to_string(),
                "windows-latest".to_string(),
            ],
            include_extra: vec![],
            exclude: vec![],
        }
    }

    /// Conservative matrix: just stable on Linux (for fast PR checks).
    pub fn minimal() -> Self {
        Self {
            rust_versions: vec!["stable".to_string()],
            os_list: vec!["ubuntu-latest".to_string()],
            include_extra: vec![],
            exclude: vec![],
        }
    }

    /// Convert to serde_json::Value for the matrix strategy.
    pub fn to_matrix_value(&self) -> serde_json::Value {
        let mut matrix = serde_json::json!({
            "rust": self.rust_versions,
            "os": self.os_list,
        });

        if !self.include_extra.is_empty() {
            let includes: Vec<serde_json::Value> = self
                .include_extra
                .iter()
                .map(|e| {
                    let mut obj = serde_json::json!({
                        "rust": e.rust,
                        "os": e.os,
                    });
                    for (k, v) in &e.extra_vars {
                        obj[k] = serde_json::json!(v);
                    }
                    obj
                })
                .collect();
            matrix["include"] = serde_json::json!(includes);
        }

        if !self.exclude.is_empty() {
            let excludes: Vec<serde_json::Value> = self
                .exclude
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "rust": e.rust,
                        "os": e.os,
                    })
                })
                .collect();
            matrix["exclude"] = serde_json::json!(excludes);
        }

        matrix
    }
}

/// Artifact configuration for build outputs.
#[derive(Debug, Clone)]
pub struct ArtifactConfig {
    pub name: String,
    pub path: String,
    pub retention_days: u32,
}

impl ArtifactConfig {
    /// Create a binary artifact configuration.
    pub fn binary(binary_name: &str, os: &str) -> Self {
        let ext = if os.contains("windows") { ".exe" } else { "" };
        Self {
            name: format!("{}-{}{}", binary_name, os, ext),
            path: format!("target/release/{}{}", binary_name, ext),
            retention_days: 7,
        }
    }

    /// Generate the upload-artifact step.
    pub fn to_upload_step(&self) -> Step {
        let mut with = std::collections::HashMap::new();
        with.insert("name".to_string(), self.name.clone());
        with.insert("path".to_string(), self.path.clone());
        with.insert("retention-days".to_string(), self.retention_days.to_string());

        Step {
            name: Some(format!("Upload artifact: {}", self.name)),
            uses_action: Some("actions/upload-artifact@v4".to_string()),
            run: None,
            with: Some(with),
            env: None,
            if_condition: None,
            id: None,
        }
    }
}

/// Helper to generate a complete Rust CI workflow.
pub fn generate_rust_ci(
    project_name: &str,
    matrix: &MatrixConfig,
    run_clippy: bool,
    run_tests: bool,
    check_docs: bool,
) -> Workflow {
    let mut builder = WorkflowBuilder::new(format!("CI for {}", project_name))
        .on_push(vec!["main"])
        .on_pull_request(vec!["main"])
        .env("CARGO_TERM_COLOR", "always")
        .env("RUSTFLAGS", "-D warnings");

    let mut steps = vec![
        Step {
            name: Some("Checkout".to_string()),
            uses_action: Some("actions/checkout@v4".to_string()),
            run: None,
            with: None,
            env: None,
            if_condition: None,
            id: None,
        },
        Step {
            name: Some("Install Rust toolchain".to_string()),
            uses_action: Some("dtolnay/rust-toolchain@master".to_string()),
            run: None,
            with: Some({
                let mut m = std::collections::HashMap::new();
                m.insert("toolchain".to_string(), "${{ matrix.rust }}".to_string());
                m.insert(
                    "components".to_string(),
                    "clippy, rustfmt".to_string(),
                );
                m
            }),
            env: None,
            if_condition: None,
            id: None,
        },
        CacheConfig::cargo().to_cache_step("${{ matrix.os }}", "${{ hashFiles('**/Cargo.lock') }}"),
    ];

    // Format check (always)
    steps.push(Step {
        name: Some("Check formatting".to_string()),
        uses_action: None,
        run: Some("cargo fmt --all -- --check".to_string()),
        with: None,
        env: None,
        if_condition: None,
        id: None,
    });

    if run_clippy {
        steps.push(Step {
            name: Some("Clippy lint".to_string()),
            uses_action: None,
            run: Some(
                "cargo clippy --all-targets --all-features -- -D warnings".to_string(),
            ),
            with: None,
            env: None,
            if_condition: None,
            id: None,
        });
    }

    if run_tests {
        steps.push(Step {
            name: Some("Run tests".to_string()),
            uses_action: None,
            run: Some("cargo test --all-features".to_string()),
            with: None,
            env: None,
            if_condition: None,
            id: None,
        });
    }

    if check_docs {
        steps.push(Step {
            name: Some("Build docs".to_string()),
            uses_action: None,
            run: Some("cargo doc --no-deps".to_string()),
            with: None,
            env: None,
            if_condition: None,
            id: None,
        });
    }

    let job = Job {
        runs_on: "${{ matrix.os }}".to_string(),
        strategy: Some(MatrixStrategy {
            matrix: matrix.to_matrix_value(),
            fail_fast: Some(false),
        }),
        steps,
        needs: None,
        if_condition: None,
    };

    builder = builder.job("test", job);
    builder.build()
}

/// Represents a CI status check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiStatus {
    pub check_name: String,
    pub status: CheckStatus,
    pub duration_secs: f64,
    pub details_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckStatus {
    Success,
    Failure,
    Cancelled,
    Skipped,
    Pending,
}

impl CiStatus {
    pub fn is_passing(&self) -> bool {
        self.status == CheckStatus::Success
    }

    /// Aggregate multiple CI statuses to determine overall result.
    pub fn aggregate(statuses: &[CiStatus]) -> CheckStatus {
        if statuses.is_empty() {
            return CheckStatus::Pending;
        }
        if statuses.iter().any(|s| s.status == CheckStatus::Failure) {
            return CheckStatus::Failure;
        }
        if statuses.iter().any(|s| s.status == CheckStatus::Cancelled) {
            return CheckStatus::Cancelled;
        }
        if statuses.iter().all(|s| s.status == CheckStatus::Success) {
            return CheckStatus::Success;
        }
        CheckStatus::Pending
    }
}

/// Represents build timing information for CI optimization.
#[derive(Debug, Clone)]
pub struct BuildTiming {
    pub step_name: String,
    pub duration_secs: f64,
    pub cache_hit: bool,
}

impl BuildTiming {
    /// Analyze build timings to identify bottlenecks.
    pub fn find_bottlenecks(timings: &[BuildTiming], threshold_secs: f64) -> Vec<&BuildTiming> {
        timings
            .iter()
            .filter(|t| t.duration_secs > threshold_secs)
            .collect()
    }

    /// Calculate total build time.
    pub fn total_duration(timings: &[BuildTiming]) -> f64 {
        timings.iter().map(|t| t.duration_secs).sum()
    }

    /// Calculate cache hit rate.
    pub fn cache_hit_rate(timings: &[BuildTiming]) -> f64 {
        if timings.is_empty() {
            return 0.0;
        }
        let hits = timings.iter().filter(|t| t.cache_hit).count() as f64;
        hits / timings.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_builder_creates_valid_workflow() {
        let workflow = WorkflowBuilder::new("Test CI")
            .on_push(vec!["main", "develop"])
            .on_pull_request(vec!["main"])
            .env("CARGO_TERM_COLOR", "always")
            .job(
                "test",
                Job {
                    runs_on: "ubuntu-latest".to_string(),
                    strategy: None,
                    steps: vec![Step {
                        name: Some("Run tests".to_string()),
                        uses_action: None,
                        run: Some("cargo test".to_string()),
                        with: None,
                        env: None,
                        if_condition: None,
                        id: None,
                    }],
                    needs: None,
                    if_condition: None,
                },
            )
            .build();

        assert_eq!(workflow.name, "Test CI");
        assert!(workflow.on.push.is_some());
        assert!(workflow.on.pull_request.is_some());
        assert!(workflow.jobs.contains_key("test"));
        assert_eq!(workflow.env.get("CARGO_TERM_COLOR").unwrap(), "always");
    }

    #[test]
    fn test_cargo_cache_config() {
        let cache = CacheConfig::cargo();
        assert!(cache.paths.contains(&"target/".to_string()));
        assert!(cache.key_template.contains("{cargo_lock_hash}"));
    }

    #[test]
    fn test_matrix_config_standard() {
        let matrix = MatrixConfig::standard();
        assert_eq!(matrix.rust_versions.len(), 2);
        assert_eq!(matrix.os_list.len(), 3);
    }

    #[test]
    fn test_matrix_config_minimal() {
        let matrix = MatrixConfig::minimal();
        assert_eq!(matrix.rust_versions.len(), 1);
        assert_eq!(matrix.os_list.len(), 1);
    }

    #[test]
    fn test_matrix_to_value() {
        let matrix = MatrixConfig::standard();
        let value = matrix.to_matrix_value();
        assert!(value.get("rust").is_some());
        assert!(value.get("os").is_some());
    }

    #[test]
    fn test_artifact_binary_config() {
        let artifact = ArtifactConfig::binary("myapp", "ubuntu-latest");
        assert!(artifact.path.contains("target/release/"));
        assert!(!artifact.path.contains(".exe"));

        let win = ArtifactConfig::binary("myapp", "windows-latest");
        assert!(win.path.contains(".exe"));
    }

    #[test]
    fn test_ci_status_aggregate() {
        let all_pass = vec![
            CiStatus {
                check_name: "test".into(),
                status: CheckStatus::Success,
                duration_secs: 60.0,
                details_url: None,
            },
            CiStatus {
                check_name: "lint".into(),
                status: CheckStatus::Success,
                duration_secs: 30.0,
                details_url: None,
            },
        ];
        assert_eq!(CiStatus::aggregate(&all_pass), CheckStatus::Success);

        let one_fail = vec![
            CiStatus {
                check_name: "test".into(),
                status: CheckStatus::Success,
                duration_secs: 60.0,
                details_url: None,
            },
            CiStatus {
                check_name: "lint".into(),
                status: CheckStatus::Failure,
                duration_secs: 30.0,
                details_url: None,
            },
        ];
        assert_eq!(CiStatus::aggregate(&one_fail), CheckStatus::Failure);
        assert_eq!(CiStatus::aggregate(&[]), CheckStatus::Pending);
    }

    #[test]
    fn test_build_timing_analysis() {
        let timings = vec![
            BuildTiming {
                step_name: "compile".into(),
                duration_secs: 120.0,
                cache_hit: false,
            },
            BuildTiming {
                step_name: "test".into(),
                duration_secs: 45.0,
                cache_hit: true,
            },
            BuildTiming {
                step_name: "clippy".into(),
                duration_secs: 30.0,
                cache_hit: true,
            },
        ];

        let bottlenecks = BuildTiming::find_bottlenecks(&timings, 60.0);
        assert_eq!(bottlenecks.len(), 1);
        assert_eq!(bottlenecks[0].step_name, "compile");

        let total = BuildTiming::total_duration(&timings);
        assert!((total - 195.0).abs() < f64::EPSILON);

        let hit_rate = BuildTiming::cache_hit_rate(&timings);
        assert!((hit_rate - 2.0 / 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_generate_rust_ci_workflow() {
        let matrix = MatrixConfig::minimal();
        let workflow = generate_rust_ci("my-project", &matrix, true, true, true);

        assert_eq!(workflow.name, "CI for my-project");
        let job = workflow.jobs.get("test").unwrap();
        assert!(job.strategy.is_some());
        // Should have: checkout, toolchain, cache, fmt, clippy, test, docs = 7 steps
        assert_eq!(job.steps.len(), 7);
    }

    #[test]
    fn test_generate_ci_without_optional_steps() {
        let matrix = MatrixConfig::minimal();
        let workflow = generate_rust_ci("my-project", &matrix, false, false, false);

        let job = workflow.jobs.get("test").unwrap();
        // checkout, toolchain, cache, fmt = 4 steps
        assert_eq!(job.steps.len(), 4);
    }

    #[test]
    fn test_workflow_serialization_roundtrip() {
        let workflow = generate_rust_ci("test-proj", &MatrixConfig::minimal(), true, true, false);
        let json = serde_json::to_string(&workflow).unwrap();
        let deserialized: Workflow = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, workflow.name);
    }

    #[test]
    fn test_cache_step_generation() {
        let cache = CacheConfig::cargo();
        let step = cache.to_cache_step("ubuntu", "abc123");
        assert!(step.uses_action.is_some());
        let with = step.with.unwrap();
        assert!(with.get("key").unwrap().contains("abc123"));
    }
}
