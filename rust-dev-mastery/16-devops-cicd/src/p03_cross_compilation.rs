//! # Cross-Compilation for Rust
//!
//! Cross-compilation allows building binaries for platforms different from the
//! host machine. This is essential for CI/CD pipelines that need to produce
//! binaries for Linux, macOS, Windows, and embedded targets.
//!
//! ## Key Concepts:
//!
//! - **Target Triple**: `<arch>-<vendor>-<os>-<env>` (e.g., `x86_64-unknown-linux-gnu`)
//! - **Linker**: The tool that combines object files into a binary
//! - **Sysroot**: The system headers and libraries for the target
//! - **cross**: A tool that simplifies cross-compilation using Docker
//! - **musl**: Static linking for Linux (no glibc dependency)
//!
//! ## Common Targets:
//!
//! | Target | Use Case |
//! |--------|----------|
//! | `x86_64-unknown-linux-gnu` | Standard Linux |
//! | `x86_64-unknown-linux-musl` | Static Linux binaries |
//! | `aarch64-unknown-linux-gnu` | ARM64 Linux |
//! | `x86_64-apple-darwin` | Intel macOS |
//! | `aarch64-apple-darwin` | Apple Silicon macOS |
//! | `x86_64-pc-windows-msvc` | Windows |
//! | `wasm32-unknown-unknown` | WebAssembly |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a compilation target with all its configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationTarget {
    /// Target triple (e.g., "x86_64-unknown-linux-gnu")
    pub triple: String,
    /// Human-readable name
    pub name: String,
    /// OS family
    pub os: OsFamily,
    /// CPU architecture
    pub arch: Architecture,
    /// C library (gnu or musl)
    pub libc: Option<Libc>,
    /// Recommended linker
    pub linker: String,
    /// Whether this target supports static linking
    pub supports_static: bool,
    /// Whether the binary runs on the host
    pub is_native: bool,
    /// Docker image for cross-compilation (if using cross)
    pub cross_image: Option<String>,
    /// Additional rustflags needed
    pub rustflags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OsFamily {
    Linux,
    MacOS,
    Windows,
    Wasm,
    FreeBsd,
    Android,
    Ios,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Architecture {
    X86_64,
    Aarch64,
    Arm,
    Wasm32,
    Riscv64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Libc {
    Gnu,
    Musl,
}

/// Registry of well-known compilation targets.
pub struct TargetRegistry {
    targets: HashMap<String, CompilationTarget>,
}

impl TargetRegistry {
    pub fn new() -> Self {
        let mut targets = HashMap::new();

        targets.insert(
            "x86_64-unknown-linux-gnu".into(),
            CompilationTarget {
                triple: "x86_64-unknown-linux-gnu".into(),
                name: "Linux x86_64 (glibc)".into(),
                os: OsFamily::Linux,
                arch: Architecture::X86_64,
                libc: Some(Libc::Gnu),
                linker: "cc".into(),
                supports_static: false,
                is_native: cfg!(target_os = "linux") && cfg!(target_arch = "x86_64"),
                cross_image: Some("ghcr.io/cross-rs/x86_64-unknown-linux-gnu:main".into()),
                rustflags: vec![],
            },
        );

        targets.insert(
            "x86_64-unknown-linux-musl".into(),
            CompilationTarget {
                triple: "x86_64-unknown-linux-musl".into(),
                name: "Linux x86_64 (static/musl)".into(),
                os: OsFamily::Linux,
                arch: Architecture::X86_64,
                libc: Some(Libc::Musl),
                linker: "musl-gcc".into(),
                supports_static: true,
                is_native: false,
                cross_image: Some("ghcr.io/cross-rs/x86_64-unknown-linux-musl:main".into()),
                rustflags: vec!["-C target-feature=+crt-static".into()],
            },
        );

        targets.insert(
            "aarch64-unknown-linux-gnu".into(),
            CompilationTarget {
                triple: "aarch64-unknown-linux-gnu".into(),
                name: "Linux ARM64".into(),
                os: OsFamily::Linux,
                arch: Architecture::Aarch64,
                libc: Some(Libc::Gnu),
                linker: "aarch64-linux-gnu-gcc".into(),
                supports_static: false,
                is_native: false,
                cross_image: Some("ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main".into()),
                rustflags: vec![],
            },
        );

        targets.insert(
            "x86_64-apple-darwin".into(),
            CompilationTarget {
                triple: "x86_64-apple-darwin".into(),
                name: "macOS Intel".into(),
                os: OsFamily::MacOS,
                arch: Architecture::X86_64,
                libc: None,
                linker: "clang".into(),
                supports_static: false,
                is_native: cfg!(target_os = "macos") && cfg!(target_arch = "x86_64"),
                cross_image: None,
                rustflags: vec![],
            },
        );

        targets.insert(
            "aarch64-apple-darwin".into(),
            CompilationTarget {
                triple: "aarch64-apple-darwin".into(),
                name: "macOS Apple Silicon".into(),
                os: OsFamily::MacOS,
                arch: Architecture::Aarch64,
                libc: None,
                linker: "clang".into(),
                supports_static: false,
                is_native: cfg!(target_os = "macos") && cfg!(target_arch = "aarch64"),
                cross_image: None,
                rustflags: vec![],
            },
        );

        targets.insert(
            "x86_64-pc-windows-msvc".into(),
            CompilationTarget {
                triple: "x86_64-pc-windows-msvc".into(),
                name: "Windows x86_64".into(),
                os: OsFamily::Windows,
                arch: Architecture::X86_64,
                libc: None,
                linker: "link.exe".into(),
                supports_static: false,
                is_native: cfg!(target_os = "windows"),
                cross_image: None,
                rustflags: vec![],
            },
        );

        targets.insert(
            "wasm32-unknown-unknown".into(),
            CompilationTarget {
                triple: "wasm32-unknown-unknown".into(),
                name: "WebAssembly".into(),
                os: OsFamily::Wasm,
                arch: Architecture::Wasm32,
                libc: None,
                linker: "wasm-ld".into(),
                supports_static: true,
                is_native: false,
                cross_image: None,
                rustflags: vec!["-C link-arg=--export-table".into()],
            },
        );

        Self { targets }
    }

    pub fn get(&self, triple: &str) -> Option<&CompilationTarget> {
        self.targets.get(triple)
    }

    pub fn list_all(&self) -> Vec<&CompilationTarget> {
        self.targets.values().collect()
    }

    pub fn list_by_os(&self, os: &OsFamily) -> Vec<&CompilationTarget> {
        self.targets.values().filter(|t| &t.os == os).collect()
    }
}

/// Configuration for cross-compilation in CI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossConfig {
    /// The target triple
    pub target: String,
    /// Use cross tool (Docker-based) instead of native toolchain
    pub use_cross: bool,
    /// Docker image override
    pub docker_image: Option<String>,
    /// Environment variables to set
    pub env: HashMap<String, String>,
    /// Pre-build script (e.g., to install dependencies)
    pub pre_build: Option<String>,
}

impl CrossConfig {
    /// Create a config for the cross tool.
    pub fn for_cross(target: &str) -> Self {
        Self {
            target: target.to_string(),
            use_cross: true,
            docker_image: None,
            env: HashMap::new(),
            pre_build: None,
        }
    }

    /// Create a config for native cargo build.
    pub fn for_cargo(target: &str) -> Self {
        Self {
            target: target.to_string(),
            use_cross: false,
            docker_image: None,
            env: HashMap::new(),
            pre_build: None,
        }
    }

    /// Generate the build command.
    pub fn build_command(&self) -> String {
        let tool = if self.use_cross { "cross" } else { "cargo" };
        format!("{} build --release --target {}", tool, self.target)
    }

    /// Generate the test command.
    pub fn test_command(&self) -> String {
        let tool = if self.use_cross { "cross" } else { "cargo" };
        format!("{} test --target {}", tool, self.target)
    }
}

/// Generate cross.toml configuration for the `cross` tool.
pub fn generate_cross_toml(targets: &[CrossConfig]) -> String {
    let mut output = String::new();

    for config in targets {
        output.push_str(&format!("[target.{}]\n", config.target));

        if let Some(ref image) = config.docker_image {
            output.push_str(&format!("image = \"{}\"\n", image));
        }

        if !config.env.is_empty() {
            output.push_str("build-std = false\n");
            output.push_str("[target.env]\n");
            for (key, value) in &config.env {
                output.push_str(&format!("{} = \"{}\"\n", key, value));
            }
        }

        if let Some(ref script) = config.pre_build {
            output.push_str(&format!("pre-build = [\"{}\"]\n", script));
        }

        output.push('\n');
    }

    output
}

/// Generate .cargo/config.toml for cross-compilation linker configuration.
pub fn generate_cargo_config(targets: &[CompilationTarget]) -> String {
    let mut output = String::new();

    for target in targets {
        output.push_str(&format!("[target.{}]\n", target.triple));
        output.push_str(&format!("linker = \"{}\"\n", target.linker));

        if !target.rustflags.is_empty() {
            let flags: Vec<&str> = target
                .rustflags
                .iter()
                .map(|s| s.as_str())
                .collect();
            output.push_str(&format!("rustflags = {:?}\n", flags));
        }

        output.push('\n');
    }

    output
}

/// Represents a build matrix entry for CI cross-compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMatrixEntry {
    pub os: String,
    pub target: String,
    pub cross: bool,
    pub artifact_name: String,
}

impl BuildMatrixEntry {
    /// Generate the standard release build matrix.
    pub fn release_matrix() -> Vec<Self> {
        vec![
            Self {
                os: "ubuntu-latest".into(),
                target: "x86_64-unknown-linux-gnu".into(),
                cross: false,
                artifact_name: "linux-amd64".into(),
            },
            Self {
                os: "ubuntu-latest".into(),
                target: "x86_64-unknown-linux-musl".into(),
                cross: true,
                artifact_name: "linux-amd64-static".into(),
            },
            Self {
                os: "ubuntu-latest".into(),
                target: "aarch64-unknown-linux-gnu".into(),
                cross: true,
                artifact_name: "linux-arm64".into(),
            },
            Self {
                os: "macos-latest".into(),
                target: "aarch64-apple-darwin".into(),
                cross: false,
                artifact_name: "macos-arm64".into(),
            },
            Self {
                os: "macos-latest".into(),
                target: "x86_64-apple-darwin".into(),
                cross: false,
                artifact_name: "macos-amd64".into(),
            },
            Self {
                os: "windows-latest".into(),
                target: "x86_64-pc-windows-msvc".into(),
                cross: false,
                artifact_name: "windows-amd64".into(),
            },
        ]
    }
}

/// Dockerfile generator for multi-stage Rust builds.
pub struct DockerfileGenerator;

impl DockerfileGenerator {
    /// Generate a multi-stage Dockerfile for cross-compilation.
    pub fn multi_stage(binary_name: &str, target: &str) -> String {
        format!(
            r#"# Stage 1: Build
FROM rust:1.77-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {{}}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy real source and build
COPY src ./src
RUN touch src/main.rs
RUN cargo build --release --target {target}

# Stage 2: Runtime
FROM scratch
COPY --from=builder /app/target/{target}/release/{binary_name} /{binary_name}
ENTRYPOINT ["/{binary_name}"]
"#,
            target = target,
            binary_name = binary_name,
        )
    }

    /// Generate a Dockerfile using cargo-chef for optimal layer caching.
    pub fn with_chef(binary_name: &str, target: &str) -> String {
        format!(
            r#"# Stage 1: Planner
FROM rust:1.77-slim as planner
RUN cargo install cargo-chef
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Builder
FROM rust:1.77-slim as builder
RUN cargo install cargo-chef
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --target {target}
COPY . .
RUN cargo build --release --target {target}

# Stage 3: Runtime
FROM scratch
COPY --from=builder /app/target/{target}/release/{binary_name} /{binary_name}
ENTRYPOINT ["/{binary_name}"]
"#,
            target = target,
            binary_name = binary_name,
        )
    }
}

/// Represents a compiled artifact with its metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledArtifact {
    pub target: String,
    pub binary_path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub is_stripped: bool,
    pub is_static: bool,
}

impl CompiledArtifact {
    /// Create a new artifact (simulated for tests).
    pub fn new(target: &str, binary_path: &str, size_bytes: u64) -> Self {
        Self {
            target: target.into(),
            binary_path: binary_path.into(),
            size_bytes,
            sha256: format!("{:064x}", size_bytes), // simplified hash for example
            is_stripped: false,
            is_static: false,
        }
    }

    /// Return human-readable size.
    pub fn human_size(&self) -> String {
        if self.size_bytes >= 1_073_741_824 {
            format!("{:.2} GB", self.size_bytes as f64 / 1_073_741_824.0)
        } else if self.size_bytes >= 1_048_576 {
            format!("{:.2} MB", self.size_bytes as f64 / 1_048_576.0)
        } else if self.size_bytes >= 1024 {
            format!("{:.2} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{} B", self.size_bytes)
        }
    }
}

/// Helper to strip debug symbols from a binary.
pub fn strip_command(binary_path: &str, target_os: &OsFamily) -> String {
    match target_os {
        OsFamily::Linux | OsFamily::MacOS => format!("strip {}", binary_path),
        OsFamily::Windows => format!("strip --strip-all {}", binary_path),
        _ => format!("# stripping not supported for {:?}", target_os),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_registry_has_common_targets() {
        let registry = TargetRegistry::new();
        assert!(registry.get("x86_64-unknown-linux-gnu").is_some());
        assert!(registry.get("x86_64-unknown-linux-musl").is_some());
        assert!(registry.get("aarch64-unknown-linux-gnu").is_some());
        assert!(registry.get("x86_64-apple-darwin").is_some());
        assert!(registry.get("aarch64-apple-darwin").is_some());
        assert!(registry.get("x86_64-pc-windows-msvc").is_some());
        assert!(registry.get("wasm32-unknown-unknown").is_some());
        assert!(registry.get("nonexistent-target").is_none());
    }

    #[test]
    fn test_target_properties() {
        let registry = TargetRegistry::new();
        let musl = registry.get("x86_64-unknown-linux-musl").unwrap();
        assert!(musl.supports_static);
        assert_eq!(musl.libc, Some(Libc::Musl));
        assert_eq!(musl.arch, Architecture::X86_64);

        let gnu = registry.get("x86_64-unknown-linux-gnu").unwrap();
        assert!(!gnu.supports_static);
        assert_eq!(gnu.libc, Some(Libc::Gnu));
    }

    #[test]
    fn test_list_by_os() {
        let registry = TargetRegistry::new();
        let linux_targets = registry.list_by_os(&OsFamily::Linux);
        assert!(linux_targets.len() >= 3); // gnu, musl, aarch64

        let macos_targets = registry.list_by_os(&OsFamily::MacOS);
        assert_eq!(macos_targets.len(), 2); // intel, arm64
    }

    #[test]
    fn test_cross_config_commands() {
        let cross = CrossConfig::for_cross("aarch64-unknown-linux-gnu");
        assert!(cross.build_command().contains("cross"));
        assert!(cross.build_command().contains("aarch64-unknown-linux-gnu"));

        let cargo = CrossConfig::for_cargo("x86_64-unknown-linux-gnu");
        assert!(cargo.build_command().contains("cargo"));
        assert!(cargo.test_command().contains("cargo test"));
    }

    #[test]
    fn test_generate_cross_toml() {
        let configs = vec![
            CrossConfig::for_cross("aarch64-unknown-linux-gnu"),
            CrossConfig {
                target: "x86_64-unknown-linux-musl".into(),
                use_cross: true,
                docker_image: Some("custom/image:latest".into()),
                env: HashMap::new(),
                pre_build: None,
            },
        ];

        let toml = generate_cross_toml(&configs);
        assert!(toml.contains("[target.aarch64-unknown-linux-gnu]"));
        assert!(toml.contains("[target.x86_64-unknown-linux-musl]"));
        assert!(toml.contains("custom/image:latest"));
    }

    #[test]
    fn test_generate_cargo_config() {
        let registry = TargetRegistry::new();
        let targets = vec![
            registry.get("x86_64-unknown-linux-musl").unwrap().clone(),
            registry.get("aarch64-unknown-linux-gnu").unwrap().clone(),
        ];

        let config = generate_cargo_config(&targets);
        assert!(config.contains("[target.x86_64-unknown-linux-musl]"));
        assert!(config.contains("linker = \"musl-gcc\""));
        assert!(config.contains("[target.aarch64-unknown-linux-gnu]"));
        assert!(config.contains("linker = \"aarch64-linux-gnu-gcc\""));
    }

    #[test]
    fn test_build_matrix_release() {
        let matrix = BuildMatrixEntry::release_matrix();
        assert_eq!(matrix.len(), 6);

        let linux_static = matrix
            .iter()
            .find(|e| e.target == "x86_64-unknown-linux-musl")
            .unwrap();
        assert!(linux_static.cross);

        let native_linux = matrix
            .iter()
            .find(|e| e.target == "x86_64-unknown-linux-gnu")
            .unwrap();
        assert!(!native_linux.cross);
    }

    #[test]
    fn test_dockerfile_multi_stage() {
        let dockerfile = DockerfileGenerator::multi_stage("myapp", "x86_64-unknown-linux-musl");
        assert!(dockerfile.contains("FROM rust:"));
        assert!(dockerfile.contains("FROM scratch"));
        assert!(dockerfile.contains("myapp"));
        assert!(dockerfile.contains("x86_64-unknown-linux-musl"));
    }

    #[test]
    fn test_dockerfile_with_chef() {
        let dockerfile = DockerfileGenerator::with_chef("myapp", "x86_64-unknown-linux-gnu");
        assert!(dockerfile.contains("cargo-chef"));
        assert!(dockerfile.contains("cargo chef prepare"));
        assert!(dockerfile.contains("cargo chef cook"));
    }

    #[test]
    fn test_compiled_artifact_size_formatting() {
        let small = CompiledArtifact::new("x86_64-unknown-linux-gnu", "/bin/app", 512);
        assert_eq!(small.human_size(), "512 B");

        let medium = CompiledArtifact::new("x86_64-unknown-linux-gnu", "/bin/app", 1536);
        assert_eq!(medium.human_size(), "1.50 KB");

        let large = CompiledArtifact::new("x86_64-unknown-linux-gnu", "/bin/app", 5_242_880);
        assert_eq!(large.human_size(), "5.00 MB");
    }

    #[test]
    fn test_strip_command() {
        let cmd = strip_command("/bin/app", &OsFamily::Linux);
        assert!(cmd.contains("strip"));

        let cmd = strip_command("/bin/app.exe", &OsFamily::Windows);
        assert!(cmd.contains("strip"));
    }

    #[test]
    fn test_wasm_target() {
        let registry = TargetRegistry::new();
        let wasm = registry.get("wasm32-unknown-unknown").unwrap();
        assert_eq!(wasm.os, OsFamily::Wasm);
        assert_eq!(wasm.arch, Architecture::Wasm32);
        assert!(wasm.supports_static);
    }
}
