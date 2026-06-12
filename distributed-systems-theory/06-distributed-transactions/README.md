# Module 06: Distributed Transactions

## Overview

Distributed transactions ensure atomicity and consistency across multiple services or
databases. This module covers the major protocols and patterns used in production systems.

## Protocols and Patterns

### Two-Phase Commit (2PC)

2PC is the classic distributed atomic commit protocol:

1. **Phase 1 (Prepare):** The coordinator asks all participants to vote (YES/NO).
   Each participant executes the transaction up to the commit point and votes.
2. **Phase 2 (Commit/Abort):** If all participants vote YES, the coordinator sends
   COMMIT. Otherwise, it sends ABORT.

**Blocking problem:** If the coordinator crashes after phase 1, participants are stuck.
They have voted but don't know the outcome. They must hold locks until the coordinator
recovers, which can cause long delays.

### Three-Phase Commit (3PC)

3PC adds a pre-commit phase to reduce the blocking window:

1. **Phase 1 (Prepare):** Same as 2PC.
2. **Phase 2 (Pre-commit):** Coordinator tells participants to prepare for commit.
   Participants acknowledge.
3. **Phase 3 (Commit):** Coordinator tells participants to commit.

If the coordinator crashes after pre-commit, participants can proceed with commit since
they know all participants were in the pre-commit state. However, 3PC can still violate
consistency under network partitions.

### Saga Pattern

Sagas decompose a distributed transaction into a sequence of local transactions, each
with a compensating transaction:

- If step k fails, compensating transactions are executed in reverse order for steps
  1 through k-1.
- Two coordination strategies:
  - **Choreography:** Each service publishes events that trigger the next service.
  - **Orchestration:** A central coordinator directs each step.

**Trade-off:** Sagas provide eventual consistency, not strong consistency. Intermediary
states are visible to other transactions.

### Transactional Outbox Pattern

To reliably publish events as part of a database transaction:

1. Write business data and an event to an "outbox" table in the same transaction.
2. A separate process polls the outbox and publishes events to a message broker.
3. Consumers deduplicate events using idempotency keys.

This avoids the dual-write problem (database + message broker are not atomic).

### Choosing the Right Approach

| Pattern | Consistency | Latency | Complexity | When to Use |
|---------|------------|---------|------------|-------------|
| 2PC | Strong | High (blocking) | Medium | Financial systems, ACID requirements |
| 3PC | Strong (no partitions) | Medium | High | Theoretical; rarely used in practice |
| Saga | Eventual | Low | Medium | Microservices, long-running workflows |
| Outbox | Eventual | Low | Medium | Event-driven architectures |

## Exercises

| # | Exercise | Difficulty | Description |
|---|----------|------------|-------------|
| 01 | Two-Phase Commit | ★★☆ | Implement coordinator and participants |
| 02 | 2PC Coordinator Crash | ★★★ | Show blocking when coordinator crashes |
| 03 | Three-Phase Commit | ★★★ | Add pre-commit phase to reduce blocking |
| 04 | 3PC Partition | ★★★★ | Show consistency violation under partition |
| 05 | Saga Choreography | ★★★ | Implement event-driven saga with 3 services |
| 06 | Saga Orchestration | ★★★ | Central coordinator saga with compensations |
| 07 | Saga Compensation | ★★★ | Deep dive into compensating transactions |
| 08 | Transactional Outbox | ★★★★ | Implement outbox pattern with deduplication |
| 09 | Isolation Analysis | ★★★★ | Analyze dirty reads and lost updates in Sagas |
| 10 | Flash Sale Saga | ★★★★★ | End-to-end saga for flash sale purchase flow |

## Key Trade-offs

- **2PC** guarantees atomicity but blocks on coordinator failure.
- **3PC** reduces blocking but is vulnerable to network partitions.
- **Sagas** avoid blocking but sacrifice isolation (intermediate states visible).
- **Outbox** solves the dual-write problem but adds polling latency.
