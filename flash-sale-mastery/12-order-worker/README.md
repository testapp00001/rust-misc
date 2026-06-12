# Module 12: Async Order Worker

> The hot path completes in under 5ms -- order creation, payment, and
> notifications happen asynchronously through Redis Streams.

## Motivation

In Module 11, the flash sale API accepts a claim and writes an event to a Redis
Stream in a single atomic Lua script. That keeps the hot path fast, but the work
is not done: we still need to create an order record, charge the customer, and
send a confirmation email. Performing those steps synchronously would blow the
latency budget. Instead, a background worker consumes events from the stream and
processes them at its own pace. This separation of concerns is fundamental to
building systems that are both fast and reliable.

## Concept Map

```
[Module 11: Flash Sale API]
        │
        │  XADD orders:stream
        ▼
┌───────────────────┐
│  Redis Stream     │  orders:stream
│  (message queue)  │
└───────────────────┘
        │
        │  XREADGROUP
        ▼
┌───────────────────┐
│  p01: Consumer    │  Consumer group with crash recovery (PEL)
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  p02: Order       │  Idempotent: duplicate events → same order
│  Processor        │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  p03: Payment     │  Circuit breaker protects downstream service
│  Handler          │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  p04: Notification│  Best-effort: failure here does not block order
│  Handler          │
└───────────────────┘
        │
        ▼
┌───────────────────┐
│  p05: Dead Letter │  Failed events retry up to N times, then park
│  Queue            │
└───────────────────┘
```

## Theory

### Consumer Groups

Redis Streams support **consumer groups**, which provide:

- **Load balancing**: Each message is delivered to exactly one consumer in the
  group (unlike Pub/Sub where every subscriber gets every message).
- **Acknowledgement**: Messages are not removed from the stream until a consumer
  sends `XACK`. If a consumer crashes before acknowledging, the message remains
  in the Pending Entry List (PEL).
- **Crash recovery**: Another consumer can claim pending messages from a crashed
  consumer using `XAUTOCLAIM` or `XCLAIM`.

### At-Least-Once Delivery

Redis Streams guarantee **at-least-once** delivery within a consumer group. A
message may be delivered again if:

- The consumer processes it but crashes before calling `XACK`.
- The consumer is too slow and the message is reclaimed by another consumer.

This means downstream processing (order creation, payment) must be
**idempotent** -- processing the same event twice must produce the same result.

### Idempotent Consumers

An idempotent consumer checks whether an event has already been processed before
doing work. Common strategies:

| Strategy | Where | Speed | Durability |
|----------|-------|-------|------------|
| Redis SET NX | Before processing | ~0.1ms | Volatile |
| DB unique constraint | During insert | ~1-5ms | Durable |
| Combined | Both layers | ~0.1ms fast path | Durable |

## Trade-offs

### Redis Streams vs Kafka vs RabbitMQ

| Feature | Redis Streams | Kafka | RabbitMQ |
|---------|---------------|-------|----------|
| Latency | Sub-ms | 2-10ms | 1-5ms |
| Throughput | 100K+ msg/sec | 1M+ msg/sec | 50K+ msg/sec |
| Persistence | Optional AOF | Always on disk | Configurable |
| Consumer groups | Built-in | Built-in | Via competing consumers |
| Message replay | Yes (by ID) | Yes (by offset) | No (once consumed) |
| Operational cost | Low (already using Redis) | High (ZooKeeper/KRaft) | Medium |
| Best for | Small-to-medium queues, low latency | Large-scale event streaming | Complex routing, RPC |

For a flash sale learning project, Redis Streams is the natural choice: it adds
no new infrastructure, supports consumer groups natively, and keeps latency low.

## Failure Modes

1. **Consumer crash mid-processing**
   - The event stays in the PEL. On restart (or via XAUTOCLAIM), another consumer
     reclaims and reprocesses it. Idempotent processing ensures correctness.

2. **Message loss (Redis restart without AOF)**
   - If Redis is not configured with `appendonly yes`, unprocessed stream entries
     are lost on restart. Mitigation: enable AOF with `appendfsync everysec`.

3. **Duplicate processing**
   - At-least-once delivery means duplicates happen. The order processor must be
     idempotent (check for existing order before inserting).

4. **Payment service unavailable**
   - The circuit breaker trips after repeated failures, fast-failing subsequent
     payment attempts. Events go to the dead letter queue for later retry.

5. **Notification failure**
   - Notifications are best-effort. A failed email should not prevent the order
     from being recorded. Log the failure and move on.

## Connection to Other Modules

- **Module 06 (Event Sourcing)**: The Redis Stream is an event log. Each entry
  represents an immutable fact ("account X claimed product Y"). The order worker
  is a projection that materializes these events into order records.
- **Module 11 (Flash Sale API)**: The API's Lua script writes events to the
  stream via `XADD`. This module is the consumer side of that same stream.
- **Module 07 (Resilience)**: The circuit breaker in the payment handler uses
  the same patterns taught in the resilience module.
- **Module 05 (Idempotency)**: The order processor's duplicate detection is an
  application of the idempotency patterns from Module 05.

## Exercises

| # | Exercise | Focus |
|---|----------|-------|
| 01 | Redis Streams Consumer | Consumer groups, XREADGROUP, PEL recovery, XACK |
| 02 | Order Processor | Idempotent event-to-order mapping, validation |
| 03 | Payment Handler | Circuit breaker, mock payment simulation |
| 04 | Notification Handler | Best-effort delivery, error isolation |
| 05 | Dead Letter Queue | Failed event tracking, retry with backoff |

## References

- [Redis Streams Documentation](https://redis.io/docs/data-types/streams/)
- [Redis Consumer Groups](https://redis.io/docs/data-types/streams/#consumer-groups)
- [Introduction to Redis Streams (antirez)](http://antirez.com/news/114)
- [Designing Data-Intensive Applications, Ch. 11 - Stream Processing](https://dataintensive.net/)
- [Circuit Breaker Pattern - Martin Fowler](https://martinfowler.com/bliki/CircuitBreaker.html)
- [Enterprise Integration Patterns - Dead Letter Channel](https://www.enterpriseintegrationpatterns.com/patterns/messaging/DeadLetterChannel.html)
