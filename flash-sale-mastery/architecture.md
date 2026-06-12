# Flash Sale System Architecture

## System Overview

The flash sale system is designed to handle Black Friday traffic patterns:
- **Normal load**: 1,000-10,000 requests/second
- **Flash sale peak**: 100,000-500,000 requests/second
- **Duration**: 1-4 hours
- **Products**: 10-100 sale items
- **Stock per product**: 100-10,000 units
- **Voucher limit per product**: configurable (e.g., 500 per product)
- **Voucher limit per account**: 1 per product per sale

## Core Invariants

These invariants must hold under all conditions, including failures:

1. **No overselling**: Stock count must never go negative
2. **One per account**: Each account can claim at most 1 voucher per product per sale
3. **Product voucher cap**: Total vouchers issued per product cannot exceed its limit
4. **Idempotent purchases**: Duplicate requests return the same result (no extra vouchers)
5. **Audit trail**: Every stock change is recorded with timestamp, account, and reason

## Data Flow

### Hot Path (< 5ms target)

```
Client Request
    │
    ▼
[Rate Limit Check] ──→ Reject (429) if exceeded
    │
    ▼
[Idempotency Check] ──→ Return cached result if duplicate
    │
    ▼
[Redis Lua Script] ──→ Atomically:
    │   1. Check stock > 0
    │   2. Check account not already claimed
    │   3. Check product voucher limit not reached
    │   4. Decrement stock
    │   5. Add account to claimed set
    │   6. Increment product voucher counter
    │   7. Record event to Redis Stream
    │
    ▼
[Return Success/Fail]
```

### Warm Path (async, 100ms-1s)

```
Redis Stream Event
    │
    ▼
[Order Worker]
    │
    ├──→ Create order record in PostgreSQL
    ├──→ Generate voucher code
    └──→ Publish to notification queue
```

### Cold Path (async, best-effort)

```
Notification Queue
    │
    ▼
[Notification Worker]
    │
    ├──→ Send email confirmation
    └──→ Send SMS notification
```

## Redis Data Model

### Key Schema

```
# Stock counter (per product)
product:{id}:stock          → STRING (integer, DECR/INCR)

# Accounts that claimed a product (per product)
product:{id}:claims         → SET (account_ids)

# Voucher count per product
product:{id}:voucher_count  → STRING (integer, INCR)

# Idempotency keys
idempotency:{key}           → STRING (JSON result, SET NX with TTL)

# Rate limiting (per account)
rate:{account_id}           → STRING (integer, INCR with TTL)

# Sliding window rate limit
rate:window:{account_id}    → SORTED SET (timestamp → request_id)

# Order event stream
orders:stream               → STREAM (order events)

# Sale configuration
sale:config                 → HASH (start_time, end_time, status)
```

### Hash Slot Co-location

For Redis Cluster, related keys must be in the same hash slot:

```
{product:123}:stock
{product:123}:claims
{product:123}:voucher_count
```

The `{product:123}` prefix ensures all keys for the same product land on the same node.

## Lua Script: Atomic Purchase

```lua
-- KEYS[1] = product:{id}:stock
-- KEYS[2] = product:{id}:claims
-- KEYS[3] = product:{id}:voucher_count
-- KEYS[4] = idempotency:{key}
-- ARGV[1] = account_id
-- ARGV[2] = product_id
-- ARGV[3] = max_vouchers_per_product
-- ARGV[4] = idempotency_result (JSON)

-- Step 1: Idempotency check
local existing = redis.call('GET', KEYS[4])
if existing then
    return {2, existing}  -- 2 = already processed
end

-- Step 2: Check stock
local stock = tonumber(redis.call('GET', KEYS[1]) or '0')
if stock <= 0 then
    return {0, 'sold_out'}  -- 0 = no stock
end

-- Step 3: Check account claim
if redis.call('SISMEMBER', KEYS[2], ARGV[1]) == 1 then
    return {1, 'already_claimed'}  -- 1 = duplicate claim
end

-- Step 4: Check product voucher limit
local voucher_count = tonumber(redis.call('GET', KEYS[3]) or '0')
if voucher_count >= tonumber(ARGV[3]) then
    return {3, 'voucher_limit_reached'}  -- 3 = limit reached
end

-- Step 5: Execute purchase (all checks passed)
redis.call('DECR', KEYS[1])
redis.call('SADD', KEYS[2], ARGV[1])
redis.call('INCR', KEYS[3])

-- Step 6: Store idempotency result
redis.call('SETEX', KEYS[4], 3600, ARGV[4])

return {4, 'success'}  -- 4 = purchase successful
```

## Failure Modes and Handling

| Failure | Impact | Mitigation |
|---------|--------|------------|
| Redis down | Cannot process purchases | Serve "temporarily unavailable", circuit breaker opens |
| DB down | Cannot create order records | Queue events in Redis, retry later |
| Network partition | Split-brain risk | Redis Cluster with majority consensus |
| Redis failover | Brief unavailability | Sentinel or Cluster automatic failover |
| Clock skew | Idempotency key issues | Use Redis TTL, not system clock |
| Memory pressure | Redis eviction | Set maxmemory-policy to noeviction for stock keys |

## Scaling Strategy

### Pre-Sale (T-30 minutes)

1. Pre-warm Redis with product data and stock counters
2. Scale API pods to expected peak capacity
3. Warm database connection pools
4. Enable rate limiting at load balancer

### During Sale

1. Monitor stock levels in real-time
2. Auto-scale based on request queue depth
3. Circuit breaker protects downstream services
4. Graceful degradation: serve "sold out" from cache when stock is 0

### Post-Sale (T+0)

1. Stop accepting new purchases
2. Run reconciliation job: sync Redis → DB
3. Generate audit report
4. Scale down infrastructure
