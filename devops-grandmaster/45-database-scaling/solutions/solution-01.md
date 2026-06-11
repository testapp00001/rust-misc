# Solution 01: Scaling Strategy Identification

## Part A: Workload Classification

| Component | Read:Write | Classification | Replicas Help? | Pooling Helps? |
|-----------|-----------|----------------|----------------|----------------|
| Product catalog | 95:5 | Read-heavy | Yes -- 95% reads can be distributed across replicas | Yes -- high QPS benefits from connection reuse |
| Shopping cart | 60:40 | Balanced-leaning-read | Partially -- 60% reads can go to replicas, but 40% writes mean the primary is still busy | Yes -- session-heavy workload benefits from pooling |
| Order processing | 20:80 | Write-heavy | Minimal -- most queries are writes that must go to primary | Yes -- transaction-heavy workload benefits from connection reuse |
| Analytics dashboard | 99:1 | Read-heavy | Yes -- nearly all reads, ideal for replicas | Yes -- long-running queries benefit from dedicated pool |
| User sessions | 70:30 | Read-heavy | Yes -- but session data needs consistency guarantees | Yes -- high connection churn from many users |

## Part B: Scaling Strategy

| Component | Replicas | Pool Size | Routing Rules |
|-----------|----------|-----------|---------------|
| Product catalog | 3 | 50 server connections | All reads to replicas, writes to primary. Use read-after-write consistency for recently updated products. |
| Shopping cart | 1 | 30 server connections | Route reads to replica only for catalog data. Cart state reads go to primary for consistency. |
| Order processing | 0 | 20 server connections | All queries to primary. Orders are write-heavy and require immediate consistency. |
| Analytics dashboard | 2 | 10 server connections | All queries to replicas. Acceptable to have slightly stale data (up to 5 seconds lag). |
| User sessions | 2 | 40 server connections | Session reads to replicas with 1-second staleness tolerance. Session writes to primary. |

### Rationale
- Product catalog gets the most replicas because it has the highest read QPS and can tolerate some staleness.
- Shopping cart has session-affinity requirements -- cart data must be consistent, so writes go to primary.
- Order processing is write-heavy with no read benefit from replicas.
- Analytics can tolerate the most staleness and has long-running queries, so dedicated replicas prevent resource contention.
- User sessions have high QPS but need some consistency guarantees.

## Part C: Anti-Pattern Identification

**1. "Add 10 read replicas to handle all our traffic"**

This is wrong because:
- Each replica adds replication lag -- with 10 replicas, some may be several seconds behind
- Not all queries are reads -- writes still go to the primary
- More replicas means more WAL shipping overhead on the primary
- Maintenance complexity increases (monitoring, failover, schema changes)
- Diminishing returns -- the primary's WAL generation rate becomes the bottleneck

**2. "Set connection pool size to 1000 for maximum throughput"**

This is wrong because:
- PostgreSQL performance degrades with too many active connections (context switching, lock contention)
- Each connection consumes ~10MB of memory -- 1000 connections = 10GB just for connections
- Optimal pool size is typically 2-4x the number of CPU cores
- Large pools cause thundering herd problems when connections wake up simultaneously

**3. "Route all queries to replicas to reduce load on the primary"**

This is wrong because:
- Write queries (INSERT, UPDATE, DELETE) MUST go to the primary
- Replicas are read-only by definition
- Routing writes to a replica will fail with an error
- Read-after-write scenarios need to read from the primary to avoid stale data

**4. "We do not need connection pooling because PostgreSQL handles connections fine"**

This is wrong because:
- PostgreSQL creates a new process per connection (fork model) -- this is expensive
- Each connection consumes memory for shared buffers, work memory, and stack
- Connection establishment overhead is ~5-10ms per connection
- Without pooling, traffic spikes cause connection storms that crash the database
- PgBouncer in transaction mode can serve 10,000+ clients with only 50 server connections

## Common Mistakes to Avoid
- Treating all workloads the same -- different components need different strategies
- Ignoring replication lag when routing reads to replicas
- Setting pool sizes based on client count rather than server capacity
- Forgetting that connection pooling does not help with query performance -- only connection overhead

## Key Takeaway
Database scaling requires understanding your workload patterns at a granular level. A one-size-fits-all approach wastes resources or creates bottlenecks. The right strategy combines read replicas for read-heavy workloads, connection pooling for connection efficiency, and query optimization for per-query performance.
