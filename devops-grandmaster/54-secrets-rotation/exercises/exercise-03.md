# Exercise 03: Zero-Downtime Rotation Strategy

**Type:** Independent
**Difficulty:** Intermediate
**Estimated Time:** 50 minutes

## Objective

Design and implement a zero-downtime secret rotation system where multiple consumers can
continue operating during a rotation event without any request failures.

## Background

In production, many services consume the same secret simultaneously. A naive rotation
approach (delete old, create new) causes failures for any consumer that has not yet
picked up the new secret. Zero-downtime rotation requires:

1. A **dual-secret window** where both old and new secrets are valid.
2. A **notification mechanism** so consumers learn about the new secret.
3. A **drain period** where the old secret is phased out gracefully.

## Instructions

### Part A: Design Document

Before writing any code, create a file `design.md` answering these questions:

1. Draw (in ASCII or describe) the timeline of a rotation event with these phases:
   - Phase 1: Normal operation (only old secret active)
   - Phase 2: Dual validity (both secrets active)
   - Phase 3: New secret only (old secret revoked)

2. How long should Phase 2 last? What factors determine this?

3. What happens if a consumer crashes during Phase 2 and restarts? How does it discover
   which secret to use?

4. How do you handle the case where the secret store itself is unavailable during
   rotation?

### Part B: Implementation

Build a `DualSecretStore` in Rust with these requirements:

```
cargo new zero_downtime_rotator
cd zero_downtime_rotator
```

**Dependencies (`Cargo.toml`):**

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
rand = "0.8"
uuid = { version = "1", features = ["v4"] }
```

**Core struct:**

```rust
struct DualSecretStore {
    active: SecretEntry,
    retiring: Option<SecretEntry>,
    listeners: Vec<tokio::sync::broadcast::Sender<RotationEvent>>,
}

struct SecretEntry {
    id: String,
    value: Vec<u8>,
    created_at: chrono::DateTime<chrono::Utc>,
    status: SecretStatus,
}

enum SecretStatus {
    Active,
    Retiring,
    Revoked,
}

enum RotationEvent {
    RotationStarted { new_id: String, old_id: String },
    RotationCompleted { current_id: String },
    GracePeriodExpired { revoked_id: String },
}
```

Implement:

1. `rotate()` -- generate a new secret, move the current active to `retiring`, broadcast
   `RotationStarted`.
2. `validate(value: &[u8]) -> bool` -- check against both active and retiring secrets.
3. `revoke_retired()` -- revoke the retiring secret, broadcast `GracePeriodExpired`.
4. `subscribe() -> broadcast::Receiver<RotationEvent>` -- allow consumers to subscribe
   to rotation events.
5. `get_active_secret() -> &SecretEntry` -- return the currently active secret.

### Part C: Consumer Simulation

Write an async consumer that:

1. Subscribes to rotation events.
2. On `RotationStarted`, fetches the new secret and begins using it for new requests.
3. Continues accepting the old secret for in-flight requests.
4. On `GracePeriodExpired`, drops all knowledge of the old secret.

Simulate 3 concurrent consumers, each processing 100 "requests" (just validation calls).
Trigger a rotation at request 50. Verify zero failures across all consumers.

### Part D: Edge Cases

Write tests for these scenarios:

1. Two rotations happen in rapid succession before the grace period of the first expires.
2. A consumer subscribes after a rotation has already started.
3. The secret value is empty (should reject).

## Success Criteria

- [ ] `design.md` has clear, correct answers for all four questions.
- [ ] `rotate()` broadcasts events and maintains dual secrets correctly.
- [ ] `validate()` accepts values from both active and retiring secrets.
- [ ] `revoke_retired()` correctly revokes and broadcasts.
- [ ] Consumer simulation shows zero request failures during rotation.
- [ ] All edge case tests pass.

## Hints

<details>
<summary>Hint 1: Broadcast channel</summary>

`tokio::sync::broadcast::channel(capacity)` creates a sender and receiver. Clone the
sender for each subscriber. The capacity should be large enough to buffer events if
consumers are slow.

</details>

<details>
<summary>Hint 2: Rapid successive rotations</summary>

When a second rotation happens during the first grace period, the first "retiring" secret
should be immediately revoked -- it is no longer relevant. Only the most recent previous
secret should remain in the retiring state.

</details>

<details>
<summary>Hint 3: Late subscriber</summary>

A subscriber who joins after `RotationStarted` will not receive that event. Consider
having `get_active_secret()` return enough information for a late subscriber to
synchronize its state.

</details>
