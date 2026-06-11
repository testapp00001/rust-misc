# Solution 01: CAP Theorem and PACELC

## Part A: CAP Theorem Classification

**1. etcd -- CP**
etcd uses Raft consensus. During a network partition, if a quorum cannot be reached, etcd refuses writes. It sacrifices availability to maintain strong consistency. Every read from the leader is guaranteed to be up-to-date.

**2. Cassandra -- AP**
Cassandra uses eventual consistency. During a partition, any node can accept writes (even without a quorum). Conflicts are resolved later using last-write-wins or vector clocks. It sacrifices consistency to maintain availability.

**3. PostgreSQL (single node) -- CA**
A single-node PostgreSQL is both consistent and available, but it does not tolerate partitions because it is not distributed. This is the only way to achieve "CA" -- by not being a distributed system. The moment you add replication, you must choose CP or AP.

**4. MongoDB (with majority write concern) -- CP**
With `w: "majority"`, MongoDB requires a majority of nodes to acknowledge writes. During a partition, the minority side cannot achieve majority and refuses writes. The majority side continues with strong consistency. This is CP behavior.

**5. DynamoDB -- AP**
DynamoDB accepts reads and writes even during partitions. It uses eventual consistency by default (you can opt into strongly consistent reads at the cost of higher latency). Conflicts are resolved using last-write-wins.

**6. CockroachDB -- CP**
CockroachDB uses Raft consensus and serializable isolation. During a partition, if a quorum cannot be reached, transactions are refused. It sacrifices availability to guarantee serializable consistency across all nodes.

### Common Mistakes to Avoid

- Classifying PostgreSQL as "CA" in a distributed context. A single node is CA, but a replicated PostgreSQL cluster is CP (synchronous replication) or AP (asynchronous replication).
- Saying Cassandra is "eventually consistent, therefore CP." Eventual consistency is AP behavior -- it provides availability at the cost of consistency.
- Forgetting that the CAP theorem applies during partitions only. When there is no partition, all three properties can be achieved.

---

## Part B: PACELC Analysis

| System | Partition: A or C | Else: L or C |
|--------|-------------------|---------------|
| **Cassandra** | A (always writable, even during partitions) | L (eventual consistency, optimized for low latency) |
| **MongoDB** | C (majority write concern refuses writes without quorum) | C (reads from primary, strong consistency) |
| **DynamoDB** | A (always writable, eventual consistency) | L (eventual consistency by default, low latency reads) |
| **PostgreSQL** | C (synchronous replication blocks until replicas confirm) | C (strong consistency, reads from primary) |
| **CockroachDB** | C (serializable transactions require quorum) | C (serializable isolation, higher latency for consistency) |

### Why This Matters

PACELC reveals trade-offs that CAP alone misses. For example:
- MongoDB and CockroachDB are both CP, but MongoDB offers lower latency in the non-partition case (reads from primary without cross-node coordination) while CockroachDB maintains serializable consistency at higher latency.
- Cassandra and DynamoDB are both AP, but Cassandra's tunable consistency lets you choose per-query (e.g., `QUORUM` reads for strong consistency on critical paths).

### Common Mistakes to Avoid

- Treating PACELC as "CAP plus one more letter." The "Else" case is often more important in practice because partitions are rare.
- Not recognizing that some systems let you tune the trade-off (Cassandra consistency levels, DynamoDB strongly consistent reads).

---

## Part C: Real-World Impact

**Requirement 1: User session store**
**Recommendation: AP system (Cassandra or DynamoDB)**
A stale session (e.g., user sees an old profile picture for 5 minutes) is annoying but not catastrophic. Availability is more important -- if users cannot log in during a partition, they cannot use the application at all. The cost of inconsistency is low (temporary UI glitch). The cost of unavailability is high (complete loss of access).

**Requirement 2: Financial ledger**
**Recommendation: CP system (CockroachDB or PostgreSQL with synchronous replication)**
A stale financial record (e.g., balance shows $100 when it should be $50) is a compliance violation and can cause real financial harm. It is better to refuse transactions during a partition than to risk inconsistency. The cost of inconsistency is high (regulatory fines, customer trust). The cost of unavailability is manageable (transactions queue up and process after the partition heals).

### Common Mistakes to Avoid

- Choosing AP for financial data because "we need high availability." Financial systems need correctness, not availability. A brief outage is acceptable; an incorrect balance is not.
- Choosing CP for session data because "we need consistency." Session data is user-facing and tolerant of staleness. Unavailability is worse than a stale session.

---

## Key Takeaway

The CAP theorem constrains distributed systems to choose between consistency and availability during partitions. PACELC extends this by acknowledging the latency vs. consistency trade-off even when there is no partition. The right choice depends on the cost of inconsistency: low for session data, high for financial data. In practice, most systems are AP for user-facing components and CP for financial/inventory components.
