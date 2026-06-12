# Distributed Systems Theorems — Reference Document

Precise statements of the fundamental theorems, impossibility results, and bounds that govern all distributed systems.

---

## 1. Two Generals' Problem (Akkoyunlu et al., 1975)

### Statement

Two armies, each led by a general, are positioned on opposite sides of a valley. They must agree on a common time to attack. Messengers can be captured (messages lost). **No finite protocol guarantees that both generals will agree on the same attack time.**

### Formal Statement

In an asynchronous network where messages can be lost, there is no deterministic protocol that guarantees agreement between two parties in a finite number of messages.

### Proof Sketch (by contradiction)

Assume a protocol exists that guarantees agreement with k messages. The last message (k-th) could be lost. The sender of message k cannot know if it was received, so they cannot commit. The receiver, not receiving it, cannot commit either. Therefore the protocol needs a (k+1)-th acknowledgment, contradicting the assumption. By induction, no finite k works.

### Implication

Reliable communication over an unreliable network is impossible. Every distributed algorithm is a workaround for this impossibility. TCP's three-way handshake does not solve it — TCP connections can reset.

### Real-world Coping Mechanisms

- Timeout + retry + idempotency
- At-least-once delivery (duplicate detection via idempotency keys)
- At-most-once delivery (fire and forget)
- "Exactly-once" is actually "effectively-once" via idempotency

---

## 2. Byzantine Generals' Problem (Lamport, Shostak, Pease, 1982)

### Statement

A group of generals, each commanding a division of the Byzantine army, surround a city. They must agree on a common plan of action (attack or retreat). Some generals may be traitors (Byzantine faults) who send conflicting messages. **A system with n generals can tolerate at most f Byzantine faulty generals if and only if n ≥ 3f + 1.**

### Formal Statement

For any system of n processes where f may exhibit Byzantine (arbitrary) behavior, no protocol can guarantee agreement unless n ≥ 3f + 1.

### Proof Sketch

With f faulty nodes and f "suspect" nodes that might be slow, you need f+1 correct nodes for a majority. Total: f faulty + f slow + f+1 correct = 3f+1. With fewer than 3f+1 nodes, the faulty nodes can create a situation where correct nodes cannot distinguish truth from lies.

### Implication

Byzantine fault tolerance is strictly harder than crash fault tolerance. Crash fault tolerance requires n ≥ 2f+1 (Raft, Paxos). Byzantine fault tolerance requires n ≥ 3f+1 (PBFT).

---

## 3. FLP Impossibility (Fischer, Lynch, Paterson, 1985)

### Statement

**In an asynchronous system, if there is even one process that might crash, no deterministic algorithm can guarantee consensus.**

More precisely: there is no deterministic algorithm that simultaneously satisfies:
- **Termination**: All correct processes eventually decide
- **Agreement**: All correct processes decide the same value
- **Validity**: The decided value was proposed by some process

### Formal Statement

Let P be a consensus protocol for n ≥ 2 processes in an asynchronous system where at most one process may crash. Then there exists an initial configuration and an execution of P in which no process decides.

### Proof Sketch (Indistinguishability)

Consider two executions:
1. Process p crashes at time t
2. Process p is just slow (delayed beyond observation)

In an asynchronous system, these are indistinguishable to other processes. The algorithm cannot know whether to wait for p (risking non-termination) or proceed without p (risking incorrectness if p wasn't crashed).

### Implication

Consensus is impossible with guaranteed termination. Real systems work around FLP using:
- **Partial synchrony**: Eventually, messages arrive within bounded time
- **Failure detectors**: Eventually accurate detector, not perfect
- **Randomization**: Randomized consensus protocols (expected termination)
- **Leader election with timeouts**: Raft's approach

### Note

FLP does NOT mean consensus is impossible. It means consensus with **guaranteed termination in all executions** is impossible. In practice, systems assume partial synchrony and achieve consensus with high probability.

---

## 4. CAP Theorem (Brewer, 2000; Gilbert & Lynch, 2002)

### Statement

A distributed data store can provide at most two of the following three guarantees:
- **Consistency (C)**: Every read returns the most recent write (linearizability)
- **Availability (A)**: Every request receives a non-error response
- **Partition Tolerance (P)**: The system continues to operate despite network partitions

### Formal Statement (Gilbert & Lynch, 2002)

It is impossible for a web service to provide the following three guarantees simultaneously:
1. Consistency (linearizability)
2. Availability (every request receives a response)
3. Partition tolerance (the system operates despite message loss)

### The Real Choice

Since partitions WILL happen (network is unreliable), P is not optional. The real choice is:
- **CP system**: Sacrifice availability during partitions to maintain consistency (etcd, HBase, MongoDB)
- **AP system**: Sacrifice consistency during partitions to maintain availability (Cassandra, DynamoDB, CouchDB)
- **CA system**: Does not exist in practice (requires perfect network)

### Common Misconception

"Pick 2 of 3" is misleading. You don't pick P — partitions happen to you. The choice is: when a partition occurs, do you sacrifice C or A?

---

## 5. PACELC Theorem (Abadi, 2012)

### Statement

**IF** there is a **P**artition: choose between **A**vailability and **C**onsistency
**E**LSE (normal operation): choose between **L**atency and **C**onsistency

### Classification

| System | Partition (C vs A) | Normal (L vs C) | Examples |
|--------|-------------------|-----------------|----------|
| PA/EL | Available | Low latency | Cassandra, DynamoDB |
| PC/EC | Consistent | Consistent | etcd, CockroachDB |
| PA/EC | Available | Consistent | Some systems |
| PC/EL | Consistent | Low latency | Some systems |

### Implication

CAP only describes behavior during partitions. PACELC adds the important insight that even during normal operation, there is a trade-off between latency and consistency. Strong consistency (linearizability) requires coordination, which adds latency.

---

## 6. Consensus Lower Bounds

### Crash Fault Tolerance

**A system of n processes can tolerate f crash faults if and only if n ≥ 2f + 1.**

Proof: With f faulty nodes, you need a majority of non-faulty nodes for quorum. Majority of (n - f) > f requires n ≥ 2f + 1.

### Byzantine Fault Tolerance

**A system of n processes can tolerate f Byzantine faults if and only if n ≥ 3f + 1.**

(See Byzantine Generals' Problem above for proof sketch.)

### Implication

- Raft uses 3 nodes to tolerate 1 crash fault (2×1 + 1 = 3)
- Raft uses 5 nodes to tolerate 2 crash faults (2×2 + 1 = 5)
- PBFT uses 4 nodes to tolerate 1 Byzantine fault (3×1 + 1 = 4)
- PBFT uses 7 nodes to tolerate 2 Byzantine faults (3×2 + 1 = 7)

---

## 7. Spanner's TrueTime Consistency Guarantee (Corbett et al., 2012)

### Statement

Google Spanner achieves **external consistency** (linearizability across globally distributed replicas) using **TrueTime**, an API that returns a time interval [earliest, latest] with bounded uncertainty.

### Mechanism

1. TrueTime uses atomic clocks + GPS receivers in each data center
2. `TT.now()` returns `[earliest, latest]` with typical uncertainty of 1-7ms
3. After a write, the transaction waits for the uncertainty to elapse before reporting success (**commit wait**)
4. This guarantees that the commit timestamp is in the real-time past when the response reaches the client

### Formal Guarantee

If transaction T1 completes before transaction T2 begins (real-time), then T1's commit timestamp is less than T2's commit timestamp.

### Implication

Physical clocks CAN provide global ordering IF you can bound the uncertainty. This is the only known way to achieve linearizability across geographic regions without a single data center.

---

## 8. CALM Theorem (Hellerstein & Alvaro, 2019)

### Statement

**A program has a consistent, coordination-free distributed implementation if and only if it is monotonic.**

### Informal Statement

If adding more information (inputs) to a computation can only add more outputs (never retract previous outputs), then the computation can be distributed without coordination (consensus).

### Examples

- **Monotonic** (coordination-free): Set union, adding elements to a CRDT
- **Non-monotonic** (requires coordination): Removing elements, maintaining a unique constraint, ensuring a total order

### Implication

CRDTs are CALM-compliant: their merge operations are monotonic. This is why CRDTs don't need consensus. Non-monotonic operations (like ensuring exactly-once delivery) require coordination.

---

## 9. Eventual Consistency vs Strong Eventual Consistency

### Eventual Consistency

If no new updates are made, all replicas will eventually converge to the same value. **No guarantee on when convergence happens.**

### Strong Eventual Consistency (SEC)

Replicas that have delivered the same set of updates have the same state. **No divergence after sync.**

### CRDTs Guarantee SEC

CRDTs achieve SEC because their merge operations are:
- **Commutative**: merge(A, B) = merge(B, A)
- **Associative**: merge(merge(A, B), C) = merge(A, merge(B, C))
- **Idempotent**: merge(A, A) = A

These properties guarantee that replicas converge as soon as they have received the same updates, regardless of delivery order.

---

## 10. Two-Phase Commit Blocking Theorem

### Statement

Two-Phase Commit (2PC) is a **blocking protocol**: if the coordinator crashes after sending the "prepare" response but before sending the "commit/abort" decision, participants are **blocked** (holding locks) until the coordinator recovers.

### Proof

After phase 1, participants have voted YES and are in a "prepared" state. They cannot unilaterally commit (they don't know if all others voted YES) or abort (the coordinator might have decided COMMIT). They must wait for the coordinator's decision.

### Implication

2PC provides atomicity but not availability. Three-Phase Commit (3PC) attempts to solve this but requires bounded message delays (synchronous network assumption), which doesn't hold in practice.

---

## 11. CAP Consistency Hierarchy

From strongest to weakest:

1. **Linearizability**: Operations appear to execute atomically at some point between invocation and response (real-time ordering)
2. **Sequential Consistency**: All operations appear in some total order consistent with each process's program order (no real-time constraint)
3. **Causal Consistency**: Causally related operations are seen in order; concurrent operations may be seen in different orders
4. **Read-Your-Writes**: A process always reads its own most recent write
5. **Monotonic Reads**: Once a process reads value v, it will never read an older value
6. **Monotonic Writes**: A process's writes are applied in the order they were issued
7. **Write-Follows-Reads**: A write that follows a read reflects the read value
8. **Eventual Consistency**: If no new updates, all replicas converge (no guarantee on when)

See [consistency-models.md](consistency-models.md) for detailed definitions.
