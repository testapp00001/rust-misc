# Exercise 02: Read Replica Setup with ProxySQL

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective
Configure PostgreSQL read replicas and set up ProxySQL to route read and write queries automatically.

## Scenario
Your team has a PostgreSQL primary database that is handling too much traffic. You need to set up a read replica and configure ProxySQL to route read queries to the replica while keeping write queries on the primary.

## Tasks

### Part A: Set Up the Primary-Replica Topology
Using Docker Compose, create a PostgreSQL primary and one read replica.

Create a file called `docker-compose.yml` with:
- A PostgreSQL primary instance (port 5432)
- A PostgreSQL replica instance (port 5433)
- Proper replication configuration

<details>
<summary>Hint</summary>
Use the official postgres Docker image. The primary needs `wal_level=replica` in its configuration. The replica uses `pg_basebackup` to initialize from the primary and `primary_conninfo` in `postgresql.conf` or `standby.signal` to connect.
</details>

### Part B: Configure ProxySQL for Read/Write Splitting
Write a ProxySQL configuration that:
1. Defines the primary as a writer (hostgroup 10)
2. Defines the replica as a reader (hostgroup 20)
3. Routes SELECT queries to the reader hostgroup
4. Routes INSERT, UPDATE, DELETE queries to the writer hostgroup

```yaml
# proxysql.cnf -- complete this configuration
datadir="/var/lib/proxysql"

mysql_servers:
(
    # Add server definitions here
)

mysql_query_rules:
(
    # Add routing rules here
)

mysql_users:
(
    # Add user definitions here
)
```

<details>
<summary>Hint</summary>
ProxySQL uses `match_digest` or `match_pattern` in query rules. SELECT queries match the pattern `^SELECT .*`. The `destination_hostgroup` determines where the query is sent.
</details>

### Part C: Verify the Setup
Write SQL statements to verify that:
1. The replica is receiving and applying WAL from the primary
2. Read queries are being routed to the replica
3. Write queries are being routed to the primary

<details>
<summary>Hint</summary>
On the replica, check `pg_stat_wal_receiver` for replication status. In ProxySQL, query `stats_mysql_query_digest` to see which hostgroup received each query.
</details>

## Success Criteria
- [ ] Docker Compose file correctly defines primary and replica
- [ ] Replication is active and lag is measurable
- [ ] ProxySQL routes SELECT to replica and writes to primary
- [ ] Verification queries confirm correct routing

## What You Should Understand After This Exercise
Setting up read replicas requires configuring WAL on the primary, initializing the replica, and establishing the replication connection. ProxySQL automates read/write splitting at the proxy layer, removing this logic from the application. However, you must handle replication lag for read-after-write consistency.
