# Exercise 03: Connection Pool Configuration

**Type:** Independent
**Time:** 30 minutes
**Difficulty:** Medium

## Objective
Design and implement a connection pooling strategy using PgBouncer that handles high-concurrency workloads efficiently.

## Scenario
Your application currently opens a direct database connection per request. Under load, this causes:
- PostgreSQL hitting `max_connections` (default 100)
- High memory usage on the database server
- Connection storms during traffic spikes

You need to implement PgBouncer to solve these problems.

## Tasks

### Part A: Configure PgBouncer
Write a complete `pgbouncer.ini` configuration file for the following requirements:
- Application connects to PgBouncer on port 6432
- PgBouncer connects to PostgreSQL primary on port 5432
- Use transaction pooling mode
- Maximum 200 client connections
- Maximum 50 server connections
- Connection timeout of 30 seconds
- Idle connection timeout of 300 seconds

```ini
; pgbouncer.ini -- complete this configuration
[databases]
; Define database mappings here

[pgbouncer]
; Define pool settings here
```

<details>
<summary>Hint</summary>
In `transaction` mode, server connections are returned to the pool after each transaction completes. This allows more clients than server connections. The `max_client_conn` and `default_pool_size` are the key settings.
</details>

### Part B: Calculate Pool Sizing
Given the following application characteristics, calculate the optimal pool size:

| Parameter | Value |
|-----------|-------|
| Average query time | 5ms |
| Peak concurrent users | 5,000 |
| Average requests per second | 2,000 |
| Queries per request | 3 |
| PostgreSQL max_connections | 200 |

Show your calculation for:
1. Queries per second at peak
2. Optimal server connection pool size
3. Whether the current `max_connections` is sufficient

<details>
<summary>Hint</summary>
Use Little's Law: L = lambda * W. The number of connections needed = QPS * average query time (in seconds). With connection pooling, you multiply by the number of queries per request.
</details>

### Part C: Handle Pool Exhaustion
Write the PgBouncer configuration and application-level logic to handle pool exhaustion gracefully. Address:
1. What happens when all server connections are busy
2. How to configure `server_idle_timeout` to reclaim connections
3. How the application should handle connection wait timeouts

<details>
<summary>Hint</summary>
PgBouncer queues clients when pool is exhausted up to `client_idle_timeout`. Set `query_timeout` to kill long-running queries. The application should implement retry logic with exponential backoff for connection failures.
</details>

## Success Criteria
- [ ] PgBouncer configuration is complete and valid
- [ ] Pool sizing calculations are correct with shown work
- [ ] Transaction pooling mode is correctly configured
- [ ] Pool exhaustion handling covers both PgBouncer and application sides
- [ ] Configuration includes both timeout and queueing parameters

## What You Should Understand After This Exercise
Connection pooling decouples client connections from server connections, allowing the database to serve far more clients than its connection limit allows. Transaction pooling mode is the most efficient for most web applications. Proper pool sizing requires understanding your workload characteristics and applying queuing theory.
