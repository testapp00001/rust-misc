//! # Docker Patterns for Rust Applications
//!
//! Docker containerization for Rust requires special consideration due to the
//! compilation model, static linking capabilities, and the large build context.
//! This module covers multi-stage builds, minimal base images, and caching
//! strategies specific to Rust.
//!
//! ## Key Patterns:
//!
//! 1. **Multi-stage builds**: Build in a full Rust image, copy binary to scratch/distroless
//! 2. **cargo-chef**: Separate dependency compilation from source compilation
//! 3. **Layer caching**: Order Dockerfile instructions to maximize cache hits
//! 4. **scratch images**: Zero-overhead containers (just the binary)
//! 5. **distroless**: Minimal images with CA certs and timezone data
//!
//! ## Size Comparison:
//!
//! | Base Image | Approx Size |
//! |------------|-------------|
//! | `rust:1.77` | ~1.5 GB |
//! | `debian:bookworm-slim` | ~75 MB |
//! | `alpine:3.19` | ~7 MB |
//! | `gcr.io/distroless/cc` | ~20 MB |
//! | `scratch` | 0 MB |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a Docker image configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    pub name: String,
    pub tag: String,
    pub registry: Option<String>,
    pub digest: Option<String>,
}

impl DockerImage {
    pub fn new(name: &str, tag: &str) -> Self {
        Self {
            name: name.to_string(),
            tag: tag.to_string(),
            registry: None,
            digest: None,
        }
    }

    pub fn with_registry(mut self, registry: &str) -> Self {
        self.registry = Some(registry.to_string());
        self
    }

    /// Get the full image reference.
    pub fn full_ref(&self) -> String {
        let prefix = self
            .registry
            .as_ref()
            .map(|r| format!("{}/", r))
            .unwrap_or_default();
        let digest_suffix = self
            .digest
            .as_ref()
            .map(|d| format!("@{}", d))
            .unwrap_or_default();
        format!("{}{}:{}{}", prefix, self.name, self.tag, digest_suffix)
    }
}

/// Multi-stage Dockerfile builder for Rust applications.
#[derive(Debug, Clone)]
pub struct DockerfileBuilder {
    stages: Vec<BuildStage>,
    binary_name: String,
    target_triple: Option<String>,
    use_chef: bool,
    runtime_base: RuntimeBase,
    extra_runtime_files: Vec<String>,
    exposed_ports: Vec<u16>,
    health_check: Option<String>,
    user: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildStage {
    pub name: String,
    pub base_image: String,
    pub instructions: Vec<DockerInstruction>,
}

#[derive(Debug, Clone)]
pub enum DockerInstruction {
    WorkDir(String),
    Run(String),
    Copy { from: Option<String>, src: String, dest: String },
    Env(String, String),
    Expose(u16),
    User(String),
    Arg(String),
}

#[derive(Debug, Clone)]
pub enum RuntimeBase {
    Scratch,
    Distroless,
    Alpine,
    DebianSlim,
    Custom(String),
}

impl RuntimeBase {
    pub fn image_ref(&self) -> &str {
        match self {
            Self::Scratch => "scratch",
            Self::Distroless => "gcr.io/distroless/cc-debian12:nonroot",
            Self::Alpine => "alpine:3.19",
            Self::DebianSlim => "debian:bookworm-slim",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn estimated_size_mb(&self) -> f64 {
        match self {
            Self::Scratch => 0.0,
            Self::Distroless => 20.0,
            Self::Alpine => 7.0,
            Self::DebianSlim => 75.0,
            Self::Custom(_) => 50.0,
        }
    }
}

impl DockerfileBuilder {
    pub fn new(binary_name: &str) -> Self {
        Self {
            stages: Vec::new(),
            binary_name: binary_name.to_string(),
            target_triple: None,
            use_chef: false,
            runtime_base: RuntimeBase::Distroless,
            extra_runtime_files: Vec::new(),
            exposed_ports: Vec::new(),
            health_check: None,
            user: None,
        }
    }

    pub fn target(mut self, triple: &str) -> Self {
        self.target_triple = Some(triple.to_string());
        self
    }

    pub fn use_chef(mut self, enable: bool) -> Self {
        self.use_chef = enable;
        self
    }

    pub fn runtime_base(mut self, base: RuntimeBase) -> Self {
        self.runtime_base = base;
        self
    }

    pub fn expose_port(mut self, port: u16) -> Self {
        self.exposed_ports.push(port);
        self
    }

    pub fn health_check(mut self, check: &str) -> Self {
        self.health_check = Some(check.to_string());
        self
    }

    pub fn run_as_user(mut self, user: &str) -> Self {
        self.user = Some(user.to_string());
        self
    }

    pub fn copy_to_runtime(mut self, path: &str) -> Self {
        self.extra_runtime_files.push(path.to_string());
        self
    }

    /// Build the complete Dockerfile string.
    pub fn build(&self) -> String {
        let target = self
            .target_triple
            .as_deref()
            .unwrap_or("x86_64-unknown-linux-musl");
        let mut output = String::new();

        if self.use_chef {
            output.push_str(&self.build_chef_stages(target));
        } else {
            output.push_str(&self.build_simple_stages(target));
        }

        output
    }

    fn build_simple_stages(&self, target: &str) -> String {
        let binary_path = format!("target/{}/release/{}", target, self.binary_name);
        let mut out = String::new();

        // Builder stage
        out.push_str("# Stage 1: Build\n");
        out.push_str("FROM rust:1.77-slim AS builder\n\n");
        out.push_str("RUN apt-get update && apt-get install -y pkg-config && rm -rf /var/lib/apt/lists/*\n");
        out.push_str("RUN rustup target add ");
        out.push_str(target);
        out.push('\n');
        out.push_str("WORKDIR /app\n\n");

        // Cache dependencies
        out.push_str("# Cache dependencies\n");
        out.push_str("COPY Cargo.toml Cargo.lock ./\n");
        out.push_str("RUN mkdir src && echo 'fn main() {}' > src/main.rs\n");
        out.push_str("RUN cargo build --release --target ");
        out.push_str(target);
        out.push_str(" 2>/dev/null || true\n");
        out.push_str("RUN rm -rf src\n\n");

        // Build real binary
        out.push_str("# Build application\n");
        out.push_str("COPY src ./src\n");
        out.push_str("RUN touch src/main.rs\n");
        out.push_str("RUN cargo build --release --target ");
        out.push_str(target);
        out.push_str("\n\n");

        // Runtime stage
        out.push_str("# Stage 2: Runtime\n");
        out.push_str("FROM ");
        out.push_str(self.runtime_base.image_ref());
        out.push_str("\n\n");

        for file in &self.extra_runtime_files {
            out.push_str(&format!("COPY --from=builder {} {}\n", file, file));
        }

        out.push_str("COPY --from=builder /app/");
        out.push_str(&binary_path);
        out.push_str(" /usr/local/bin/");
        out.push_str(&self.binary_name);
        out.push('\n');

        for port in &self.exposed_ports {
            out.push_str(&format!("EXPOSE {}\n", port));
        }

        if let Some(ref user) = self.user {
            out.push_str(&format!("USER {}\n", user));
        }

        if let Some(ref check) = self.health_check {
            out.push_str(&format!("HEALTHCHECK CMD {}\n", check));
        }

        out.push_str(&format!("ENTRYPOINT [\"/usr/local/bin/{}\"]\n", self.binary_name));
        out
    }

    fn build_chef_stages(&self, target: &str) -> String {
        let binary_path = format!("target/{}/release/{}", target, self.binary_name);
        let mut out = String::new();

        // Planner stage
        out.push_str("# Stage 1: Planner\n");
        out.push_str("FROM rust:1.77-slim AS planner\n");
        out.push_str("RUN cargo install cargo-chef\n");
        out.push_str("WORKDIR /app\n");
        out.push_str("COPY . .\n");
        out.push_str("RUN cargo chef prepare --recipe-path recipe.json\n\n");

        // Builder stage
        out.push_str("# Stage 2: Builder\n");
        out.push_str("FROM rust:1.77-slim AS builder\n");
        out.push_str("RUN cargo install cargo-chef\n");
        out.push_str("RUN rustup target add ");
        out.push_str(target);
        out.push('\n');
        out.push_str("WORKDIR /app\n");
        out.push_str("COPY --from=planner /app/recipe.json recipe.json\n");
        out.push_str("RUN cargo chef cook --release --recipe-path recipe.json --target ");
        out.push_str(target);
        out.push_str("\nCOPY . .\n");
        out.push_str("RUN cargo build --release --target ");
        out.push_str(target);
        out.push_str("\n\n");

        // Runtime stage
        out.push_str("# Stage 3: Runtime\n");
        out.push_str("FROM ");
        out.push_str(self.runtime_base.image_ref());
        out.push_str("\n\n");

        out.push_str("COPY --from=builder /app/");
        out.push_str(&binary_path);
        out.push_str(" /usr/local/bin/");
        out.push_str(&self.binary_name);
        out.push('\n');

        for port in &self.exposed_ports {
            out.push_str(&format!("EXPOSE {}\n", port));
        }

        if let Some(ref user) = self.user {
            out.push_str(&format!("USER {}\n", user));
        }

        if let Some(ref check) = self.health_check {
            out.push_str(&format!("HEALTHCHECK CMD {}\n", check));
        }

        out.push_str(&format!("ENTRYPOINT [\"/usr/local/bin/{}\"]\n", self.binary_name));
        out
    }
}

/// .dockerignore file generator for Rust projects.
pub fn generate_dockerignore() -> String {
    r#"# Git
.git
.gitignore

# Rust build artifacts (rebuild in container)
target/

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# CI
.github/
.gitlab-ci.yml

# Documentation (not needed for build)
docs/
*.md
LICENSE

# Docker
Dockerfile
docker-compose*.yml
.dockerignore
"#
    .to_string()
}

/// Docker Compose configuration for a Rust service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerComposeService {
    pub name: String,
    pub build_context: String,
    pub dockerfile: String,
    pub ports: Vec<(u16, u16)>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<String>,
    pub depends_on: Vec<String>,
    pub restart_policy: String,
    pub health_check: Option<HealthCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub test: Vec<String>,
    pub interval_secs: u32,
    pub timeout_secs: u32,
    pub retries: u32,
    pub start_period_secs: u32,
}

impl DockerComposeService {
    pub fn to_yaml(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("  {}:\n", self.name));
        out.push_str(&format!("    build:\n      context: {}\n      dockerfile: {}\n",
            self.build_context, self.dockerfile));

        if !self.ports.is_empty() {
            out.push_str("    ports:\n");
            for (host, container) in &self.ports {
                out.push_str(&format!("      - \"{}:{}\"\n", host, container));
            }
        }

        if !self.environment.is_empty() {
            out.push_str("    environment:\n");
            for (key, value) in &self.environment {
                out.push_str(&format!("      {}: \"{}\"\n", key, value));
            }
        }

        if !self.volumes.is_empty() {
            out.push_str("    volumes:\n");
            for vol in &self.volumes {
                out.push_str(&format!("      - {}\n", vol));
            }
        }

        if !self.depends_on.is_empty() {
            out.push_str("    depends_on:\n");
            for dep in &self.depends_on {
                out.push_str(&format!("      - {}\n", dep));
            }
        }

        out.push_str(&format!("    restart: {}\n", self.restart_policy));

        if let Some(ref hc) = self.health_check {
            out.push_str("    healthcheck:\n");
            out.push_str(&format!("      test: {:?}\n", hc.test));
            out.push_str(&format!("      interval: {}s\n", hc.interval_secs));
            out.push_str(&format!("      timeout: {}s\n", hc.timeout_secs));
            out.push_str(&format!("      retries: {}\n", hc.retries));
            out.push_str(&format!("      start_period: {}s\n", hc.start_period_secs));
        }

        out
    }
}

/// Image size estimation based on build configuration.
pub fn estimate_image_size(
    binary_size_mb: f64,
    runtime_base: &RuntimeBase,
    include_ca_certs: bool,
    include_tz_data: bool,
) -> f64 {
    let mut total = binary_size_mb + runtime_base.estimated_size_mb();
    if include_ca_certs && matches!(runtime_base, RuntimeBase::Scratch) {
        total += 0.5; // ~500KB for CA certs
    }
    if include_tz_data && matches!(runtime_base, RuntimeBase::Scratch) {
        total += 1.0; // ~1MB for timezone data
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_image_full_ref() {
        let img = DockerImage::new("rust", "1.77");
        assert_eq!(img.full_ref(), "rust:1.77");

        let img = DockerImage::new("myapp", "latest")
            .with_registry("ghcr.io/owner");
        assert_eq!(img.full_ref(), "ghcr.io/owner/myapp:latest");
    }

    #[test]
    fn test_dockerfile_builder_simple() {
        let builder = DockerfileBuilder::new("myapp")
            .target("x86_64-unknown-linux-musl")
            .runtime_base(RuntimeBase::Scratch)
            .expose_port(8080)
            .run_as_user("nonroot");

        let dockerfile = builder.build();
        assert!(dockerfile.contains("FROM rust:"));
        assert!(dockerfile.contains("FROM scratch"));
        assert!(dockerfile.contains("myapp"));
        assert!(dockerfile.contains("EXPOSE 8080"));
        assert!(dockerfile.contains("USER nonroot"));
        assert!(dockerfile.contains("ENTRYPOINT"));
    }

    #[test]
    fn test_dockerfile_builder_with_chef() {
        let builder = DockerfileBuilder::new("myapp")
            .use_chef(true)
            .target("x86_64-unknown-linux-gnu")
            .runtime_base(RuntimeBase::Distroless);

        let dockerfile = builder.build();
        assert!(dockerfile.contains("cargo-chef"));
        assert!(dockerfile.contains("cargo chef prepare"));
        assert!(dockerfile.contains("cargo chef cook"));
        assert!(dockerfile.contains("distroless"));
    }

    #[test]
    fn test_dockerfile_builder_with_health_check() {
        let builder = DockerfileBuilder::new("myapp")
            .health_check("curl -f http://localhost:8080/health || exit 1");

        let dockerfile = builder.build();
        assert!(dockerfile.contains("HEALTHCHECK"));
        assert!(dockerfile.contains("/health"));
    }

    #[test]
    fn test_dockerfile_builder_distroless() {
        let builder = DockerfileBuilder::new("myapp")
            .runtime_base(RuntimeBase::Distroless);

        let dockerfile = builder.build();
        assert!(dockerfile.contains("distroless"));
    }

    #[test]
    fn test_dockerfile_builder_alpine() {
        let builder = DockerfileBuilder::new("myapp")
            .runtime_base(RuntimeBase::Alpine);

        let dockerfile = builder.build();
        assert!(dockerfile.contains("alpine"));
    }

    #[test]
    fn test_generate_dockerignore() {
        let ignore = generate_dockerignore();
        assert!(ignore.contains("target/"));
        assert!(ignore.contains(".git"));
        assert!(ignore.contains("Dockerfile"));
        assert!(ignore.contains(".dockerignore"));
    }

    #[test]
    fn test_runtime_base_size_estimates() {
        assert_eq!(RuntimeBase::Scratch.estimated_size_mb(), 0.0);
        assert!(RuntimeBase::Distroless.estimated_size_mb() > 0.0);
        assert!(RuntimeBase::Alpine.estimated_size_mb() > 0.0);
        assert!(RuntimeBase::DebianSlim.estimated_size_mb() > RuntimeBase::Alpine.estimated_size_mb());
    }

    #[test]
    fn test_runtime_base_image_refs() {
        assert_eq!(RuntimeBase::Scratch.image_ref(), "scratch");
        assert!(RuntimeBase::Distroless.image_ref().contains("distroless"));
        assert!(RuntimeBase::Alpine.image_ref().contains("alpine"));
        assert!(RuntimeBase::DebianSlim.image_ref().contains("debian"));
    }

    #[test]
    fn test_docker_compose_service_yaml() {
        let service = DockerComposeService {
            name: "api".into(),
            build_context: ".".into(),
            dockerfile: "Dockerfile".into(),
            ports: vec![(8080, 8080)],
            environment: {
                let mut m = HashMap::new();
                m.insert("RUST_LOG".into(), "info".into());
                m
            },
            volumes: vec!["./data:/data".into()],
            depends_on: vec!["postgres".into()],
            restart_policy: "unless-stopped".into(),
            health_check: Some(HealthCheck {
                test: vec!["CMD".into(), "curl".into(), "-f".into(), "http://localhost:8080/health".into()],
                interval_secs: 30,
                timeout_secs: 10,
                retries: 3,
                start_period_secs: 10,
            }),
        };

        let yaml = service.to_yaml();
        assert!(yaml.contains("api:"));
        assert!(yaml.contains("8080:8080"));
        assert!(yaml.contains("RUST_LOG"));
        assert!(yaml.contains("postgres"));
        assert!(yaml.contains("healthcheck"));
        assert!(yaml.contains("unless-stopped"));
    }

    #[test]
    fn test_estimate_image_size() {
        let size = estimate_image_size(10.0, &RuntimeBase::Scratch, true, true);
        // 10MB binary + 0.5MB CA certs + 1MB tz data = 11.5
        assert!((size - 11.5).abs() < f64::EPSILON);

        let size = estimate_image_size(10.0, &RuntimeBase::Distroless, false, false);
        // 10MB binary + 20MB distroless = 30
        assert!((size - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dockerfile_builder_with_extra_files() {
        let builder = DockerfileBuilder::new("myapp")
            .copy_to_runtime("/etc/ssl/certs/ca-certificates.crt")
            .copy_to_runtime("/usr/share/zoneinfo");

        let dockerfile = builder.build();
        assert!(dockerfile.contains("ca-certificates.crt"));
        assert!(dockerfile.contains("zoneinfo"));
    }

    #[test]
    fn test_chef_stages_distinct_from_simple() {
        let simple = DockerfileBuilder::new("myapp").build();
        let chef = DockerfileBuilder::new("myapp").use_chef(true).build();

        assert!(!simple.contains("cargo-chef"));
        assert!(chef.contains("cargo-chef"));

        // Chef has 3 stages, simple has 2
        let chef_stages = chef.matches("FROM ").count();
        let simple_stages = simple.matches("FROM ").count();
        assert_eq!(chef_stages, 3);
        assert_eq!(simple_stages, 2);
    }
}
