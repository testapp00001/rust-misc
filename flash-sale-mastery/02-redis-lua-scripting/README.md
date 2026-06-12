# Module 02: Redis Lua Scripting for Flash Sales

## Motivation

A flash sale purchase requires **four atomic guarantees**:

1. **Stock**: inventory must never go negative
2. **Claim**: each account may only purchase once per product
3. **Voucher**: the product's voucher cap must not be exceeded
4. **Idempotency**: duplicate request IDs must return the same result

If these are four separate Redis commands, a crash between step 2 and step 3
leaves the system in an inconsistent state -- the user is marked as having
claimed, but no stock was decremented. The only way to make all four steps
atomic is to execute them inside a single Lua script that Redis runs
indistinguishably from a single command.

**Lua scripts are the backbone of the flash sale system.** Every critical
operation in later modules (counters, idempotency, reconciliation) is either
a Lua script or builds on top of one.

## Concept Map

```
                    HTTP Request
                         |
                         v
              +---------------------+
              |  Idempotency Check  |  (p07)
              +---------------------+
                    |         |
              new   |         | duplicate
                    v         v
         +------------------+  cached result
         | Account Claim    |  (p03)
         +------------------+
                    |
              new   |
                    v
         +------------------+
         | Stock Decrement  |  (p02)
         +------------------+
                    |
              ok    |
                    v
         +------------------+
         | Voucher Limit    |  (p04)
         +------------------+
                    |
              ok    |
                    v
         +------------------+
         | Generate Code    |
         | Store Idemp Key  |
         +------------------+
                    |
                    v
              PurchaseResult

    All of the above executes inside ONE Lua script (p05).
```

The exercises build up to this combined script incrementally:

| Exercise | Primitive | Used In |
|----------|-----------|---------|
| p01 | EVAL basics | Foundation |
| p02 | Stock decrement | p05 Step 3 |
| p03 | Claim check | p05 Step 2 |
| p04 | Voucher limit | p05 Step 4 |
| **p05** | **Combined purchase** | **The master script** |
| p06 | Rate limiting | Traffic shaping (module 04) |
| p07 | Idempotency | p05 Step 1, module 05 |
| p08 | Batch operations | Product listing pages |
| p09 | Script caching | Production optimization |
| p10 | Error patterns | Production hardening |

## Theory

### Lua in Redis

Redis executes Lua scripts on its **single main thread**. While a script
runs, no other command can execute. This gives scripts **serializable
isolation** -- the strongest guarantee Redis can offer.

Key properties:

- **Atomicity**: A script either completes entirely or fails entirely.
  There is no partial execution visible to other clients.
- **Isolation**: No other Redis command can interleave with a running script.
- **Determinism**: Scripts must only use Redis commands (no I/O, no
  randomness, no system calls). Non-deterministic scripts break replication.
- **No blocking**: Scripts must execute quickly. Redis blocks ALL clients
  while a script runs. The default timeout is 5 seconds (`lua-time-limit`).

### EVAL vs EVALSHA

```
EVAL "return 42" 0
    Sends the full script text (every time).
    Simple but wastes bandwidth for frequently executed scripts.

SCRIPT LOAD "return 42"
    Loads the script into Redis's script cache.
    Returns the SHA1 hex digest: "af03f72c3962b37e..." (40 bytes).

EVALSHA af03f72c3962b37e... 0
    Sends only the 40-byte SHA1. Redis looks up the script by hash.
    ~10-100x less bandwidth per call.
```

The `redis::Script` type in the Rust `redis` crate handles this
automatically: it tries EVALSHA first, and falls back to EVAL if Redis
responds with `NOSCRIPT`. Exercise 09 teaches you to implement this
manually so you understand the mechanism.

### Script Replication

In Redis replication (master-replica), scripts are replicated as EVALSHA
commands. The replicas must have the script cached, or replication breaks.
Redis ensures this by requiring that any script sent via EVALSHA was
previously loaded via SCRIPT LOAD on the master.

### Keys vs Arguments

Redis 7.0+ requires that scripts declare their key access patterns via
the `KEYS` array. This enables Redis Cluster to route scripts to the
correct shard. Always pass keys through `KEYS`, never hardcode them in
the script body.

```
EVAL "return redis.call('GET', KEYS[1])" 1 mykey
                                   ^^^    ^  ^^^^
                                  keys   n  key values
```

## Trade-offs

### Lua Scripts vs WATCH/MULTI/EXEC

| Dimension | Lua Script | WATCH/MULTI/EXEC |
|-----------|-----------|------------------|
| Atomicity | Full (all-or-nothing) | Optimistic (retry on conflict) |
| Latency | 1 round trip | 2+ round trips (WATCH, then EXEC) |
| Retry cost | None (atomic) | Re-execute on CAS failure |
| Complexity | Lua code in Rust | Multiple command sequences |
| Cluster | KEYS-based routing | WATCH keys must be on same shard |
| Debugging | Harder (Lua in strings) | Easier (standard Redis commands) |

**Use Lua** when you need guaranteed atomicity without retries.
**Use WATCH/MULTI** when the check-and-set window is simple and conflicts
are rare.

### Lua Scripts vs Application-Level Locking

| Dimension | Lua Script | App-Level Lock (Redlock) |
|-----------|-----------|--------------------------|
| Correctness | Guaranteed by Redis single thread | Depends on clock sync, TTL tuning |
| Latency | 1 round trip | 2+ round trips (acquire + release) |
| Failure mode | Script error (fast, clear) | Stale lock (slow, subtle) |
| Fairness | FIFO by request order | No fairness guarantee |

**Lua scripts are almost always the better choice** for flash sale
operations. Application-level locks add complexity without improving
correctness.

### When NOT to Use Lua

- Operations that take more than a few milliseconds (blocks all clients)
- Operations that require external I/O (network, filesystem)
- Operations that access keys across multiple Redis Cluster shards
- Simple single-key operations (just use the native command)

## Failure Modes

### Script Errors

A Lua script that calls `redis.call()` on a wrong-type key raises a
runtime error. This aborts the script and returns an error to the client.
Use `redis.pcall()` to catch errors within the script, or use the
structured error pattern from exercise 10.

### Memory Pressure

Every loaded script lives in Redis memory forever (until SCRIPT FLUSH).
If you load thousands of unique scripts, memory grows without bound.
Mitigation: reuse scripts (the `redis::Script` type caches SHA1 digests).

### Long-Running Scripts

Redis blocks ALL clients while a script runs. The `lua-time-limit`
configuration (default 5000ms) is a hard timeout. If a script exceeds it,
Redis starts accepting `SCRIPT KILL` commands but cannot interrupt the
script until it issues its next Redis command.

**Rule of thumb**: A Lua script should complete in under 1ms. Avoid loops
over large collections.

### Non-Deterministic Scripts

Scripts that use `math.random()`, `os.time()`, or any non-Redis I/O will
produce different results on replicas, breaking replication. Redis 7.0+
warns about this with `redis.replicate_commands()`.

## Connection to Other Modules

| Module | Relationship |
|--------|-------------|
| 01 - Redis Fundamentals | Prerequisite: connection pooling, data types |
| 03 - Atomic Counters | Builds on p02 (decrement) for distributed counters |
| 04 - Traffic Shaping | Uses p06 (rate limiting) as the core primitive |
| 05 - Idempotency | Uses p07 (idempotency check) for exactly-once semantics |
| 07 - Resilience | Handles Lua script errors in circuit breakers |
| 11 - Flash Sale API | Calls p05 (combined purchase) for every buy request |
| 14 - Integration Tests | Validates the Lua scripts under realistic load |

**This module is the atomic core of the entire system.** Every subsequent
module either uses or is tested against the Lua scripts defined here.

## Exercise List

| # | Exercise | Difficulty | Key Concept |
|---|----------|-----------|-------------|
| 01 | EVAL Basics | Easy | EVAL, KEYS, ARGV, return types |
| 02 | Stock Decrement | Medium | Atomic check-and-decrement |
| 03 | Account Claim Check | Medium | Atomic SISMEMBER + SADD |
| 04 | Voucher Limit | Medium | Atomic count with ceiling |
| **05** | **Combined Purchase** | **Hard** | **Full atomic purchase flow** |
| 06 | Sliding Window Rate Limit | Medium | Sorted set time window |
| 07 | Idempotency Check | Medium | Atomic check-and-set |
| 08 | Batch Operations | Easy | Multi-key batch read |
| 09 | Script Caching | Medium | EVALSHA vs EVAL |
| 10 | Error Patterns | Medium | pcall, structured errors |

Exercises 02, 03, 04, and 05 are marked **CRITICAL** because they build
the primitives used by every flash sale request. Exercise 05 is the most
important exercise in the entire flash-sale-mastery project.

## Running the Exercises

```bash
# Run all tests (stubs will panic with todo!())
cd flash-sale-mastery/02-redis-lua-scripting
cargo test

# Run a specific exercise
cargo test --test p01_eval_basics

# Run with solutions enabled
cargo test --features solution

# Run a specific solution
cargo test --features solution p05_combined_purchase
```

**Prerequisite**: A running Redis instance at `redis://127.0.0.1:6379`.
Tests that cannot connect will print `SKIP:` and pass silently.

## References

- [Redis Lua Scripting](https://redis.io/docs/interact/programmability/eval-intro/)
- [EVAL Command](https://redis.io/commands/eval/)
- [EVALSHA Command](https://redis.io/commands/evalsha/)
- [Redis Lua API](https://redis.io/docs/interact/programmability/lua-api/)
- [redis crate Script documentation](https://docs.rs/redis/latest/redis/struct.Script.html)
- [Atomicity in Redis](https://redis.io/docs/interact/programmability/eval-intro/#atomicity-of-scripts)
