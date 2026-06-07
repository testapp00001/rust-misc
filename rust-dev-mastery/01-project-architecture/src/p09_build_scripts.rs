//! # Lesson 9: Build Scripts (build.rs)
//!
//! Build scripts run before compilation and can generate code, link native
//! libraries, set environment variables, and enable conditional compilation.
//! This lesson covers build.rs patterns and environment variables.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents what a build script can do.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildScript {
    pub name: String,
    pub actions: Vec<BuildAction>,
    pub environment: BTreeMap<String, String>,
    pub cfg_flags: Vec<String>,
    pub links: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuildAction {
    /// Generate Rust source code from a template.
    CodeGeneration {
        input: String,
        output: String,
        generator: String,
    },
    /// Link a native C library.
    LinkNativeLib {
        name: String,
        kind: LinkKind,
        search_path: Option<String>,
    },
    /// Re-run build script if file changes.
    RerunIfChanged { path: String },
    /// Set an environment variable for compilation.
    SetEnv { key: String, value: String },
    /// Set a cfg flag.
    SetCfg { flag: String },
    /// Print a cargo instruction.
    CargoInstruction { instruction: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkKind {
    Static,
    Dynamic,
    Framework,
}

impl BuildScript {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            actions: Vec::new(),
            environment: BTreeMap::new(),
            cfg_flags: Vec::new(),
            links: None,
        }
    }

    pub fn generate_code(
        &mut self,
        input: &str,
        output: &str,
        generator: &str,
    ) {
        self.actions.push(BuildAction::CodeGeneration {
            input: input.to_string(),
            output: output.to_string(),
            generator: generator.to_string(),
        });
    }

    pub fn link_lib(&mut self, name: &str, kind: LinkKind, search: Option<&str>) {
        self.actions.push(BuildAction::LinkNativeLib {
            name: name.to_string(),
            kind,
            search_path: search.map(|s| s.to_string()),
        });
    }

    pub fn rerun_if_changed(&mut self, path: &str) {
        self.actions.push(BuildAction::RerunIfChanged {
            path: path.to_string(),
        });
    }

    pub fn set_env(&mut self, key: &str, value: &str) {
        self.environment.insert(key.to_string(), value.to_string());
        self.actions.push(BuildAction::SetEnv {
            key: key.to_string(),
            value: value.to_string(),
        });
    }

    pub fn set_cfg(&mut self, flag: &str) {
        self.cfg_flags.push(flag.to_string());
        self.actions.push(BuildAction::SetCfg {
            flag: flag.to_string(),
        });
    }

    pub fn set_links(&mut self, lib: &str) {
        self.links = Some(lib.to_string());
    }

    /// Generate the build.rs source code.
    pub fn to_rust_source(&self) -> String {
        let mut lines = vec![
            "fn main() {".to_string(),
            "    // Build script generated from BuildScript definition".to_string(),
        ];

        for action in &self.actions {
            match action {
                BuildAction::CodeGeneration {
                    input,
                    output,
                    generator,
                } => {
                    lines.push(format!("    // Generate code from {}", input));
                    lines.push(format!(
                        "    let content = std::fs::read_to_string(\"{}\").unwrap();",
                        input
                    ));
                    lines.push(format!(
                        "    let generated = {}::generate(&content);",
                        generator
                    ));
                    lines.push(format!(
                        "    std::fs::write(\"{}\", generated).unwrap();",
                        output
                    ));
                }
                BuildAction::LinkNativeLib {
                    name,
                    kind,
                    search_path,
                } => {
                    let kind_str = match kind {
                        LinkKind::Static => "static",
                        LinkKind::Dynamic => "dylib",
                        LinkKind::Framework => "framework",
                    };
                    if let Some(path) = search_path {
                        lines.push(format!(
                            "    println!(\"cargo:rustc-link-search=native={}\");",
                            path
                        ));
                    }
                    lines.push(format!(
                        "    println!(\"cargo:rustc-link-lib={}\");",
                        kind_str
                    ));
                    lines.push(format!(
                        "    println!(\"cargo:rustc-link-lib={}\");",
                        name
                    ));
                }
                BuildAction::RerunIfChanged { path } => {
                    lines.push(format!(
                        "    println!(\"cargo:rerun-if-changed={}\");",
                        path
                    ));
                }
                BuildAction::SetEnv { key, value } => {
                    lines.push(format!(
                        "    println!(\"cargo:rustc-env={}={}\");",
                        key, value
                    ));
                }
                BuildAction::SetCfg { flag } => {
                    lines.push(format!(
                        "    println!(\"cargo:rustc-cfg={}\");",
                        flag
                    ));
                }
                BuildAction::CargoInstruction { instruction } => {
                    lines.push(format!(
                        "    println!(\"cargo:{}\");",
                        instruction
                    ));
                }
            }
        }

        lines.push("}".to_string());
        lines.join("\n")
    }
}

/// Environment variables available to build scripts.
/// These are set by Cargo before build.rs runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildEnvironment {
    pub cargo_manifest_dir: String,
    pub out_dir: String,
    pub target: String,
    pub target_arch: String,
    pub target_os: String,
    pub host: String,
    pub num_jobs: u32,
    pub opt_level: String,
    pub profile: String,
}

impl BuildEnvironment {
    /// Create a build environment with typical values.
    pub fn typical() -> Self {
        Self {
            cargo_manifest_dir: "/home/user/project".to_string(),
            out_dir: "/home/user/project/target/debug/build/myproject-out".to_string(),
            target: "x86_64-unknown-linux-gnu".to_string(),
            target_arch: "x86_64".to_string(),
            target_os: "linux".to_string(),
            host: "x86_64-unknown-linux-gnu".to_string(),
            num_jobs: 8,
            opt_level: "0".to_string(),
            profile: "debug".to_string(),
        }
    }

    /// Check if we're cross-compiling.
    pub fn is_cross_compiling(&self) -> bool {
        self.target != self.host
    }

    /// Check if targeting a specific OS.
    pub fn is_targeting(&self, os: &str) -> bool {
        self.target_os == os
    }

    /// Get the output directory for generated files.
    pub fn generated_files_dir(&self) -> String {
        format!("{}/generated", self.out_dir)
    }
}

/// Represents a code generator that build.rs might invoke.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGenerator {
    pub name: String,
    pub input_formats: Vec<String>,
    pub output_format: String,
}

impl CodeGenerator {
    pub fn new(
        name: impl Into<String>,
        inputs: Vec<impl Into<String>>,
        output: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            input_formats: inputs.into_iter().map(|i| i.into()).collect(),
            output_format: output.into(),
        }
    }

    /// Check if this generator can handle the given input file.
    pub fn can_handle(&self, filename: &str) -> bool {
        let ext = filename.rsplit('.').next().unwrap_or("");
        self.input_formats.iter().any(|fmt| fmt == ext)
    }

    /// Generate output filename from input.
    pub fn output_filename(&self, input: &str) -> String {
        format!("{}.{}", self.name, self.output_format)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_script_creation() {
        let bs = BuildScript::new("test-build");
        assert_eq!(bs.name, "test-build");
        assert!(bs.actions.is_empty());
    }

    #[test]
    fn test_build_script_generate_code() {
        let mut bs = BuildScript::new("test");
        bs.generate_code("schema.json", "schema.rs", "schemars");
        assert_eq!(bs.actions.len(), 1);
    }

    #[test]
    fn test_build_script_link_lib() {
        let mut bs = BuildScript::new("test");
        bs.link_lib("ssl", LinkKind::Dynamic, Some("/usr/lib"));
        bs.link_lib("crypto", LinkKind::Static, None);
        assert_eq!(bs.actions.len(), 2);
    }

    #[test]
    fn test_build_script_rerun_if_changed() {
        let mut bs = BuildScript::new("test");
        bs.rerun_if_changed("build.rs");
        bs.rerun_if_changed("schema.json");
        assert_eq!(bs.actions.len(), 2);
    }

    #[test]
    fn test_build_script_set_env() {
        let mut bs = BuildScript::new("test");
        bs.set_env("VERSION", "1.0.0");
        assert!(bs.environment.contains_key("VERSION"));
        assert_eq!(bs.environment["VERSION"], "1.0.0");
    }

    #[test]
    fn test_build_script_set_cfg() {
        let mut bs = BuildScript::new("test");
        bs.set_cfg("has_feature_x");
        assert!(bs.cfg_flags.contains(&"has_feature_x".to_string()));
    }

    #[test]
    fn test_build_script_to_rust_source() {
        let mut bs = BuildScript::new("test");
        bs.link_lib("z", LinkKind::Static, None);
        bs.rerun_if_changed("build.rs");

        let source = bs.to_rust_source();
        assert!(source.contains("fn main()"));
        assert!(source.contains("cargo:rustc-link-lib"));
        assert!(source.contains("cargo:rerun-if-changed"));
    }

    #[test]
    fn test_build_script_code_generation_source() {
        let mut bs = BuildScript::new("test");
        bs.generate_code("templates/api.hbs", "src/generated_api.rs", "handlebars");

        let source = bs.to_rust_source();
        assert!(source.contains("templates/api.hbs"));
        assert!(source.contains("src/generated_api.rs"));
        assert!(source.contains("handlebars::generate"));
    }

    #[test]
    fn test_build_script_set_env_source() {
        let mut bs = BuildScript::new("test");
        bs.set_env("COMPILE_TIME", "2024-01-01");

        let source = bs.to_rust_source();
        assert!(source.contains("cargo:rustc-env=COMPILE_TIME=2024-01-01"));
    }

    #[test]
    fn test_build_script_set_cfg_source() {
        let mut bs = BuildScript::new("test");
        bs.set_cfg("feature_x");

        let source = bs.to_rust_source();
        assert!(source.contains("cargo:rustc-cfg=feature_x"));
    }

    #[test]
    fn test_build_environment_typical() {
        let env = BuildEnvironment::typical();
        assert_eq!(env.target, "x86_64-unknown-linux-gnu");
        assert_eq!(env.target_os, "linux");
        assert_eq!(env.profile, "debug");
    }

    #[test]
    fn test_build_environment_cross_compiling() {
        let env = BuildEnvironment::typical();
        assert!(!env.is_cross_compiling());

        let mut cross_env = BuildEnvironment::typical();
        cross_env.target = "aarch64-unknown-linux-gnu".to_string();
        assert!(cross_env.is_cross_compiling());
    }

    #[test]
    fn test_build_environment_is_targeting() {
        let env = BuildEnvironment::typical();
        assert!(env.is_targeting("linux"));
        assert!(!env.is_targeting("windows"));
    }

    #[test]
    fn test_build_environment_generated_dir() {
        let env = BuildEnvironment::typical();
        let dir = env.generated_files_dir();
        assert!(dir.ends_with("/generated"));
    }

    #[test]
    fn test_code_generator() {
        let gen = CodeGenerator::new("protobuf", vec!["proto", "protobuf"], "rs");
        assert!(gen.can_handle("api.proto"));
        assert!(gen.can_handle("messages.protobuf"));
        assert!(!gen.can_handle("api.json"));
    }

    #[test]
    fn test_code_generator_output_filename() {
        let gen = CodeGenerator::new("proto", vec!["proto"], "rs");
        assert_eq!(gen.output_filename("api.proto"), "proto.rs");
    }

    #[test]
    fn test_link_kind_variants() {
        let mut bs = BuildScript::new("test");
        bs.link_lib("a", LinkKind::Static, None);
        bs.link_lib("b", LinkKind::Dynamic, None);
        bs.link_lib("c", LinkKind::Framework, None);
        assert_eq!(bs.actions.len(), 3);
    }

    #[test]
    fn test_build_script_set_links() {
        let mut bs = BuildScript::new("test");
        bs.set_links("z");
        assert_eq!(bs.links, Some("z".to_string()));
    }
}
