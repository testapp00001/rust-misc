# Exercise 02: Master-Slave Setup

**Type:** Guided | **Time:** 30 min | **Difficulty:** Easy-Medium

## Objective
Configure PostgreSQL streaming replication from a primary to a hot standby, set up monitoring, and implement automatic failover.

## Scenario
Your team runs a PostgreSQL 15 primary that is approaching its connection and CPU limits. You need to set up a streaming replication hot standby to offload read traffic and provide failover capability.

## Tasks

### Part A: Configure the Primary for Replication
Write the PostgreSQL configuration changes needed on the primary to enable streaming replication. Include:
1. WAL level settings
2. Replication connection limits
3. Replication slot configuration
4. A replication user with minimal privileges

<details>
<summary>Hint</summary>
Set `wal_level = replica` (or `logical` for more flexibility). Use `max_wal_senders` to limit replication connections. Replication slots prevent the primary from removing WAL segments before replicas consume them.
</details>

### Part B: Initialize the Replica
Write the step-by-step procedure to create a hot standby from the primary. Include:
1. Taking a base backup
2. Configuring the standby to connect to the primary
3. Starting the standby in recovery mode

<details>
<summary>Hint</summary>
Use `pg_basebackup` with the `-R` flag, which automatically creates `standby.signal` and sets `primary_conninfo` in `postgresql.auto.conf`. The standby connects to the primary using the replication user created in Part A.
</details>

### Part C: Set Up Replication Monitoring
Write SQL queries to monitor replication health:
1. Check replication state and lag on the primary
2. Check WAL receiver status on the replica
3. Alert when lag exceeds 10 seconds

<details>
<summary>Hint</summary>
On the primary, query `pg_stat_replication` for `state`, `sent_lsn`, `replay_lsn`, and calculate byte lag. On the replica, query `pg_stat_wal_receiver`. Use `pg_last_wal_receive_lsn()` and `pg_last_wal_replay_lsn()` to calculate replay lag.
</details>

### Part D: Configure Automatic Failover with Patroni
Write a Patroni configuration that:
1. Uses etcd as the distributed consensus store
2. Configures the primary and replica as Patroni members
3. Sets automatic failover with a 30-second timeout
4. Configures synchronous replication to prevent data loss

<details>
<summary>Hint</summary>
Patroni YAML configuration includes `postgresql` (database settings), `etcd` (consensus store), `bootstrap` (initialization), and `ttl`/`retry_timeout` (failover timing). Synchronous mode uses `synchronous_mode: true` in the `postgresql` section.
</details>

## Success Criteria

- [ ] Primary configuration enables streaming replication
- [ ] Replica successfully connects and receives WAL
- [ ] Monitoring queries return accurate lag metrics
- [ ] Patroni configuration includes automatic failover and sync replication

## What You Should Understand After This Exercise

Master-slave replication in PostgreSQL is built on WAL shipping. The primary generates WAL records, and the replica receives and replays them. Replication slots prevent WAL accumulation. Patroni adds automatic failover on top of streaming replication by using distributed consensus to elect a new primary when the old one fails.
