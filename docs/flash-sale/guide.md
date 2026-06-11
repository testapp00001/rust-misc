Here is a comprehensive, agent-ready document. Save it as a file and feed it to your AI agent.

---

```markdown
# FLASH SALE MASTERY — Agent Specification Document

## PURPOSE

Build a complete, production-grade learning project in Rust that teaches
every concept needed to design, implement, and operate a commercial flash
sale system handling Black Friday traffic. The learner already possesses
significant Rust, DevOps, security, and algorithm knowledge. This project
must fill specific gaps and integrate existing knowledge into a cohesive
system.

The final deliverable is a working system, not theoretical explanations.
Every module must contain compilable, tested Rust code.

---

## LEARNER'S EXISTING KNOWLEDGE INVENTORY

The learner has completed 5 major learning repositories. Below is every
module relevant to the flash sale problem, with the specific skills
extracted from each.

### SOURCE 1: rust-dev-mastery (20 modules, ~200 exercises)

#### 05-async-rust
Skills acquired:
- Tokio runtime internals (multi-threaded scheduler, task spawning)
- Future trait implementation and polling mechanics
- Async error handling patterns (Result in async context)
- Concurrency primitives: tokio::join!, tokio::select!, JoinSet
- Cancellation safety (which async operations are safe to cancel)
- Stream processing (futures::Stream, async iteration)
- Async channel patterns (mpsc, broadcast, oneshot, watch)
- Task-local storage
- Async performance: avoiding allocation on hot paths

Relevance to flash sale: HIGH (9/10)
Specific application: Every request handler is async. The purchase flow
involves concurrent Redis + DB operations. Cancellation safety ensures
stock is never "lost" when a client disconnects mid-transaction.

#### 06-performance-profiling
Skills acquired:
- perf, flamegraph, cargo-flamegraph
- Criterion.rs benchmarking
- CPU cache line awareness
- Branch prediction optimization
- SIMD intrinsics (AVX2 basics)
- Memory allocation profiling (dhat, heaptrack)
- Zero-allocation patterns (pre-allocated buffers, arena allocation)
- Lazy evaluation strategies
- Profile-guided optimization (PGO)

Relevance to flash sale: HIGH (7/10)
Specific application: The hot path (stock check + decrement) must be
sub-millisecond. Profiling reveals bottlenecks under load.

#### 12-networking-web
Skills acquired:
- HTTP client construction (reqwest, hyper)
- Axum framework: routing, handlers, extractors, state
- Middleware pipeline design (tower::Service, Layer)
- REST API design principles
- Request validation (custom extractors, serde validation)
- Authentication middleware (JWT, session tokens)
- WebSocket handling
- gRPC basics (tonic)
- Rate limiting implementation
- Custom protocol design

Relevance to flash sale: HIGH (8/10)
Specific application: The API layer is built on Axum. Rate limiting
middleware protects against abuse. Request validation prevents malformed
payloads.

#### 14-database-storage
Skills acquired:
- SQLx: connection pools, compile-time query checking
- Connection pooling theory (min/max connections, idle timeout)
- Database migrations (sqlx-cli, embedded migrations)
- Transaction isolation levels (Read Committed, Repeatable Read, Serializable)
- Repository pattern
- Embedded databases (sled, rusqlite)
- Cache-aside pattern with Redis
- Query optimization (EXPLAIN, indexing strategy)
- Storage engine concepts (B-tree, LSM tree awareness)

Relevance to flash sale: HIGH (8/10)
Specific application: Order persistence, audit logging, inventory
reconciliation. Connection pools must be sized for burst traffic.
Transactions ensure order consistency.

#### 17-concurrency
Skills acquired:
- Channel patterns: mpsc, broadcast, rendezvous, bounded
- Shared state: Arc<Mutex<T>>, Arc<RwLock<T>>, Arc<AtomicXxx>
- Lock-free data structures (crossbeam, atomic operations)
- Actor model (message-passing ownership)
- Work-stealing (rayon, tokio task stealing)
- Parallel iterators
- Synchronization barriers (Barrier, Condvar)
- Concurrent HashMap (dashmap)
- Testing concurrent code (loom, shuttle)
- Architectural patterns for concurrent systems

Relevance to flash sale: CRITICAL (10/10)
Specific application: The core problem IS concurrency. Multiple instances
decrementing the same stock counter. Per-account claim tracking. The
actor model maps directly to per-product stock ownership.

#### 02-error-handling
Skills acquired:
- Error hierarchy design (thiserror, anyhow)
- Error context chaining
- Client-facing vs internal errors
- Error recovery strategies
- Panic handling (catch_unwind, set_hook)
- Error testing patterns

Relevance to flash sale: MEDIUM (6/10)
Specific application: Graceful error responses. Never expose internal
errors to clients during flash sale. Recovery from Redis connection
failures.

#### 03-logging-tracing
Skills acquired:
- tracing crate: spans, events, structured fields
- Subscriber configuration (fmt, env-filter)
- Error tracing integration
- Distributed tracing (OpenTelemetry concepts)
- Metrics integration (prometheus)
- Production observability patterns

Relevance to flash sale: HIGH (7/10)
Specific application: Every purchase attempt must be traceable. Distributed
tracing across gateway → stock service → Redis → DB. Structured logging
for post-sale analysis.

#### 04-testing
Skills acquired:
- Unit test patterns (assertions, fixtures)
- Integration tests (test containers, HTTP testing)
- Property-based testing (proptest)
- Fuzzing (cargo-fuzz, libfuzzer)
- Mocking (mockall)
- Benchmarking (criterion)
- Snapshot testing
- Test coverage (cargo-tarpaulin)

Relevance to flash sale: HIGH (7/10)
Specific application: Concurrency tests (can 1000 threads all try to buy
the last item?). Property tests (stock never goes negative). Load tests.

### SOURCE 2: rust-security-lab (31 modules, ~310 exercises)

#### 14-api-security
Skills acquired:
- Rate limiting: token bucket, sliding window, fixed window
- HMAC request signing
- CORS configuration
- CSRF protection
- API key management
- Input size limits
- JSON bomb prevention

Relevance to flash sale: HIGH (8/10)
Specific application: Rate limiting per account prevents bot abuse.
Input validation rejects malformed purchase requests.

#### 17-input-validation
Skills acquired:
- Type-driven validation (newtype pattern for IDs)
- SQL injection prevention (parameterized queries)
- Command injection prevention
- Path traversal prevention
- Integer overflow detection
- Input sanitization

Relevance to flash sale: MEDIUM (5/10)
Specific application: Validate product_id, account_id, quantities.
Prevent injection through any user-controlled field.

#### 18-logging-and-audit
Skills acquired:
- PII redaction in logs
- Structured logging for security
- Log injection prevention
- Tamper-evident logging
- Audit trail design
- Correlation IDs

Relevance to flash sale: HIGH (7/10)
Specific application: Audit every stock change. Correlation IDs track
a request through the entire system. PII redaction for GDPR compliance.

### SOURCE 3: devops-grandmaster (81 modules)

#### 19-load-balancing
Skills acquired:
- Nginx load balancer configuration
- Algorithms: round-robin, least-connections, IP hash
- Health checks
- Session persistence

Relevance to flash sale: MEDIUM (6/10)
Specific application: Distribute flash sale traffic across app instances.

#### 20-reverse-proxy
Skills acquired:
- Nginx reverse proxy configuration
- SSL termination
- Request buffering
- Connection limiting

Relevance to flash sale: MEDIUM (5/10)
Specific application: Front gate for all traffic. SSL offload.

#### 67-auto-scaling
Skills acquired:
- HPA (Horizontal Pod Autoscaler)
- CPU/memory-based scaling
- Custom metrics scaling
- Scale-up/down policies

Relevance to flash sale: MEDIUM (6/10)
Specific application: Scale app pods during sale. Pre-scale before
sale starts.

#### 70-caching-strategies
Skills acquired:
- Cache-aside, write-through, write-behind
- Cache invalidation strategies
- TTL management
- Cache warming

Relevance to flash sale: HIGH (7/10)
Specific application: Product data caching. Pre-warm stock counters
in Redis before sale starts.

#### 71-performance-tuning
Skills acquired:
- Linux kernel parameter tuning
- File descriptor limits
- TCP tuning (backlog, keepalive)
- Memory overcommit settings

Relevance to flash sale: MEDIUM (5/10)
Specific application: OS-level tuning for high connection count.

#### 74-distributed-systems-patterns
Skills acquired:
- Circuit breaker
- Bulkhead pattern
- Retry with exponential backoff
- Timeout patterns
- Saga pattern

Relevance to flash sale: HIGH (7/10)
Specific application: Circuit breaker on Redis calls. Retry for transient
failures. Timeout for slow DB writes.

### SOURCE 4: rust-DSA (18 categories, ~120 problems)

#### Relevant data structures:
- HashMap / HashSet (product lookups, deduplication)
- BTreeMap (ordered iteration, range queries)
- Binary Heap / Priority Queue (priority-based request ordering)
- Ring Buffer / Circular Queue (sliding window counters)

#### Relevant algorithms:
- Two Sum pattern (matching requests to inventory)
- Sliding Window (rate limiting window calculation)

Relevance to flash sale: LOW-MEDIUM (3/10)
Specific application: The algorithmic thinking helps more than specific
data structures. Most DSA problems don't directly apply because the
flash sale problem is about distributed coordination, not algorithmic
complexity.

### SOURCE 5: rust-interview-guide (10 modules)

#### 05-mid-level-systems
Skills acquired:
- File I/O, serialization, CLI tools
- Networking fundamentals
- Logging, configuration management
- Database basics
- Testing, performance measurement

#### 08-senior-performance
Skills acquired:
- Profiling methodology
- Memory optimization
- CPU optimization
- SIMD, zero-cost abstractions
- Caching, lazy evaluation
- Parallel optimization

#### 09-lead-architecture
Skills acquired:
- System design methodology
- API design principles
- Library design
- Scalability patterns
- Reliability patterns
- Trade-off analysis

Relevance to flash sale: MEDIUM (5/10)
Specific application: Architectural thinking and trade-off analysis.
The interview guide teaches HOW to think about these problems.

---

## KNOWLEDGE GAPS — Detailed Specification

These are the specific topics NOT covered by any existing repository.
Each gap includes: topic, why it matters, what to learn, expected
proficiency, and how it integrates with existing knowledge.

### GAP 1: Redis as an Atomic Coordination Layer

Why critical:
The flash sale hot path cannot rely on database transactions. A Postgres
write takes 1-5ms. At 500K concurrent requests, that's 500-2500 seconds
of sequential writes. Redis operations take 0.1-0.5ms and support
atomic multi-key operations via Lua scripting.

What to learn:
1. Redis data types and their use in this problem:
   - STRING with DECR/INCR (stock counter)
   - SET with SADD/SISMEMBER (per-account claim tracking)
   - HASH with HINCRBY (per-product voucher counters)
   - SORTED SET with ZADD/ZRANGEBYSCORE (sliding window rate limiting)
   - STREAM with XADD/XREADGROUP (order event queue)

2. Redis Lua scripting:
   - EVAL and EVALSHA commands
   - Atomic multi-step operations (check-then-act as single operation)
   - KEYS and ARGV parameter passing
   - Return value types (integer, string, table, nil)
   - Error handling within Lua scripts
   - Script caching and SHA1 hashing

3. Redis transactions:
   - WATCH/MULTI/EXEC (optimistic locking)
   - Why Lua scripting is preferred over WATCH/MULTI for flash sales
   - Transaction abort behavior

4. Redis Cluster:
   - Hash slot distribution
   - Cross-slot Lua limitations (all keys must be in same slot)
   - Hash tags for key co-location: {product:123}:stock, {product:123}:claims
   - Cluster failover and consistency implications

5. Redis persistence:
   - RDB snapshots vs AOF (Append-Only File)
   - AOF fsync policies (always, everysec, no)
   - Data loss window under each policy
   - Why flash sale data can tolerate some Redis data loss (DB is source of truth)

6. redis-rs crate:
   - Connection pooling (deadpool-redis or r2d2)
   - Connection manager for async
   - Pipeline and transaction support
   - Custom command execution (EVAL)
   - Cluster client (redis-cluster-async)

Expected proficiency:
Write production-quality Redis Lua scripts that atomically check stock,
check per-account limits, check per-product limits, and execute
decrement — all in a single atomic operation with no race conditions.

Integration with existing knowledge:
- rust-dev-mastery/05-async-rust → async Redis client usage
- rust-dev-mastery/14-database-storage → cache-aside pattern extended
- rust-dev-mastery/17-concurrency → atomic operations replace mutex

### GAP 2: Distributed Atomic Counters and Compare-and-Swap

Why critical:
With multiple application instances (pods), in-memory counters are
useless. Stock must be decremented atomically across all instances.

What to learn:
1. Atomic operations in distributed context:
   - Compare-and-Swap (CAS) protocol
   - Optimistic locking with version numbers
   - Pessimistic locking with distributed locks
   - When to use each approach

2. Redis-based atomic patterns:
   - DECR with guard (check > 0 before decrement, atomically)
   - INCR with ceiling (check < max before increment, atomically)
   - Lua script as atomic transaction boundary
   - SETNX-based distributed lock (basic, not Redlock)

3. Database-based atomic patterns (fallback/reconciliation):
   - UPDATE ... SET stock = stock - 1 WHERE stock > 0
   - Optimistic locking: WHERE version = expected_version
   - SELECT FOR UPDATE (pessimistic, last resort)
   - Advisory locks in PostgreSQL

4. Reconciliation pattern:
   - Redis as fast path (authoritative during sale)
   - Background sync to database (eventual consistency)
   - Conflict resolution when Redis and DB disagree
   - Post-sale reconciliation job

5. Oversell prevention guarantees:
   - Theoretical analysis: under what conditions can overselling occur?
   - Redis single-threaded execution model (why Lua is safe)
   - Network partition scenarios
   - Redis Cluster failover during operation

Expected proficiency:
Design and implement a multi-layer atomic counter system where Redis
handles the hot path with Lua scripts and the database provides
durability with eventual reconciliation. Prove that overselling is
impossible under normal operation and characterize failure modes.

Integration with existing knowledge:
- rust-dev-mastery/17-concurrency → lock-free patterns extended to distributed
- rust-dev-mastery/14-database-storage → transaction patterns for reconciliation
- devops-grandmaster/74-distributed-systems → consistency models

### GAP 3: Request Queuing and Traffic Shaping

Why critical:
5 million concurrent users cannot all hit the stock service simultaneously.
The system must admit, queue, shape, and reject traffic intelligently.

What to learn:
1. Admission control:
   - Reject vs queue decision logic
   - "Sold out" fast response (don't queue if stock is 0)
   - Connection limiting at reverse proxy level
   - Max in-flight requests per instance

2. Virtual waiting room:
   - Concept: hold excess users in a queue, release in batches
   - Implementation: HTTP 202 + polling, or Server-Sent Events
   - Position tracking and estimated wait time
   - Fairness guarantees (FIFO)

3. Backpressure propagation:
   - Tokio bounded channels (try_send vs send vs send with timeout)
   - Tower load shed middleware
   - How backpressure propagates: DB → service → gateway → client
   - Observable queue depth metrics

4. Traffic shaping algorithms:
   - Token bucket: allows bursts, smooths long-term rate
   - Sliding window log: precise counting, memory-expensive
   - Sliding window counter: approximate, memory-efficient
   - Leaky bucket: constant output rate
   - Implementation of each in Rust

5. Graceful degradation:
   - Serve "sold out" in <10ms when stock is depleted (cache the state)
   - Return 503 with Retry-After when queue is full
   - Serve stale data vs error
   - Feature flags to disable features under extreme load

6. Rate limiting at multiple layers:
   - Layer 1: Nginx limit_req (binary rate limiting)
   - Layer 2: Application rate limit (per account, per IP)
   - Layer 3: Redis distributed rate limit (cross-instance)
   - Layer 4: Upstream service rate limit (protect DB)

Expected proficiency:
Implement a multi-layer admission control system that keeps the service
responsive under 10x expected load. Design a virtual waiting room that
provides fairness guarantees.

Integration with existing knowledge:
- rust-security-lab/14-api-security → rate limiting patterns extended
- rust-dev-mastery/12-networking-web → Axum middleware pipeline
- rust-dev-mastery/05-async-rust → bounded channels for backpressure
- devops-grandmaster/19-load-balancing → Nginx-level shaping

### GAP 4: Idempotency and Exactly-Once Semantics

Why critical:
Users will click "Buy" multiple times. Browsers will retry HTTP requests.
Network timeouts will cause duplicate submissions. Without idempotency,
one user could receive multiple vouchers.

What to learn:
1. Idempotency keys:
   - Client generates unique key per intended purchase
   - Server stores key → result mapping
   - Duplicate requests return cached result
   - Key expiration policy

2. Implementation approaches:
   - Redis SET NX with TTL (fast path)
   - Database unique constraint (durable path)
   - Two-phase: Redis check → DB constraint as safety net

3. Exactly-once delivery:
   - Why true exactly-once is impossible in distributed systems
   - Effectively-once: idempotent consumer + deduplication
   - Message deduplication in event queues
   - Deduplication window sizing

4. Client-side considerations:
   - Idempotency key generation (UUID v4 or deterministic hash)
   - Storing key in browser (retry after page refresh)
   - Mobile app considerations (background retry)

5. Server-side implementation:
   - Middleware that checks idempotency before handler
   - Storage: Redis (fast, volatile) + DB (slow, durable)
   - Race condition: two identical requests arrive simultaneously
   - Resolution: Redis SET NX atomic check

Expected proficiency:
Implement idempotent purchase handlers that correctly deduplicate
concurrent duplicate requests with zero false positives (never give
extra vouchers) and zero false negatives (never reject legitimate
first attempts).

Integration with existing knowledge:
- rust-dev-mastery/12-networking-web → middleware pattern
- rust-security-lab/14-api-security → request validation
- rust-dev-mastery/05-async-rust → concurrent dedup race conditions

### GAP 5: Event Sourcing and Audit Trail (Lightweight)

Why critical:
Every stock change must be traceable. If there's a dispute ("I didn't
receive my voucher"), the system must reconstruct the exact sequence
of events.

What to learn:
1. Event sourcing basics:
   - Store events, not state (append-only log)
   - Rebuild state by replaying events
   - Event: StockDecremented { product_id, account_id, timestamp, remaining }
   - Event: VoucherClaimed { product_id, account_id, voucher_code, timestamp }

2. Lightweight implementation (not full CQRS):
   - Append events to Redis Stream (fast, in-memory)
   - Background worker persists events to database (durable)
   - Event schema design (versioned, self-describing)
   - Correlation IDs linking related events

3. Event store design:
   - PostgreSQL table: events(id, type, aggregate_id, payload, timestamp)
   - Partitioning by time for query performance
   - Indexing strategy for audit queries

4. Replay and reconciliation:
   - Rebuild inventory state from events
   - Detect discrepancies between event log and current state
   - Automatic reconciliation job

5. Event-driven architecture:
   - Redis Streams as event bus
   - Consumer groups for parallel processing
   - At-least-once delivery with idempotent consumers
   - Dead letter queue for failed events

Expected proficiency:
Design an event-sourced audit system that records every inventory
change, supports replay for reconciliation, and provides a complete
trail for dispute resolution.

Integration with existing knowledge:
- rust-dev-mastery/03-logging-tracing → structured event logging
- rust-security-lab/18-logging-and-audit → audit trail patterns
- rust-dev-mastery/05-async-rust → stream processing

### GAP 6: Circuit Breaker and Resilience Patterns

Why critical:
During Black Friday, downstream services (DB, payment, notification)
will experience failures. The system must degrade gracefully, not
cascade into total failure.

What to learn:
1. Circuit breaker pattern:
   - States: Closed → Open → Half-Open
   - Failure threshold and recovery timeout
   - Per-endpoint circuit breakers
   - Implementation in Rust (tower middleware or manual)

2. Bulkhead pattern:
   - Isolate resources per operation type
   - Separate thread pools / Tokio tasks for stock check vs order creation
   - Semaphore-based concurrency limiting
   - Prevent one slow operation from blocking others

3. Retry strategies:
   - Exponential backoff with jitter
   - Retry budget (max retries per time window)
   - Retryable vs non-retryable errors
   - Idempotency requirement for retries

4. Timeout management:
   - Per-operation timeouts (Redis: 50ms, DB: 500ms, Payment: 5s)
   - Cascading timeout calculation
   - Tokio timeout and timeout_at

5. Fallback strategies:
   - Stock check fails → serve "temporarily unavailable" (not "sold out")
   - DB write fails → queue for retry (eventually consistent)
   - Notification fails → best-effort, don't block purchase

Expected proficiency:
Implement a resilience layer that keeps the flash sale operational
when individual components fail. Characterize the system's behavior
under each failure mode.

Integration with existing knowledge:
- devops-grandmaster/74-distributed-systems → resilience patterns
- rust-dev-mastery/02-error-handling → error recovery strategies
- rust-dev-mastery/05-async-rust → tokio::time::timeout

### GAP 7: Load Testing and Capacity Planning

Why critical:
You cannot ship a flash sale system without knowing its breaking point.
Load testing reveals bottlenecks before real users do.

What to learn:
1. Load testing tools and methodology:
   - k6 (JavaScript-based, HTTP load testing)
   - wrk / wrk2 (C-based, high-throughput HTTP)
   - Drill (Rust-based load testing)
   - Custom Rust load generator (tokio + reqwest)

2. Test scenarios for flash sale:
   - Ramp-up: 0 → 100K concurrent over 60 seconds
   - Spike: instant jump to 500K concurrent
   - Sustained: hold 100K concurrent for 10 minutes
   - Soak: moderate load for hours (memory leak detection)

3. Metrics to collect:
   - Throughput: requests/second
   - Latency: p50, p95, p99, p99.9
   - Error rate: 4xx, 5xx, timeouts
   - Redis latency: per-operation timing
   - DB connection pool utilization
   - Memory usage over time
   - CPU utilization per core

4. Capacity planning:
   - Little's Law: L = λW (concurrency = arrival_rate × latency)
   - Queueing theory: utilization vs latency curve
   - Headroom planning: target 70% utilization at peak
   - Pre-sale scaling strategy

5. Bottleneck identification:
   - CPU-bound vs I/O-bound detection
   - Redis vs DB vs network as bottleneck
   - Connection pool exhaustion
   - Tokio runtime saturation
   - Memory allocation pressure

Expected proficiency:
Design and execute load tests that validate the system handles
target traffic. Identify and resolve bottlenecks. Provide capacity
planning recommendations with mathematical backing.

Integration with existing knowledge:
- rust-dev-mastery/06-performance-profiling → profiling under load
- rust-dev-mastery/04-testing → benchmarking methodology
- devops-grandmaster/67-auto-scaling → scaling based on load test data

### GAP 8: Message Queue Patterns

Why critical:
The purchase flow must be decoupled. The hot path (check stock + reserve)
must complete in milliseconds. Order creation, payment, notification
happen asynchronously.

What to learn:
1. Async processing pipeline:
   - Hot path: Redis atomic check + decrement (synchronous, fast)
   - Warm path: Order record creation (async, queued)
   - Cold path: Payment processing, email/SMS notification (async, best-effort)

2. Redis Streams as message queue:
   - XADD: append event to stream
   - XREADGROUP: consumer group reading
   - XACK: acknowledge processing
   - Pending entry list (PEL) for crash recovery
   - Consumer group scaling

3. Alternative: Tokio channels as in-process queue:
   - Bounded mpsc channel between handler and worker
   - Overflow handling (reject vs block vs drop oldest)
   - Multiple workers consuming from same channel
   - When to use channels vs Redis Streams

4. Message ordering and deduplication:
   - Per-product ordering guarantees
   - Message ID generation
   - Idempotent message processing
   - Dead letter handling

5. At-least-once vs at-most-once vs effectively-once:
   - Trade-offs for each delivery guarantee
   - Flash sale requirement: at-least-once + idempotent consumer
   - Implementation of idempotent consumer

Expected proficiency:
Design a multi-stage async processing pipeline where the hot path
completes in <5ms and downstream processing is reliable with
at-least-once delivery guarantees.

Integration with existing knowledge:
- rust-dev-mastery/17-concurrency → channel patterns
- rust-dev-mastery/05-async-rust → stream processing
- rust-dev-mastery/14-database-storage → transactional outbox pattern

### GAP 9: Distributed Tracing and Observability Under Load

Why critical:
When 500K requests hit simultaneously and something goes wrong, you
need to trace individual requests through the system and aggregate
metrics to identify patterns.

What to learn:
1. Distributed tracing:
   - Trace context propagation (W3C Trace Context header)
   - Span creation for each operation (Redis call, DB query, queue publish)
   - Sampling strategies (probabilistic, rate-limiting, tail-based)
   - Jaeger or Zipkin deployment
   - OpenTelemetry SDK in Rust (opentelemetry crate)

2. Flash-sale-specific metrics:
   - Stock level gauge (real-time remaining inventory)
   - Purchase attempt counter (success, sold_out, already_claimed, error)
   - Request latency histogram (p50, p95, p99)
   - Queue depth gauge (pending orders)
   - Circuit breaker state gauge
   - Redis operation latency histogram

3. Alerting rules:
   - Stock drops below threshold → alert operations
   - Error rate exceeds 1% → alert engineering
   - p99 latency exceeds 500ms → alert engineering
   - Redis connection failures → alert infrastructure
   - Oversell detected (stock < 0) → CRITICAL alert

4. Grafana dashboard design:
   - Real-time stock level panel
   - Requests/second with success/failure breakdown
   - Latency heatmap
   - Redis performance panel
   - System resource panel (CPU, memory, connections)

Expected proficiency:
Instrument the flash sale system with distributed tracing, custom
metrics, and real-time dashboards. Set up alerts that catch problems
before users report them.

Integration with existing knowledge:
- rust-dev-mastery/03-logging-tracing → tracing crate usage
- devops-grandmaster/39-metrics-collection → Prometheus setup
- devops-grandmaster/42-alerting-systems → Alertmanager rules
- devops-grandmaster/44-dashboard-design → Grafana dashboards

### GAP 10: Cache Invalidation and Pre-Warming Strategy

Why critical:
Product data, stock counts, and sale configuration must be cached
effectively. A cache miss during the flash sale creates a thundering
herd on the database.

What to learn:
1. Cache warming:
   - Pre-load product data into Redis before sale starts
   - Pre-initialize stock counters with correct values
   - Pre-compute voucher limits
   - Warm-up script that runs before sale start time

2. Cache-aside for flash sale:
   - Product details: cache with TTL (rarely changes during sale)
   - Stock count: in Redis (primary, not cache)
   - Account claims: in Redis SET (primary, not cache)
   - Sale configuration: cache with short TTL

3. Thundering herd prevention:
   - Singleflight pattern: only one request fetches from DB
   - Mutex-based cache stampede protection
   - Probabilistic early expiration (add jitter to TTL)

4. Cache as primary vs cache as cache:
   - During flash sale: Redis is PRIMARY for stock/claims
   - After flash sale: database becomes source of truth
   - Transition strategy: drain Redis → sync to DB → switch mode

5. Invalidation strategy:
   - Stock changes: update Redis directly (not invalidation)
   - Product changes: invalidate cache key, next request re-fetches
   - Sale end: batch sync all Redis state to DB

Expected proficiency:
Design a caching architecture that eliminates database hits on the
hot path while maintaining data consistency. Implement cache warming
and transition strategies.

Integration with existing knowledge:
- devops-grandmaster/70-caching-strategies → general caching theory
- rust-dev-mastery/14-database-storage → cache-aside pattern

---

## PROJECT STRUCTURE

The learning project must be structured as follows. Each module is a
Rust crate within a Cargo workspace. Each module contains:
- README.md (concept explanation, diagrams, trade-off analysis)
- src/lib.rs (module root)
- src/pXX_topic_name.rs (individual exercises)
- tests/ (integration tests)
- benches/ (where applicable)

### Workspace Structure

flash-sale-mastery/
├── Cargo.toml                    (workspace root)
├── README.md                     (project overview, architecture diagram)
├── architecture.md               (complete system architecture document)
├── 01-redis-fundamentals/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_connection_pool.rs
│       ├── p02_string_operations.rs
│       ├── p03_hash_operations.rs
│       ├── p04_set_operations.rs
│       ├── p05_sorted_set_operations.rs
│       ├── p06_pipeline_operations.rs
│       ├── p07_transaction_basics.rs
│       ├── p08_key_expiration.rs
│       ├── p09_error_handling.rs
│       └── p10_connection_resilience.rs
│
├── 02-redis-lua-scripting/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_eval_basics.rs
│       ├── p02_stock_decrement.rs         (atomic check + decrement)
│       ├── p03_account_claim_check.rs     (SISMEMBER + SADD atomic)
│       ├── p04_product_voucher_limit.rs   (HINCRBY with ceiling)
│       ├── p05_combined_purchase.lua      (THE master Lua script)
│       ├── p06_sliding_window_rate_limit.rs
│       ├── p07_idempotency_check.rs
│       ├── p08_batch_operations.rs
│       ├── p09_script_caching.rs
│       └── p10_lua_error_patterns.rs
│
├── 03-atomic-counters/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_local_atomic_counter.rs
│       ├── p02_redis_atomic_counter.rs
│       ├── p03_cas_pattern.rs
│       ├── p04_optimistic_locking.rs
│       ├── p05_pessimistic_locking.rs
│       ├── p06_reconciliation.rs
│       ├── p07_counter_benchmark.rs
│       ├── p08_failure_analysis.rs
│       └── p09_multi_instance_test.rs
│
├── 04-traffic-shaping/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_token_bucket.rs
│       ├── p02_sliding_window_log.rs
│       ├── p03_sliding_window_counter.rs
│       ├── p04_leaky_bucket.rs
│       ├── p05_admission_control.rs
│       ├── p06_virtual_waiting_room.rs
│       ├── p07_backpressure_channel.rs
│       ├── p08_graceful_degradation.rs
│       ├── p09_multi_layer_rate_limit.rs
│       └── p10_rate_limit_middleware.rs
│
├── 05-idempotency/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_idempotency_key_generation.rs
│       ├── p02_redis_idempotency_store.rs
│       ├── p03_db_idempotency_store.rs
│       ├── p04_concurrent_dedup.rs
│       ├── p05_idempotent_handler_middleware.rs
│       ├── p06_exactly_once_consumer.rs
│       ├── p07_race_condition_test.rs
│       └── p08_idempotency_benchmark.rs
│
├── 06-event-sourcing/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_event_design.rs
│       ├── p02_event_store.rs
│       ├── p03_event_replay.rs
│       ├── p04_redis_streams.rs
│       ├── p05_consumer_groups.rs
│       ├── p06_reconciliation_job.rs
│       ├── p07_audit_query.rs
│       └── p08_event_versioning.rs
│
├── 07-resilience/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_circuit_breaker.rs
│       ├── p02_bulkhead.rs
│       ├── p03_retry_with_backoff.rs
│       ├── p04_timeout_management.rs
│       ├── p05_fallback_strategies.rs
│       ├── p06_graceful_shutdown.rs
│       ├── p07_health_check.rs
│       └── p08_failure_injection.rs
│
├── 08-load-testing/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_load_generator.rs
│       ├── p02_ramp_up_scenario.rs
│       ├── p03_spike_scenario.rs
│       ├── p04_sustained_load.rs
│       ├── p05_metrics_collector.rs
│       ├── p06_latency_analysis.rs
│       ├── p07_capacity_model.rs
│       └── p08_bottleneck_identifier.rs
│
├── 09-observability/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_tracing_setup.rs
│       ├── p02_span_propagation.rs
│       ├── p03_custom_metrics.rs
│       ├── p04_redis_metrics.rs
│       ├── p05_business_metrics.rs
│       ├── p06_alert_rules.rs
│       ├── p07_dashboard_spec.rs
│       └── p08_log_analysis.rs
│
├── 10-caching-strategy/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── p01_cache_warming.rs
│       ├── p02_thundering_herd.rs
│       ├── p03_singleflight.rs
│       ├── p04_cache_aside.rs
│       ├── p05_primary_vs_cache.rs
│       ├── p06_invalidation.rs
│       ├── p07_transition_strategy.rs
│       └── p08_cache_benchmark.rs
│
├── 11-flash-sale-api/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── main.rs
│       ├── config.rs
│       ├── routes/
│       │   ├── mod.rs
│       │   ├── purchase.rs
│       │   ├── stock_query.rs
│       │   └── health.rs
│       ├── middleware/
│       │   ├── mod.rs
│       │   ├── rate_limit.rs
│       │   ├── idempotency.rs
│       │   ├── tracing.rs
│       │   └── error_handler.rs
│       ├── services/
│       │   ├── mod.rs
│       │   ├── stock_service.rs
│       │   ├── voucher_service.rs
│       │   ├── order_service.rs
│       │   └── notification_service.rs
│       ├── models/
│       │   ├── mod.rs
│       │   ├── purchase.rs
│       │   ├── stock.rs
│       │   ├── voucher.rs
│       │   └── events.rs
│       └── resilience/
│           ├── mod.rs
│           ├── circuit_breaker.rs
│           └── fallback.rs
│
├── 12-order-worker/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── main.rs
│       ├── consumer.rs
│       ├── order_processor.rs
│       ├── payment_handler.rs
│       ├── notification_handler.rs
│       └── dead_letter.rs
│
├── 13-reconciliation/
│   ├── Cargo.toml
│   ├── README.md
│   └── src/
│       ├── lib.rs
│       ├── main.rs
│       ├── inventory_sync.rs
│       ├── event_replay.rs
│       ├── discrepancy_detector.rs
│       └── report_generator.rs
│
├── 14-integration-tests/
│   ├── Cargo.toml
│   ├── README.md
│   └── tests/
│       ├── p01_single_purchase.rs
│       ├── p02_concurrent_purchases.rs
│       ├── p03_oversell_prevention.rs
│       ├── p04_duplicate_request.rs
│       ├── p05_sold_out_handling.rs
│       ├── p06_per_account_limit.rs
│       ├── p07_per_product_limit.rs
│       ├── p08_redis_failure.rs
│       ├── p09_db_failure.rs
│       ├── p10_full_load_simulation.rs
│       └── p11_reconciliation_test.rs
│
└── 15-capstone-deployment/
    ├── Cargo.toml
    ├── README.md
    ├── docker-compose.yml
    ├── Dockerfile
    ├── k8s/
    │   ├── deployment.yaml
    │   ├── service.yaml
    │   ├── hpa.yaml
    │   └── configmap.yaml
    ├── monitoring/
    │   ├── prometheus.yml
    │   ├── grafana-dashboard.json
    │   └── alerts.yml
    ├── scripts/
    │   ├── pre-sale-warmup.sh
    │   ├── load-test.sh
    │   └── reconciliation.sh
    └── src/
        ├── lib.rs
        └── main.rs

---

## TECHNICAL CONSTRAINTS

1. Language: Rust (2021 edition, stable toolchain)
2. Async runtime: Tokio (multi-threaded)
3. Web framework: Axum
4. Redis client: redis crate with connection pooling (deadpool-redis)
5. Database: PostgreSQL via SQLx
6. Serialization: serde + serde_json
7. Error handling: thiserror for library errors, anyhow for application errors
8. Logging: tracing + tracing-subscriber
9. Testing: built-in test framework + proptest for property tests
10. Benchmarking: criterion

---

## MODULE DEPENDENCY GRAPH

01-redis-fundamentals
    ↓
02-redis-lua-scripting
    ↓
03-atomic-counters ← 01 + 02
    ↓
04-traffic-shaping ← 01
    ↓
05-idempotency ← 01 + 02
    ↓
06-event-sourcing ← 01
    ↓
07-resilience (standalone concepts, uses all above)
    ↓
08-load-testing ← 11 (needs API to test)
    ↓
09-observability (integrates with all above)
    ↓
10-caching-strategy ← 01
    ↓
11-flash-sale-api ← ALL ABOVE (integration module)
    ↓
12-order-worker ← 06 + 11
    ↓
13-reconciliation ← 03 + 06
    ↓
14-integration-tests ← 11 + 12 + 13
    ↓
15-capstone-deployment ← 11 + 12 + 13 + 14

---

## EXERCISE FORMAT

Each exercise file (pXX_name.rs) must follow this structure:

```rust
//! # Exercise: [Title]
//!
//! ## Learning Objective
//! [What the learner should understand after completing this exercise]
//!
//! ## Context
//! [How this relates to the flash sale system]
//!
//! ## Instructions
//! [Step-by-step what to implement]
//!
//! ## Hints
//! [Conceptual hints, not code]

// ===== IMPLEMENTATION =====
// Learner writes code here

// ===== TESTS =====
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() { /* ... */ }

    #[test]
    fn test_edge_cases() { /* ... */ }

    #[test]
    fn test_concurrent_access() { /* ... */ }
}

// ===== BENCHMARKS (where applicable) =====
// In benches/ directory
```

---

## README FORMAT FOR EACH MODULE

Each module README.md must contain:

1. **Motivation**: Why this module exists in the flash sale context
2. **Concept Map**: Visual diagram of concepts and their relationships
3. **Theory**: Detailed explanation of algorithms and patterns
4. **Trade-offs**: What alternatives exist and why chosen approach is optimal
5. **Failure Modes**: What can go wrong and how to handle it
6. **Connection to Other Modules**: How this integrates with the full system
7. **Exercise List**: Table of exercises with difficulty rating (1-5)
8. **References**: Papers, documentation, blog posts

---

## EXERCISE DIFFICULTY DISTRIBUTION

Per module:
- 3 exercises at difficulty 1-2 (fundamentals, setup)
- 4 exercises at difficulty 3-4 (core implementation)
- 2-3 exercises at difficulty 5 (production-hardening, edge cases)

Total across all modules: ~130-150 exercises

---

## SUCCESS CRITERIA

After completing this project, the learner must be able to:

1. Design a flash sale architecture from scratch with no guidance
2. Implement atomic stock management that provably cannot oversell
3. Handle 100K+ concurrent requests without degradation
4. Implement idempotent purchase flows
5. Design and implement multi-layer rate limiting
6. Set up observability for real-time sale monitoring
7. Write load tests that validate system capacity
8. Design reconciliation processes for data consistency
9. Implement resilience patterns (circuit breaker, bulkhead, fallback)
10. Deploy and operate the system with Kubernetes

---

## CONSTRAINTS FOR THE AGENT

1. Every Rust file must compile with `cargo build` and pass `cargo test`
2. Every exercise must have at least 3 tests (basic, edge case, concurrent)
3. Lua scripts must be included as string constants with inline comments
4. Redis-dependent tests must use testcontainers or a mock Redis
5. Database-dependent tests must use testcontainers or SQLite in-memory
6. Load tests must produce measurable metrics (throughput, latency percentiles)
7. All benchmarks must use criterion with proper statistical analysis
8. No placeholder code, no TODO comments, no unimplemented!() in final version
9. README diagrams must use ASCII art or Mermaid syntax
10. The final capstone must be deployable with a single `docker-compose up`
```

---

Save this as `flash-sale-spec.md` and feed it to your agent. It contains every piece of knowledge mapped from your existing repos, every gap specified with learning objectives, the complete project structure with 15 modules and ~140 exercises, dependency ordering, technical constraints, and success criteria.