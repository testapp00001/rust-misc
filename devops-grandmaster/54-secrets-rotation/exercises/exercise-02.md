# Exercise 02: Implement Basic Secret Rotation

**Type:** Guided
**Difficulty:** Intermediate
**Estimated Time:** 40 minutes

## Objective

Build a basic secret rotation mechanism in Rust that generates, stores, and rotates a
shared secret on a configurable schedule, maintaining backward compatibility during the
transition window.

## Background

The simplest rotation pattern uses an overlapping validity window: the old secret remains
valid for a grace period after the new secret is issued. This ensures that in-flight
requests using the old secret are not rejected during rotation.

## Instructions

### Step 1: Set Up the Project

Create a new Rust project:

```bash
cargo new secret_rotator
cd secret_rotator
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
chrono = "0.4"
rand = "0.8"
sha2 = "0.10"
hex = "0.4"
```

### Step 2: Define the Secret Structure

Create a struct that represents a secret with its metadata:

```rust
struct Secret {
    id: String,
    value: Vec<u8>,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: chrono::DateTime<chrono::Utc>,
    is_active: bool,
}
```

### Step 3: Implement the Rotator

Create a `SecretRotator` struct with the following methods:

```rust
struct SecretRotator {
    current: Option<Secret>,
    previous: Option<Secret>,
    rotation_interval: chrono::Duration,
    grace_period: chrono::Duration,
}
```

Implement these methods:

1. `new(rotation_interval, grace_period)` -- constructor.
2. `generate_secret()` -- generate a new random 32-byte secret, return a `Secret`.
3. `rotate()` -- generate a new secret, move current to previous, set the new one as
   current. Set `expires_at` on the old secret to `now + grace_period`.
4. `validate(value: &[u8]) -> bool` -- return `true` if the value matches either the
   current or the previous (and the previous has not expired).
5. `needs_rotation() -> bool` -- return `true` if the current secret's `created_at` plus
   `rotation_interval` is in the past.

### Step 4: Add a Rotation Loop

Write a `main` function that:

1. Creates a `SecretRotator` with a 30-second rotation interval and a 10-second grace
   period.
2. Enters a loop that checks `needs_rotation()` every second.
3. When rotation is needed, calls `rotate()` and prints the new secret's ID and creation
   timestamp.
4. After each check, prints whether a hypothetical old secret value still validates.
5. Exits after 3 rotations.

### Step 5: Test Validation Across Rotation

Write a test that:

1. Creates a `SecretRotator`.
2. Generates and stores the current secret's value.
3. Rotates.
4. Asserts the old value still validates (grace period).
5. Simulates the grace period expiring (adjust `expires_at` to the past).
6. Asserts the old value no longer validates.

## Success Criteria

- [ ] `SecretRotator` generates cryptographically random 32-byte secrets.
- [ ] `rotate()` maintains exactly two secrets: current and previous.
- [ ] `validate()` accepts both current and unexpired previous secrets.
- [ ] `validate()` rejects expired previous secrets.
- [ ] `needs_rotation()` correctly triggers based on elapsed time.
- [ ] The test in Step 5 passes.

## Hints

<details>
<summary>Hint 1: Generating random bytes</summary>

Use `rand::Rng` and call `rng.gen::<[u8; 32]>()` to produce 32 random bytes.

</details>

<details>
<summary>Hint 2: Time comparison</summary>

Use `chrono::Utc::now()` to get the current time and compare with `secret.created_at +
rotation_interval` using the `>` operator.

</details>

<details>
<summary>Hint 3: Simulating expired grace period in tests</summary>

When constructing a `Secret` for testing, set `expires_at` to
`chrono::Utc::now() - chrono::Duration::seconds(1)` to simulate an already-expired
secret.

</details>

<details>
<summary>Hint 4: Validation logic</summary>

```rust
fn validate(&self, value: &[u8]) -> bool {
    if let Some(ref s) = self.current {
        if s.value == value { return true; }
    }
    if let Some(ref s) = self.previous {
        if s.value == value && s.expires_at > chrono::Utc::now() {
            return true;
        }
    }
    false
}
```

</details>
