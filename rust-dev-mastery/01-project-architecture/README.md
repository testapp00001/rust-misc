# Module 1: Project Architecture

Master Rust project organization from small crates to large monorepos.
Learn workspace design, crate organization, feature flags, build profiles,
dependency management, build scripts, and release engineering.

## Lesson Index

| # | File | Topic |
|---|------|-------|
| 1 | `p01_workspace_design.rs` | Multi-crate workspace patterns, when to split crates, shared deps |
| 2 | `p02_crate_organization.rs` | lib vs bin crates, multiple binaries, crate structure |
| 3 | `p03_module_system_mastery.rs` | Module tree, visibility, re-exports, glob imports, mod.rs vs filename.rs |
| 4 | `p04_feature_flags.rs` | Cargo features, optional deps, feature unification, cfg attributes |
| 5 | `p05_build_profiles.rs` | Release vs dev, custom profiles, opt-level, LTO, codegen-units |
| 6 | `p06_cargo_toml_deep_dive.rs` | All Cargo.toml sections, package metadata, targets, patches, replace |
| 7 | `p07_dependency_management.rs` | SemVer, pinning, cargo audit, cargo tree, dependency resolution |
| 8 | `p08_project_templates.rs` | Project scaffolding, cargo-generate, standard layouts, conventions |
| 9 | `p09_build_scripts.rs` | build.rs, code generation, linking, environment variables, conditional compilation |
| 10 | `p10_release_engineering.rs` | Versioning strategies, changelog, cargo release, publishing, tagging |

## Running

```bash
cargo test -p project_architecture
```
