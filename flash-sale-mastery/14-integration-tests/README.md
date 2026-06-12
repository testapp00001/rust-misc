# 14 — Integration Tests

## Motivation

Unit tests verify individual components in isolation. Integration tests verify
that the components work together correctly under realistic conditions —
especially under concurrency, partial failures, and load. For a flash sale
system, the cost of getting it wrong is financial: overselling means shipping
items you don't have, and double-issuing vouchers means giving away money.

These tests prove that the system's critical invariants hold end-to-end.

## Concept Map

```
Purchase Request
    |
    v
[Idempotency Check] -- duplicate? --> return cached result
    |
    v
[Account Claim Check] -- already claimed? --> AlreadyClaimed
    |
    v
[Voucher Limit Check] -- limit reached? --> VoucherLimitReached
    |
    v
[Atomic Stock CAS] -- stock == 0? --> SoldOut
    |
    v
[Issue Voucher + Persist Order]
    |
    v
Success + Event Recorded
```

Each test exercises a specific path through this pipeline, and the concurrency
tests exercise many paths simultaneously.

## Theory

### Property-Based Testing for Concurrency

Traditional example-based tests check specific inputs and outputs. For
concurrent systems, we need to verify *properties* that hold regardless of
scheduling:

- **Invariant:** `stock >= 0` at all times (p03)
- **Invariant:** `successes <= initial_stock` for every product (p02, p10)
- **Invariant:** each account succeeds at most once per product (p06)
- **Invariant:** total vouchers <= max_vouchers per product (p07)

### Invariant Checking

Rather than checking intermediate states (which are racy), we check invariants
*after* all concurrent operations complete. The `tokio::sync::Barrier` ensures
all tasks start at roughly the same time, maximizing contention.

### Compare-and-Set (CAS)

The stock decrement uses a CAS loop:

```text
loop {
    current = stock.load()
    if current == 0: return false
    if stock.cas(current, current - 1): return true
    // else: another thread changed it, retry
}
```

This is the same pattern used by Redis Lua scripts in the real system.

## Trade-offs

| Decision | Pro | Con |
|----------|-----|-----|
| DashMap mocks vs real Redis | No external deps, fast | Doesn't test Redis protocol |
| Barrier synchronisation | Maximises contention | May not reflect real arrival patterns |
| In-memory DB mock | Deterministic, isolated | Doesn't test SQL/transaction semantics |
| Separate test files | Clear organisation | Some code duplication |

## Failure Modes

### Flaky Tests

Tests that depend on timing (e.g., "must complete in <1ms") can be flaky on
slow CI runners. We use generous timeouts for mock-based tests and note where
real Redis tests would need tighter bounds.

### Barrier Deadlock

If the barrier count doesn't match the number of spawned tasks, the test hangs
forever. Always use `tasks.len()` for the barrier count.

### DashMap Contention

Under extreme contention (p03 with 1000 tasks), DashMap's sharded locking means
some threads may spin-wait longer than others. This is expected and correct —
the CAS loop handles it.

## Connection to Other Modules

| Module | Connection |
|--------|------------|
| 01-redis-fundamentals | Tests verify the stock/decrement/claim patterns taught there |
| 02-redis-lua-scripting | The atomic CAS pattern mirrors Lua script atomicity |
| 03-atomic-counters | Stock decrement and voucher counter use atomic ops |
| 04-traffic-shaping | Rate limiting would be tested here if enabled |
| 05-idempotency | p04 directly tests the idempotency contract |
| 06-event-sourcing | Every test verifies event recording |
| 07-resilience | p08 and p09 test Redis/DB failure scenarios |
| 08-load-testing | p10 simulates realistic load |
| 09-observability | Events recorded match the tracing patterns |
| 10-caching-strategy | Cache-through patterns tested implicitly |
| 11-flash-sale-api | These tests verify the API's business logic |
| 12-order-worker | DB persistence tested in p09 |
| 13-reconciliation | p11 directly tests reconciliation |

## Test List

| File | Test | What It Verifies |
|------|------|-----------------|
| `p01_single_purchase.rs` | Single purchase | Happy path: stock decremented, voucher issued, event recorded |
| `p02_concurrent_purchases.rs` | 100 concurrent, stock=10 | Exactly 10 succeed, 90 get SoldOut, stock=0 |
| `p03_oversell_prevention.rs` | 1000 concurrent, stock=1 | **CRITICAL** — exactly 1 succeeds, stock never negative |
| `p04_duplicate_request.rs` | Same request 10 times | Idempotency: 1 voucher, 9 duplicates |
| `p05_sold_out_handling.rs` | Stock=0 | Fast-path SoldOut, no wasted work |
| `p06_per_account_limit.rs` | Same account, 5 attempts | First succeeds, rest get AlreadyClaimed |
| `p07_per_product_limit.rs` | 10 accounts, voucher cap=5 | First 5 succeed, rest get VoucherLimitReached |
| `p08_redis_failure.rs` | Redis unavailable | Returns 503, no partial state |
| `p09_db_failure.rs` | DB unavailable | Purchase succeeds (Redis-first), order queued |
| `p10_full_load_simulation.rs` | 5 products, 100 accounts, 500 requests | All invariants hold across products |
| `p11_reconciliation_test.rs` | Post-sale drift detection | Mismatches found and reported |

## Running the Tests

```bash
# All tests
cargo test -p integration-tests

# A specific test
cargo test -p integration-tests test_oversell_impossible

# With output
cargo test -p integration-tests -- --nocapture

# With solution feature (same for this module)
cargo test -p integration-tests --features solution
```
