# Solution 01: Replication Topology Design

---

## Part A: Application Classification

| Application | Topology | Replicas | Replication Mode | Rationale |
|-------------|----------|----------|-----------------|-----------|
| Banking ledger | Master-slave | 2 (sync + async) | Synchronous to 1, async to 1 | Strong consistency requires sync replication. Async replica for disaster recovery. |
| Social media feed | Multi-master | 2 per region (6 total) | Asynchronous | Eventual consistency is acceptable. Local writes reduce latency. |
| Inventory system | Master-slave | 1 (async) | Asynchronous | Single write region. One async replica for read scaling and failover. |
| Content delivery | Multi-master | 2 per region (6 total) | Asynchronous | Content is rarely updated. Multi-master gives lowest read latency globally. |
| Analytics warehouse | Single primary | 0 replicas | N/A | Low availability target. Reads can tolerate high latency. Single node is sufficient. |

### Key Insight: Consistency Drives Topology

The consistency requirement is the primary decision factor:
- **Strong consistency** requires synchronous replication, which limits you to master-slave or single-node topologies
- **Eventual consistency** enables multi-master, which provides better latency and availability
- **No replication** is viable when availability requirements are low and the workload fits on one node

## Part B: Topology Diagrams

### Banking Ledger: Master-Slave with Sync + Async

```
                    +-----------------+
                    |   Application   |
                    |   (US-East)     |
                    +--------+--------+
                             |
                    +--------v--------+
                    |   PgBouncer     |
                    | (read/write     |
                    |  routing)       |
                    +--------+--------+
                             |
              +--------------+--------------+
              |                             |
    +---------v---------+      +-----------v-----------+
    |     PRIMARY       |      |    SYNC REPLICA       |
    |    (US-East-1a)   |----->|    (US-East-1b)       |
    |   Accepts writes  | sync |   Hot standby         |
    |   Accepts reads   |      |   Accepts reads       |
    +-------------------+      +-----------------------+
              |
              | async
              v
    +-------------------+
    |   ASYNC REPLICA   |
    |   (US-West-2a)    |
    |   Hot standby     |
    |   DR purposes     |
    +-------------------+
```

**Routing rules:**
- All writes go to the primary
- Reads can go to the sync replica (strong consistency guaranteed)
- Async replica is for disaster recovery only, not for serving reads (to avoid stale data)

### Social Media Feed: Multi-Master

```
   US Region                    EU Region                  APAC Region
+----------------+         +----------------+         +----------------+
|   PRIMARY US   |<------->|   PRIMARY EU   |<------->|  PRIMARY APAC  |
|  (US-East-1a)  |  async  | (EU-West-1a)   |  async  | (AP-SE-1a)     |
+-------+--------+         +-------+--------+         +-------+--------+
        |                          |                          |
   async|                     async|                     async|
        v                          v                          v
+-------+--------+         +-------+--------+         +-------+--------+
|  REPLICA US    |         |  REPLICA EU    |         | REPLICA APAC   |
| (US-East-1b)   |         | (EU-West-1b)   |         | (AP-SE-1b)     |
+----------------+         +----------------+         +----------------+
```

**Routing rules:**
- US users write to PRIMARY US, read from REPLICA US
- EU users write to PRIMARY EU, read from REPLICA EU
- APAC users write to PRIMARY APAC, read from REPLICA APAC
- Bi-directional async replication keeps all regions eventually consistent
- Application handles conflict resolution (last-writer-wins for feed posts)

## Part C: Trade-off Analysis

### Banking Ledger (Master-Slave, Sync)

**Gain:** Zero data loss on failover. The sync replica has an exact copy of the primary. Reads from the sync replica are guaranteed to be current.

**Sacrifice:** Write latency increases by the network round-trip to the sync replica (~2ms within a region). If the sync replica fails, writes block until a new sync replica is designated or sync mode is disabled.

**Failure scenario:** If the sync replica becomes unresponsive and the primary cannot reach it, PostgreSQL blocks all writes (because it cannot confirm replication). This is the cost of zero data loss -- availability suffers when the sync replica is down. Mitigation: use `synchronous_standby_names = 'FIRST 1 (replica_a, replica_b)'` to allow automatic fallback to another replica.

### Social Media Feed (Multi-Master, Async)

**Gain:** Users in each region get local read and write latency (< 10ms). The system survives the loss of any single region.

**Sacrifice:** Conflicts can occur when two users in different regions interact with the same data. Conflict resolution is complex and may lose data.

**Failure scenario:** During a network partition between US and EU, users in both regions can still write. When the partition heals, conflicting writes must be resolved. If a user in US likes a post while a EU user deletes the same post, the conflict resolution must decide whether the like survives or is discarded with the deleted post.

### Inventory System (Master-Slave, Async)

**Gain:** Simple architecture, strong consistency for writes, one async replica for read scaling and failover.

**Sacrifice:** If the primary fails, the async replica may have lag, causing data loss equal to the lag duration. RPO is measured in seconds, not zero.

**Failure scenario:** During a bulk inventory update, the async replica falls 5 seconds behind. If the primary fails during this window, the last 5 seconds of inventory changes are lost. For an inventory system, this could mean overselling products.

### Content Delivery (Multi-Master, Async)

**Gain:** Content is served from the nearest region with minimal latency. Content updates propagate eventually.

**Sacrifice:** Content may be stale for a short period after updates. If the same content is edited in two regions simultaneously, one edit may be overwritten.

**Failure scenario:** A content editor in US updates an article while an editor in EU updates the same article. With last-writer-wins, one edit is silently lost. The editor whose edit was lost sees their changes disappear after the conflict resolution runs.

### Analytics Warehouse (Single Node)

**Gain:** Simplest architecture. No replication overhead, no conflict resolution, no failover complexity.

**Sacrifice:** Single point of failure. If the node goes down, the analytics dashboard is unavailable until the node is restored.

**Failure scenario:** A disk failure takes down the single node. Recovery from backup takes 30 minutes. During this time, the analytics dashboard is unavailable. Given the 99.5% availability target (3.6 hours downtime/year), this is within budget if it happens only once.

## Common Mistakes to Avoid

1. **Choosing multi-master for strong consistency workloads** -- conflict resolution cannot guarantee consistency
2. **Using synchronous replication across regions** -- the latency penalty (50-150ms) makes writes too slow
3. **Treating all replicas as equal** -- sync replicas serve consistent reads, async replicas may serve stale data
4. **Not planning for the failure of the replication topology itself** -- what happens when the sync replica dies?

## Key Takeaway

Replication topology selection is driven by the consistency model. Strong consistency forces master-slave with synchronous replication. Eventual consistency enables multi-master with better latency and availability. The trade-off is always between consistency, latency, and complexity. Choose the simplest topology that meets your requirements.
