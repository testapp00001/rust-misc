# Module 24: Secure Deployment

Build it right, ship it safely, respond fast when things go wrong.

## Why This Module Matters

A perfectly written application is still vulnerable if the deployment pipeline is compromised.
Attackers target build systems, container images, CI/CD pipelines, and supply chains because
one compromise there gives them access to every downstream system. This module teaches you to
defend the entire path from source code to running production service.

## Learning Path

| # | Lesson | Core Concept |
|---|--------|--------------|
| 01 | Reproducible Builds | Deterministic compilation, build environment isolation |
| 02 | Container Security | Minimal base images, no root, read-only filesystem |
| 03 | SBOM in CI | Track all dependencies, license compliance |
| 04 | Release Signing | Sign binaries, verify before deployment |
| 05 | Build Attestation | SLSA levels, provenance verification |
| 06 | Secret Injection in CI | Don't bake secrets into images |
| 07 | Image Scanning | CVE detection, base image updates |
| 08 | Supply Chain Verification | Verify dependency integrity |
| 09 | Deployment Hardening | Least privilege, network policy, seccomp |
| 10 | Incident Response | Rollback, hotfix, communication |

## Reproducible Builds

A build is **reproducible** if anyone with the same source code and build environment can
produce bit-identical output. This is critical because:

- **Verification**: Anyone can verify that a binary was built from the claimed source
- **Detection**: Non-reproducible builds may indicate injected malware
- **Auditability**: Regulators and auditors can verify software provenance

### What Breaks Reproducibility

- Timestamps embedded in binaries
- Random build IDs (e.g., Go's `-buildid`)
- Absolute paths (use `--remap-path-prefix` in Rust)
- Non-deterministic parallel code generation
- Locale-dependent behavior

### Rust Reproducible Build Flags

```bash
# Remap source paths for deterministic debug info
RUSTFLAGS="--remap-path-prefix=$(pwd)=/src" cargo build --release

# Use a locked Cargo.lock
cargo build --locked --release

# Pin the toolchain
rustup override set 1.75.0
```

## Container Security

### The Problem

Default container configurations are insecure:
- Running as root (UID 0)
- Full filesystem access
- All Linux capabilities granted
- No resource limits
- Large attack surface from bloated base images

### Defense: Minimal Container Build

```dockerfile
# Multi-stage build: compile in full image, run in scratch
FROM rust:1.75-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --locked

# scratch has NO shell, NO libraries, NO attack surface
FROM scratch
COPY --from=builder /app/target/release/myapp /myapp
USER 65534:65534
ENTRYPOINT ["/myapp"]
```

### Key Principles

1. **No root**: Always `USER 65534` (nobody) or a dedicated UID
2. **Read-only filesystem**: `--read-only` with tmpfs for writable paths
3. **Drop capabilities**: `--cap-drop=ALL`, add back only what's needed
4. **No shell**: Use `scratch` or `distroless` base images
5. **Resource limits**: Set `--memory`, `--cpus`, `--pids-limit`

## SLSA Framework

**SLSA** (Supply-chain Levels for Software Artifacts) is a security framework that
gradually increases the integrity guarantees of a build pipeline.

| Level | Requirement | What It Prevents |
|-------|------------|-----------------|
| 1 | Build process documented | Accidental misconfigurations |
| 2 | Hosted build platform, signed provenance | Tampering by individual developers |
| 3 | Hardened build platform, non-falsifiable provenance | Compromise of build platform |
| 4 | Hermetic, reproducible builds | All above + two-person review |

### Provenance

Provenance is metadata describing how an artifact was built:
- Source repository and commit
- Build instructions and environment
- Builder identity
- Timestamps

```json
{
  "builder": { "id": "https://github.com/actions/runner" },
  "buildType": "https://actions.github.io/buildtypes/workflow/v1",
  "invocation": {
    "configSource": {
      "uri": "git+https://github.com/org/repo@refs/heads/main",
      "digest": { "sha1": "abc123..." }
    }
  },
  "materials": [
    { "uri": "git+https://github.com/org/dep.git", "digest": { "sha1": "def456..." } }
  ]
}
```

## Release Signing

### Why Sign Releases

- **Authenticity**: Users can verify the binary came from you
- **Integrity**: Any modification after signing is detectable
- **Non-repudiation**: You cannot deny having published a signed release

### Signing Workflow

```
1. Build binary
2. Compute SHA-256 digest
3. Sign digest with private key (Ed25519 or RSA)
4. Publish: binary + signature + public key
5. User: verify signature against public key before running
```

### Key Management for Signing

- Store signing keys in HSMs or cloud KMS (never in CI config)
- Use separate keys per environment (dev, staging, prod)
- Rotate keys annually
- Keep an offline root key for disaster recovery

## Secret Injection in CI

### The Anti-Pattern: Baked Secrets

```dockerfile
# NEVER DO THIS
ENV API_KEY=sk-1234567890abcdef
COPY credentials.json /app/
```

Secrets in Docker images are:
- Visible in `docker history`
- Stored in every layer forever
- Accessible to anyone with image access
- Logged in CI build logs

### The Right Way: Runtime Injection

```bash
# Pass secrets as files mounted at runtime
docker run --secret id=api_key,target=/run/secrets/api_key myapp

# Or use environment variables from a secret manager
docker run -e API_KEY="$(vault kv get -field=key secret/myapp)" myapp
```

## Incident Response

When a deployment goes wrong, speed matters. Follow this framework:

1. **Detect**: Monitoring alerts on anomalous behavior
2. **Triage**: Determine severity and blast radius
3. **Contain**: Roll back to last known good deployment
4. **Eradicate**: Fix the root cause
5. **Recover**: Deploy the fix, verify normal operation
6. **Learn**: Post-mortem, update runbooks, improve automation

### Rollback Decision Tree

```
Is production down?
  YES -> Roll back immediately, fix later
  NO  -> Is data integrity at risk?
    YES -> Roll back immediately
    NO  -> Is security compromised?
      YES -> Roll back, rotate credentials
      NO  -> Monitor, fix in next release
```

## Running Tests

```bash
# Test exercise stubs (should compile, tests will fail with todo!())
cargo test -p 24-secure-deployment

# Test reference solutions
cargo test -p 24-secure-deployment --features solution
```

## Dependencies

- `ring` -- Cryptographic operations (signing, hashing)
- `sha2` -- SHA-256 hashing for artifact digests
- `serde` / `serde_json` -- Serialization for SBOM, provenance, and policy documents
