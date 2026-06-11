# Exercise 03: Active-Active Configuration

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective

Design and configure an active-active PostgreSQL setup where two instances in different regions serve read-write traffic simultaneously, handling the inherent challenge of write conflicts in multi-writer architectures.

## Scenario

Your company has expanded to serve customers in both the US and EU. Latency requirements mandate that each region's users hit a local database. Both regions must accept writes. The application is a multi-tenant SaaS platform where each customer (tenant) has a `tenant_id` that determines their home region.

You have:
- **us-east** (10.1.0.10): PostgreSQL instance serving US tenants
- **eu-west** (10.2.0.10): PostgreSQL instance serving EU tenants
- A shared `orders` table with columns: `order_id`, `tenant_id`, `amount`, `status`, `updated_at`, `origin_region`

## Tasks

### Part A: Configure Bi-Directional Logical Replication

Set up PostgreSQL logical replication so that writes on each node are replicated to the other.

```yaml
# Write the PostgreSQL publication configuration for us-east
# This node publishes changes from the 'orders' table
```

```yaml
# Write the PostgreSQL subscription configuration for eu-west
# This node subscribes to changes from us-east
# Include: connection string, publication name, copy_data setting
```

Write the SQL commands to set up the publication on us-east and the subscription on eu-west:

```sql
-- On us-east: Create publication for orders table
-- On eu-west: Create subscription pointing to us-east
```

Then write the reverse direction (eu-west publishes, us-east subscribes).

### Part B: Design a Conflict-Avoidance Partitioning Strategy

Logical replication creates a conflict risk: if the same row is written on both nodes simultaneously, one write will overwrite the other. Design a partitioning strategy that avoids this.

Fill in the design table:

| Aspect | Design Decision | Rationale |
|--------|-----------------|-----------|
| Data ownership model | | |
| Tenant routing rule | | |
| What happens if a US tenant is accessed from EU? | | |
| How do you handle tenant migration between regions? | | |
| What about shared/global data (no tenant_id)? | | |

### Part C: Handle Conflict Resolution

Even with partitioning, edge cases will cause conflicts. Implement a conflict resolution strategy.

Write the SQL to add a `conflict_resolution` column and a trigger that implements "last writer wins" using `updated_at`:

```sql
-- Write a BEFORE UPDATE trigger on the orders table
-- that compares the incoming updated_at with the existing row
-- and only applies the update if the incoming value is newer
```

Write a function that detects and logs conflicts:

```sql
-- Write a function called log_replication_conflict()
-- that records conflict details into a conflicts_audit table
-- Include: table_name, conflict_type, local_value, remote_value, resolution, timestamp
```

### Part D: Implement Application-Level Routing

Write the application-layer routing logic that directs writes to the correct regional database based on tenant ownership.

```python
# Write a Python class called RegionRouter that:
# 1. Maintains a mapping of tenant_id -> home_region
# 2. Routes write queries to the tenant's home region
# 3. Routes read queries to the nearest region (local or remote)
# 4. Falls back to the other region if the home region is down
# 5. Handles the case where a tenant's region is unknown
```

```python
# Write the connection pool configuration for both regions
# Include: host, port, database, pool_size, health_check_interval
```

## Success Criteria

- [ ] Logical replication is configured in both directions (verified with `pg_stat_subscription`)
- [ ] The partitioning strategy assigns tenants to regions based on a clear, documented rule
- [ ] The conflict resolution trigger prevents stale writes from overwriting newer data
- [ ] The application router correctly directs writes to the tenant's home region
- [ ] A simulated regional failure causes the router to fail over to the surviving region

## Hints

<details>
<summary>Hint 1: Logical replication gotcha</summary>

By default, logical replication subscriptions set `origin = NONE`, which means changes received from a subscription are NOT forwarded to other subscriptions. This prevents infinite replication loops. However, you must ensure that both sides have `origin = NONE` configured. Check with:

```sql
SELECT subname, suborigin FROM pg_subscription;
```

</details>

<details>
<summary>Hint 2: Conflict detection with updated_at</summary>

The trigger approach for last-writer-wins requires a reliable timestamp. Use a hybrid approach:

```sql
-- Use a combination of timestamp and region identifier
-- to break ties when timestamps are identical
-- (updated_at, origin_region) as a composite comparison key
```

Consider using `pg_logical_emit_message()` to embed metadata in the replication stream.

</details>

<details>
<summary>Hint 3: Read routing optimization</summary>

For reads, you can serve from the local region for strong consistency or from either region for eventual consistency. Document the tradeoff:

- **Strong consistency:** Always read from the tenant's home region (higher latency for cross-region users)
- **Eventual consistency:** Read from the nearest region (may return slightly stale data)

The choice depends on whether the application can tolerate reading data that is a few hundred milliseconds behind.

</details>

## What You Should Understand After This Exercise

Active-active is not just "two active-passive clusters." The fundamental challenge is that writes can happen anywhere, and replication has latency. Conflict resolution is unavoidable -- you either prevent conflicts through careful data partitioning (preferred) or detect and resolve them after the fact (necessary as a safety net). The application layer must be aware of the data ownership model, or all the database-level work is wasted.
