---
name: production-audit
description: >
  This skill should be used when the user asks to audit a project for production readiness,
  review security posture, prepare for release, or run a comprehensive code quality check.
  It performs a multi-dimensional audit covering security, code quality, dependencies,
  testing, build/CI, documentation, and production hardening.
version: 1.0.0
argument-hint: "[--focus security|quality|testing|deps|docs|all]"
allowed-tools:
  - Bash
  - Read
  - Agent
  - LSP
---

# Production Readiness Audit

You are performing a comprehensive production-readiness audit of this project.
Your goal is to identify every issue that could cause problems in production —
from critical security vulnerabilities to minor code quality nits.

## Audit Dimensions

Run ALL of the following audit dimensions unless the user specifies `--focus`:

### 1. CRITICAL — Tests & Build Health
- Run `cargo test --workspace` and report ALL failures
- Run `cargo clippy --workspace` with `-D warnings` and report ALL warnings
- Run `cargo build --workspace` and verify clean compilation
- Check for `#[allow(unused)]`, `todo!()`, `unimplemented!()`, `panic!()` in production code
- Verify all tests pass in isolation AND together (no test interdependence / race conditions)

### 2. CRITICAL — Security Audit
- **Cryptography**: Review all crypto code for proper algorithm choices, key sizes, nonce handling,
  authentication tag verification, and timing-safe comparisons
- **Memory Safety**: Verify `zeroize`/`ZeroizeOnDrop` on all sensitive types, check for copies
  of secrets that aren't zeroized, verify no secrets leak in logs/errors/Debug impls
- **Input Validation**: Check all user-facing inputs (CLI args, HTTP requests, Tauri commands,
  WASM exports) for proper validation and sanitization
- **Authentication & Authorization**: Review JWT implementation, session management, token expiry,
  and authorization checks on all protected endpoints
- **Secrets in Code**: Scan for hardcoded secrets, API keys, default passwords, or credentials
- **SQL Injection**: Verify all database queries use parameterized statements
- **Error Messages**: Ensure error messages don't leak sensitive information to users
- **Timing Attacks**: Check for constant-time comparison of secrets (passwords, tokens, MACs)

### 3. HIGH — Dependency Audit
- Run `cargo audit` if available, or manually check for known vulnerabilities
- Check all dependency versions are pinned (not wildcard `*`)
- Identify outdated dependencies that need updating
- Check for unnecessary dependencies that increase attack surface
- Verify `Cargo.lock` is committed and up to date

### 4. HIGH — Code Quality
- Check for `unwrap()` / `expect()` in production code (not tests) — these should be `Result`
- Check for `panic!()` / `todo!()` / `unimplemented!()` in production code
- Look for dead code, unused imports, unused variables
- Check error handling: are errors properly propagated with context?
- Look for code duplication (especially desktop/mobile Tauri apps)
- Verify consistent coding style and naming conventions
- Check for proper `#[must_use]` on important return types
- Look for potential integer overflow, division by zero
- Check async code for proper cancellation safety

### 5. HIGH — Production Hardening
- **Logging**: Verify structured logging is in place, no secrets in logs, appropriate log levels
- **Rate Limiting**: Check server rate limiting configuration, login attempt limits
- **Configuration**: Verify no hardcoded production values, proper environment variable handling
- **Graceful Shutdown**: Check server handles SIGTERM/SIGINT properly
- **Resource Limits**: Check file descriptor limits, connection pool sizes, memory limits
- **Health Checks**: Verify `/health` endpoint exists and is meaningful
- **Error Recovery**: Check what happens on corrupted vault data, disk full, network failures
- **Concurrency**: Check for race conditions, proper mutex usage, deadlock potential

### 6. MEDIUM — Testing Gaps
- Map test coverage: which modules/functions have tests, which don't?
- Check for edge case coverage: empty inputs, very large inputs, unicode, boundary values
- Verify error path testing: are error conditions tested?
- Check for integration tests beyond unit tests
- Look for missing negative tests (wrong password, corrupted data, etc.)
- Verify test isolation (no shared state between tests)

### 7. MEDIUM — Documentation & Release
- Check README completeness (build instructions, usage, security model)
- Verify CHANGELOG exists
- Check for API documentation (rustdoc)
- Verify LICENSE file exists
- Check for CONTRIBUTING.md
- Verify all public items in library crates have doc comments
- Check CLI `--help` output quality

### 8. MEDIUM — Build & CI
- Check for CI/CD configuration (.github/workflows)
- Verify reproducible builds
- Check for release automation (versioning, changelog, publishing)
- Verify cross-platform build support
- Check for Docker/container support if applicable

## Output Format

After completing the audit, produce a structured report:

```
# Production Readiness Audit Report
Generated: <date>
Project: <name> v<version>

## Executive Summary
<one paragraph — is this project production-ready? what's blocking it?>

## Critical Issues (must fix before release)
| # | Category | File | Issue | Recommendation |
|---|----------|------|-------|----------------|

## High Priority (should fix before release)
| # | Category | File | Issue | Recommendation |
|---|----------|------|-------|----------------|

## Medium Priority (fix soon)
| # | Category | File | Issue | Recommendation |
|---|----------|------|-------|----------------|

## Low Priority (nice to have)
| # | Category | File | Issue | Recommendation |
|---|----------|------|-------|----------------|

## Test Results Summary
- Unit tests: X passed / Y failed
- Integration tests: X passed / Y failed
- Clippy warnings: N
- Build: clean / errors

## Security Score: X/10
## Code Quality Score: X/10
## Test Coverage Score: X/10
## Documentation Score: X/10
## Production Readiness Score: X/10

## Recommended Action Plan
1. <most important fix>
2. <next most important>
...
```

## Execution

Begin the audit immediately. Run tests and clippy first (they reveal the most actionable issues).
Then deep-dive into source code for security and quality issues.
Read every file in the project — don't skip anything.
Be thorough. Be specific. Every finding must include the exact file path and line number.
