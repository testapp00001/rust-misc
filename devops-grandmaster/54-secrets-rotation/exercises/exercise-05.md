# Exercise 05: Secrets Rotation in CI/CD Pipeline

**Type:** Integration
**Difficulty:** Advanced
**Estimated Time:** 60 minutes

## Objective

Integrate secrets rotation into a CI/CD pipeline so that deploying an application
automatically triggers secret rotation, injects the new secrets, and validates the
deployment without exposing secrets in logs or build artifacts.

## Background

CI/CD pipelines often need secrets for:

- Authenticating to container registries.
- Connecting to deployment targets (Kubernetes, cloud providers).
- Accessing databases during migration steps.
- Signing artifacts.

If these secrets are static and stored in CI/CD platform variables, a compromise of the
CI/CD system gives the attacker long-lived credentials. Rotation after every deploy
limits this exposure.

## Instructions

### Part A: Pipeline Design

Create a file `pipeline-design.md` that describes the following:

1. **Pre-deploy phase**: How does the pipeline obtain the current secrets?
2. **Deploy phase**: How are secrets injected into the application without being logged?
3. **Post-deploy phase**: How does the pipeline trigger rotation and verify the new
   secrets work?
4. **Rollback phase**: If the deploy fails, should the rotation be reverted? Why or why
   not?

Draw an ASCII diagram of the pipeline stages.

### Part B: Build the Rotation Trigger

Create a Rust CLI tool that acts as the rotation trigger:

```bash
cargo new rotation-trigger
cd rotation-trigger
```

**Dependencies:**

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
hex = "0.4"
```

The CLI should support these subcommands:

```
rotation-trigger pre-deploy --env staging --secret-path database/creds
rotation-trigger deploy --env staging --app myapp
rotation-trigger post-deploy --env staging --secret-path database/creds
rotation-trigger rollback --env staging --secret-path database/creds
```

Implement:

1. `pre-deploy`: Reads the current secret from a secret store, writes it to a
   **temporary file** (not stdout, not environment variables that might be logged). Prints
   only the secret's ID and a checksum (SHA-256) for verification.

2. `deploy`: Triggers the application deploy. Does NOT handle secrets directly -- it
   assumes the deployment mechanism reads from the temporary file.

3. `post-deploy`: Generates a new secret, writes it to the secret store as a new version,
   updates the temporary file, and verifies the application can connect with the new
   secret (by hitting a health check endpoint that reports which secret version it is
   using).

4. `rollback`: Reverts to the previous secret version in the store and updates the
   temporary file.

### Part C: Security Guards

Implement these security measures in your CLI:

1. **No secret in stdout**: Any function that handles raw secret values must NOT print
   them. Use a `SecretString` wrapper type that redacts its `Debug` and `Display` output.

2. **File permissions**: The temporary secret file must be created with mode `0600`
   (owner read/write only). Use `std::os::unix::fs::OpenOptionsExt`.

3. **Automatic cleanup**: The temporary file is deleted when the CLI process exits.
   Implement a `Drop` guard or use `ctrlc` / signal handlers.

4. **Audit logging**: Every rotation event writes an audit entry to a log file with:
   timestamp, action, environment, secret ID (not value), and result (success/failure).

### Part D: End-to-End Test

Write an integration test that simulates a full pipeline run:

1. Pre-deploy: reads secret, asserts checksum is printed, raw value is NOT in stdout.
2. Deploy: mock succeeds.
3. Post-deploy: new secret is written, old version is still accessible, application health
   check passes.
4. Verify audit log has exactly 2 entries (pre-deploy read, post-deploy rotation).
5. Verify the temporary file is cleaned up after the process exits.

## Success Criteria

- [ ] `pipeline-design.md` has a clear, correct pipeline diagram and answers.
- [ ] CLI handles all four subcommands correctly.
- [ ] `SecretString` type redacts output in `Debug` and `Display`.
- [ ] Temporary files are created with `0600` permissions.
- [ ] Temporary files are cleaned up on exit.
- [ ] Audit log captures all rotation events with correct metadata.
- [ ] End-to-end test passes with zero secret leaks to stdout.

## Hints

<details>
<summary>Hint 1: SecretString wrapper</summary>

```rust
struct SecretString(String);

impl std::fmt::Debug for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl std::fmt::Display for SecretString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED]")
    }
}
```

</details>

<details>
<summary>Hint 2: File permissions on Unix</summary>

```rust
use std::os::unix::fs::OpenOptionsExt;
let file = std::fs::OpenOptions::new()
    .write(true)
    .create(true)
    .mode(0o600)
    .open(&path)?;
```

</details>

<details>
<summary>Hint 3: Cleanup on exit</summary>

Register a cleanup function using `ctrlc` crate or use a `Drop` implementation on a guard
struct that holds the file path. For tests, use `tempfile::NamedTempFile` which
auto-deletes.

</details>

<details>
<summary>Hint 4: Verifying no leaks in tests</summary>

Capture stdout in your test by running the CLI as a subprocess with
`std::process::Command` and checking the output. Search for known secret substrings to
assert they are absent.

</details>
