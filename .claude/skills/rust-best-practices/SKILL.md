---
name: rust-best-practices
description: >
  This skill should be used when writing, reviewing, or fixing Rust code.
  It provides battle-tested rules learned from auditing a production password
  manager — covering security, memory safety, error handling, testing, and
  build configuration. Use it to avoid the 87 mistakes found in audit.
version: 1.0.0
---

# Rust Best Practices — Lessons from Production Audit

Every rule below was found as an actual bug or vulnerability in this project.
Each entry includes the audit finding ID so you can cross-reference `AUDIT_REPORT.md`.

---

## 1. SECRETS & CRYPTO (Critical)

### 1.1 NEVER hardcode secrets — fail at startup instead
```rust
// BAD (C1): fallback to known value
let jwt_secret = std::env::var("JWT_SECRET")
    .unwrap_or_else(|_| "change-me".into());

// GOOD: fail hard if not set
let jwt_secret = std::env::var("JWT_SECRET")
    .expect("JWT_SECRET must be set");
```
**Why:** A hardcoded default means every deployment without the env var is trivially compromised.

### 1.2 Verify authentication before allowing state changes
```rust
// BAD (C2/C3): ignore the old password parameter
fn change_password(_old_password: String, new_password: String, ...) {
    vault.change_password(&new_password)?;
}

// GOOD: verify old password first
fn change_password(old_password: String, new_password: String, ...) {
    Vault::open(&old_password, &vault_path)
        .map_err(|_| "Current password is incorrect")?;
    vault.change_password(&new_password)?;
}
```
**Why:** An underscore prefix on a parameter is a code smell — it means the value is accepted but ignored.

### 1.3 Use ZeroizeOnDrop on ALL types holding secrets
```rust
// BAD (H1-H5): String persists in heap memory
fn get_entry(&self, id: &str) -> Result<(Entry, String), VaultError> {
    // String with decrypted password — never zeroized
    Ok((entry, password))
}

// GOOD: return zeroizing wrapper
use zeroize::Zeroizing;
fn get_entry(&self, id: &str) -> Result<(Entry, Zeroizing<String>), VaultError> {
    Ok((entry, Zeroizing::new(password)))
}
```
**Why:** Rust `String` has no `Zeroize` impl. Decrypted secrets linger in heap memory indefinitely.

### 1.4 Zeroize intermediate buffers immediately after use
```rust
// BAD (H4): plaintext survives re-encryption
let plaintext = old_crypto.decrypt(&ciphertext)?;
let new_ciphertext = new_crypto.encrypt(&plaintext)?;
// plaintext still in memory here

// GOOD: zeroize immediately
let mut plaintext = old_crypto.decrypt(&ciphertext)?;
let new_ciphertext = new_crypto.encrypt(&plaintext)?;
plaintext.zeroize();
```

### 1.5 Never log secrets, password hashes, or PII
```rust
// BAD (M5-M6): Debug impl leaks password hash
#[derive(Debug)]
struct UserRow { password_hash: String, email: String }

// GOOD: custom Debug with redaction
impl std::fmt::Debug for UserRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserRow")
            .field("email", &"[redacted]")
            .field("password_hash", &"[redacted]")
            .finish()
    }
}
```

### 1.6 Use constant-time comparison for secrets
```rust
// BAD (M3): timing side-channel
if self.hash == expected_hash { /* ... */ }

// GOOD: constant-time
use subtle::ConstantTimeEq;
if self.hash.ct_eq(&expected_hash).into() { /* ... */ }
```

### 1.7 Add domain separation to KDF derivations
```rust
// BAD (M2): raw concatenation
let key = blake3::hash(&[master_key, entry_id].concat());

// GOOD: domain-separated
let mut hasher = blake3::Hasher::new();
hasher.update(b"vault-entry-key-v1");
hasher.update(&master_key);
hasher.update(entry_id.as_bytes());
let key = hasher.finalize();
```

### 1.8 Add `#![forbid(unsafe_code)]` to crypto crates
```rust
// In lib.rs of any crate handling secrets
#![forbid(unsafe_code)]
```
**Why:** Prevents accidental introduction of unsafe code. If no unsafe exists today, lock it out.

---

## 2. FILE I/O (Critical)

### 2.1 Atomic writes — never overwrite in place
```rust
// BAD (C8): crash during write = data loss
std::fs::write(&path, data)?;

// GOOD: write to temp, then atomic rename
let temp_path = path.with_extension("tmp");
std::fs::write(&temp_path, data)?;
std::fs::rename(&temp_path, &path)?;
```
**Why:** Power failure, disk full, or crash during `write()` corrupts the file. The rename is atomic on POSIX.

### 2.2 Set restrictive file permissions on sensitive data
```rust
// BAD (C7): default umask (0644) — world-readable
std::fs::write(&path, data)?;

// GOOD: owner-only permissions (Unix)
std::fs::write(&temp_path, data)?;
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o600))?;
}
std::fs::rename(&temp_path, &path)?;
```

### 2.3 Set directory permissions too
```rust
// BAD: default umask on directory
std::fs::create_dir_all(&path)?;

// GOOD: owner-only directory
std::fs::create_dir_all(&path)?;
#[cfg(unix)]
{
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700))?;
}
```

---

## 3. NETWORK I/O (Critical)

### 3.1 ALWAYS validate length prefixes before allocating
```rust
// BAD (H8): unbounded allocation — 4GB OOM crash
let len = u32::from_be_bytes(buf);
let mut data = vec![0u8; len as usize]; // attacker sends 0xFFFFFFFF

// GOOD: validate first
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024; // 10MB
let len = u32::from_be_bytes(buf) as usize;
if len > MAX_MESSAGE_SIZE {
    return Err(Error::Protocol("message too large".into()));
}
let mut data = vec![0u8; len];
```

### 3.2 Use `write_all` / `read_exact` — never `try_write` / `try_read`
```rust
// BAD (H9): may write/read fewer bytes than requested
stream.try_write(&data)?;

// GOOD: guaranteed complete I/O
use tokio::io::AsyncWriteExt;
stream.write_all(&data).await?;
```
**Why:** Non-blocking I/O may complete partially. Without looping, messages are silently truncated.

### 3.3 Add request body size limits to servers
```rust
// BAD (H20): no limit — attacker sends gigabytes
Router::new().route("/api/data", post(handler))

// GOOD: bounded
use tower_http::limit::RequestBodyLimitLayer;
Router::new()
    .route("/api/data", post(handler))
    .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10MB
```

### 3.4 Add request timeouts
```rust
// BAD (H22): slowloris — client holds connection forever
axum::serve(listener, app).await?;

// GOOD: timeout layer
use tower::timeout::TimeoutLayer;
Router::new()
    .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)))
```

---

## 4. ERROR HANDLING (High)

### 4.1 No `unwrap()` / `expect()` in production code (except main)
```rust
// BAD (M21): poisoned mutex = cascade panics
let store = self.data.lock().unwrap();

// GOOD: handle gracefully
let store = self.data.lock()
    .map_err(|e| VaultError::InvalidData(format!("lock poisoned: {}", e)))?;
```
**Exception:** `expect()` is acceptable in `fn main()` for truly unrecoverable setup failures.

### 4.2 Never flatten errors to String
```rust
// BAD (M13): loses error type — frontend can't distinguish
vault.save().map_err(|e| e.to_string())?;

// GOOD: structured error enum
#[derive(Serialize)]
enum AppError { WrongPassword, VaultLocked, Internal(String) }
impl From<VaultError> for AppError { /* ... */ }
```

### 4.3 Don't swallow errors in boolean checks
```rust
// BAD (4.3): hash parse failure returns Ok(false), same as wrong password
fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    Ok(argon2::verify_password(password.as_bytes(), &hash).is_ok())
}

// GOOD: distinguish parse failure from mismatch
fn verify_password(password: &str, hash: &str) -> Result<bool, Error> {
    match argon2::verify_password(password.as_bytes(), &hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(e.into()),
    }
}
```

### 4.4 Propagate directory read errors, don't silently skip
```rust
// BAD (4.4): silently ignores failed entries
.filter_map(|e| e.ok())

// GOOD: collect and report errors
for entry in entries {
    let entry = entry?;
    // ...
}
```

---

## 5. TESTING (High)

### 5.1 Test isolation — never share paths between tests
```rust
// BAD: shared path = race condition in parallel tests
fn test_vault_path() -> String {
    "/tmp/.vault_test".to_string()
}

// GOOD: unique temp directory per test
fn test_vault_path() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}
```
**Why:** `cargo test` runs tests in parallel. Shared paths cause "No such file or directory" errors.

### 5.2 Test specific error variants, not just `is_err()`
```rust
// BAD: doesn't verify which error occurred
assert!(result.is_err());

// GOOD: verify the specific error
assert!(matches!(result.unwrap_err(), VaultError::WrongPassword));
```

### 5.3 Test edge cases that break assumptions
Always test:
- Empty inputs (`""`)
- Unicode in all string fields (emoji, CJK, RTL)
- Null bytes in strings (`\0`)
- Very large inputs (10MB+)
- Boundary values (exactly 12 bytes for nonce-only, etc.)
- Special characters (`!@#$%^&*`, quotes, backslashes)

### 5.4 Use `tempfile::TempDir` — it auto-cleans on drop
```rust
#[test]
fn test_save_and_load() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().to_str().unwrap();
    // ... test code ...
    // tmp is automatically cleaned up when it goes out of scope
}
```

### 5.5 Assert the invariant, not just the result
```rust
// BAD: doesn't explain what's being tested
assert!(vault.is_ok());

// GOOD: descriptive assertion
assert!(vault.is_ok(), "Vault creation should succeed with valid password");
```

---

## 6. SECURITY PATTERNS (High)

### 6.1 CORS — never use `permissive()` in production
```rust
// BAD (C6): any website can make authenticated requests
CorsLayer::permissive()

// GOOD: configurable, restrictive by default
let cors = if let Ok(origins) = std::env::var("CORS_ORIGINS") {
    CorsLayer::new()
        .allow_origin(origins.split(',').filter_map(|o| o.trim().parse().ok()))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
} else {
    tracing::warn!("CORS_ORIGINS not set — using restrictive defaults");
    CorsLayer::new().allow_origin(same_origin)
};
```

### 6.2 Rate limiting on auth endpoints
```rust
use tower::limit::RateLimitLayer;
use std::time::Duration;

// Apply stricter limits to auth routes
let auth_routes = Router::new()
    .route("/login", post(login))
    .route("/register", post(register))
    .layer(RateLimitLayer::new(5, Duration::from_secs(60))); // 5/min
```

### 6.3 Health checks must verify actual dependencies
```rust
// BAD (H21): always returns OK
async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

// GOOD: checks database connectivity
async fn health_check(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    sqlx::query("SELECT 1").execute(&state.db).await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(json!({ "status": "ok" })))
}
```

### 6.4 Graceful shutdown — drain in-flight requests
```rust
use tokio::signal;

axum::serve(listener, app)
    .with_graceful_shutdown(shutdown_signal())
    .await?;

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
```

### 6.5 Validate all user input lengths
```rust
// BAD (M17): no upper bound — Argon2 degrades with very long passwords
if password.is_empty() { return Err(...); }

// GOOD: bounded
if password.len() < 8 || password.len() > 128 {
    return Err("Password must be 8-128 characters".into());
}
if email.len() > 254 {  // RFC 5321
    return Err("Email too long".into());
}
```

---

## 7. BUILD & DEPENDENCIES (Medium)

### 7.1 Always define a release profile
```toml
# In workspace root Cargo.toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
panic = "abort"
```
**Why:** Without this, release binaries are larger, slower, and contain debug symbols that leak info.

### 7.2 Centralize workspace dependencies
```toml
# BAD: each crate pins its own versions
[dependencies]
uuid = "1"
chrono = "0.4"

# GOOD: workspace-level
[workspace.dependencies]
uuid = "1"
chrono = "0.4"

# In member Cargo.toml
[dependencies]
uuid = { workspace = true }
chrono = { workspace = true }
```

### 7.3 Specify exact tokio features, not `full`
```toml
# BAD: enables everything including unused features
tokio = { version = "1", features = ["full"] }

# GOOD: only what you need
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net", "sync", "time"] }
```

### 7.4 Pin crypto-critical dependencies to minor versions
```toml
# BAD: broad range could get a breaking minor
chacha20poly1305 = "0.10"

# GOOD: pin to exact minor
chacha20poly1305 = "=0.10.1"
```

### 7.5 Always have CI with these checks
```yaml
# .github/workflows/ci.yml minimum checks:
- cargo fmt --all -- --check
- cargo clippy --workspace --all-targets -- -D warnings
- cargo test --workspace --all-targets
- cargo test --workspace --doc
- cargo audit  # security vulnerabilities
```

---

## 8. CODE ORGANIZATION (Medium)

### 8.1 Never duplicate code across crates — extract shared module
```rust
// BAD (H13): mobile/src/lib.rs is verbatim copy of desktop/src/main.rs (~600 lines)
// Every bug must be fixed in two places

// GOOD: extract to shared crate
// crates/vault-tauri-common/src/lib.rs
// Contains: AppState, response types, all Tauri commands
// Both desktop and mobile depend on it
```

### 8.2 Don't reimplement crypto in WASM — feature-gate the core crate
```toml
# In vault-core/Cargo.toml
[features]
default = ["std"]
std = []
wasm = ["wasm-bindgen"]

# In vault-wasm/Cargo.toml
[dependencies]
vault-core = { path = "../../crates/vault-core", features = ["wasm"] }
```

### 8.3 Stub commands are worse than missing commands
```rust
// BAD (H12): user thinks vault is locked but it's not
fn cmd_lock() {
    println!("Vault locked"); // does nothing
}

// GOOD: fail honestly
fn cmd_lock() {
    eprintln!("Lock not supported in CLI mode — vault key is not held in memory");
    std::process::exit(1);
}
```

---

## 9. QUICK REFERENCE CHECKLIST

Before committing Rust code, verify:

- [ ] No `unwrap()` in production code (except `main()`)
- [ ] No hardcoded secrets — all from env vars with fail-on-missing
- [ ] All sensitive types implement `ZeroizeOnDrop`
- [ ] File writes are atomic (temp + rename)
- [ ] File permissions set to 0600 for secrets, 0700 for directories
- [ ] Network reads use `read_exact`, writes use `write_all`
- [ ] Length prefixes validated before allocation
- [ ] Errors propagated with `?`, never flattened to `String` in libraries
- [ ] No `Debug` impl on types containing secrets
- [ ] Tests use unique temp directories (no shared paths)
- [ ] Tests verify specific error variants, not just `is_err()`
- [ ] CORS is configurable, not `permissive()`
- [ ] Auth endpoints have rate limiting
- [ ] Health checks verify actual dependencies
- [ ] Release profile has LTO, strip, codegen-units=1
- [ ] `#![forbid(unsafe_code)]` on crypto crates
