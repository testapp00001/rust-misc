# Consistency Models — Reference Document

Hierarchy from strongest to weakest, with formal definitions, example scenarios, and real systems.

---

## 1. Linearizability (Strongest)

### Definition

Every operation appears to take effect atomically at some point between its invocation and its response. This is the strongest consistency model — it makes a distributed system behave as if there is a single copy of the data.

### Formal Property

For any execution E, there exists a total order ≺ on all operations such that:
1. If operation A completes before operation B begins (real-time), then A ≺ B
2. The result of each read is the value of the most recent write in ≺

### Example

```
Client 1: SET(x, 1)──────────────────────done
Client 2:          GET(x)──→returns 1
Client 3:          GET(x)──→returns 0   ← VIOLATION (stale read)
```

Linearizability forbids Client 3's stale read because SET(x,1) completed before GET(x) began.

### Real Systems

- etcd (single-key operations)
- Google Spanner (via TrueTime)
- CockroachDB (serializable isolation)
- ZooKeeper (reads from leader)

### Cost

Requires coordination (consensus or locking). High latency, reduced availability during partitions.

---

## 2. Sequential Consistency

### Definition

All operations appear in some total order that is consistent with each process's program order. Unlike linearizability, there is no real-time constraint — the order just needs to be consistent with each client's local order.

### Formal Property

For any execution E, there exists a total order ≺ on all operations such that:
1. If operation A is before operation B in some process's program order, then A ≺ B
2. Each read returns the value of the most recent write in ≺

### Example

```
Process 1: SET(x, 1)──SET(y, 1)
Process 2: SET(x, 2)──SET(y, 2)

Valid sequential: SET(x,1)→SET(y,1)→SET(x,2)→SET(y,2) → reads: x=2, y=2
Valid sequential: SET(x,2)→SET(x,1)→SET(y,2)→SET(y,1) → reads: x=1, y=1
Invalid: x=2, y=1 (would require reordering within a process)
```

### Difference from Linearizability

Sequential consistency doesn't require the total order to respect real-time ordering. Two operations that happen in real-time order can appear in reverse order in the sequential order, as long as each process's own operations are ordered correctly.

### Real Systems

- x86 memory model (TSO — Total Store Order)
- Some configurations of ZooKeeper

---

## 3. Causal Consistency

### Definition

Operations that are causally related are seen in the same order by all processes. Concurrent operations (not causally related) may be seen in different orders by different processes.

### Causal Ordering (Lamport's happened-before)

Operation A causally precedes operation B (A → B) if:
1. A and B are on the same process, and A comes before B in program order
2. A is a send and B is the corresponding receive
3. There exists C such that A → C and C → B (transitivity)

### Example

```
Process 1: write(x=1)──send(msg)──→
Process 2:                         recv(msg)──write(y=1)

Causal: write(x=1) → write(y=1) (via message passing)
All processes must see write(x=1) before write(y=1)
```

But concurrent writes (on different processes, no causal link) can be seen in any order.

### Real Systems

- MongoDB (causal consistency sessions)
- COPS (Consistency protocol for partial services)
- GentleRain (causal consistency for geo-replicated stores)

---

## 4. Read-Your-Writes

### Definition

A process always reads its own most recent write. Other processes may see older values.

### Example

```
Process 1: write(x=1)──read(x)──→must return 1
Process 2: read(x)──→may return old value (no guarantee)
```

### Real Systems

- Most single-node databases (trivially satisfied)
- Session stickiness in load balancers (route to same replica)
- DynamoDB (consistent reads from same partition)

---

## 5. Monotonic Reads

### Definition

Once a process reads a value v for key k, all subsequent reads of k by the same process will return v or a more recent value. A process will never "go back in time."

### Example

```
Process 1: read(x)──→returns 1
Process 1: read(x)──→returns 0   ← VIOLATION (went back in time)
Process 1: read(x)──→returns 2   ← OK (more recent)
```

### Real Systems

- Session stickiness in load balancers
- Read replicas with causal ordering

---

## 6. Monotonic Writes

### Definition

A process's writes are applied in the order they were issued. If process P writes x=1 then x=2, all processes will see x=1 before x=2 (or just see x=2 if they missed x=1, but never see x=2 then x=1).

### Real Systems

- Most databases with per-client session ordering

---

## 7. Write-Follows-Reads

### Definition

If a process reads a value and then writes, the write is causally dependent on the read. Other processes that see the write must also see the read (or a more recent value).

### Example

```
Process 1: read(x)──→returns 1──write(y=x+1)
Process 2: read(y)──→must have seen x=1 (or later) before
```

### Real Systems

- Implemented via causal metadata (vector clocks, version vectors)

---

## 8. Eventual Consistency (Weakest)

### Definition

If no new updates are made, all replicas will eventually converge to the same value. **No guarantee on when convergence happens.** During convergence, reads may return stale values.

### Example

```
Replica A: write(x=1)
Replica B: read(x)──→may return old value
...some time passes...
Replica B: read(x)──→eventually returns 1
```

### Real Systems

- DNS
- Amazon S3 (before strong consistency was added)
- Cassandra (default)
- DynamoDB (eventually consistent reads)
- CouchDB

### Problems

- Can return arbitrarily stale data
- No guarantee of convergence time
- Conflict resolution needed (last-write-wins, application merge, CRDTs)

---

## 9. Strong Eventual Consistency (SEC)

### Definition

Replicas that have delivered the same set of updates have the same state. **No divergence after sync.** This is strictly stronger than eventual consistency but weaker than the models above.

### Requirements

Merge operations must be:
- **Commutative**: merge(A, B) = merge(B, A)
- **Associative**: merge(merge(A, B), C) = merge(A, merge(B, C))
- **Idempotent**: merge(A, A) = A

### Real Systems

- CRDTs (all types)
- Riak (with CRDT data types)
- Redis CRDB

### Advantage over Eventual Consistency

No "divergence then merge" cycle. Replicas are always in a consistent state relative to the updates they've seen.

---

## Comparison Table

| Model | Real-time order | Causal order | Per-process order | Coordination needed |
|-------|:-:|:-:|:-:|:-:|
| Linearizability | ✓ | ✓ | ✓ | High |
| Sequential | ✗ | ✓ | ✓ | High |
| Causal | ✗ | ✓ | ✓ | Medium |
| Read-Your-Writes | ✗ | ✗ | ✓ (self only) | Low |
| Monotonic Reads | ✗ | ✗ | ✓ (self only) | Low |
| Eventual | ✗ | ✗ | ✗ | None |
| SEC (CRDTs) | ✗ | ✗ | ✗ | None (merge only) |

---

## Choosing a Consistency Model

### Use Linearizability when:
- Financial transactions (no double-spending)
- Unique constraints (no duplicate usernames)
- Leader election (at most one leader)
- Systems: etcd, ZooKeeper, Spanner

### Use Causal Consistency when:
- Social media feeds (comments must appear after the post they reply to)
- Collaborative editing (operations must respect causal order)
- Systems: MongoDB, COPS

### Use Read-Your-Writes when:
- User profile updates (user should see their own changes)
- Shopping cart (user should see items they just added)
- Systems: DynamoDB (consistent reads), session stickiness

### Use Eventual Consistency when:
- DNS records
- Content delivery networks
- Analytics data
- Systems: Cassandra, S3, CouchDB

### Use CRDTs (SEC) when:
- Distributed counters (page views, likes)
- Shopping carts (concurrent add/remove)
- Collaborative data structures
- Systems: Riak, Redis CRDB
