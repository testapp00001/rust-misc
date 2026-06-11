# Exercise 03: Automated Failover Implementation

**Type:** Independent | **Time:** 30 min | **Difficulty:** Medium

## Objective

Implement automated PostgreSQL failover using Patroni with a distributed consensus layer, health check scripts, and alerting configuration.

## Scenario

You work for DataForge, a fintech company running PostgreSQL 15 for their transaction processing system. The current setup is manual failover with a runbook, but the team has decided to automate failover to meet their 30-second RTO. You are tasked with implementing Patroni-based automatic failover.

Current infrastructure:
- 3 PostgreSQL nodes across 3 availability zones in us-east-1
- Existing etcd cluster (3 nodes) already deployed for service discovery
- Prometheus and Alertmanager for monitoring
- Kubernetes cluster available but PostgreSQL runs on EC2 (not in K8s)

Target:
- Automatic failover within 30 seconds of primary failure detection
- No split-brain under any failure scenario
- Alerting on all failover events
- Health checks that distinguish between "node is slow" and "node is dead"

## Tasks

### Part A: Patroni Configuration

Write a complete Patroni configuration file for a 3-node PostgreSQL cluster.

1. Write the Patroni YAML configuration for the primary node (node-1).
2. Configure synchronous replication to at least one replica.
3. Configure the bootstrap section for initial cluster creation.
4. Set appropriate timeouts: TTL (30s), retry_timeout (10s), maximum_lag_on_failover.
5. Configure PostgreSQL parameters that Patroni should manage (wal_level, max_wal_senders, etc.).

<details>
<summary>Hint</summary>
Patroni configuration has top-level sections: scope, name, restapi, etcd, bootstrap, postgresql, and watchdog. The bootstrap.databases section creates databases on first initialization.
</details>

### Part B: Distributed Consensus Layer Setup

Write the etcd configuration and Patroni integration for distributed consensus.

1. Write the etcd configuration for a 3-node etcd cluster.
2. Configure Patroni to use etcd for leader election.
3. Write a script to verify etcd cluster health.
4. Explain what happens in Patroni when etcd becomes unavailable.

<details>
<summary>Hint</summary>
Patroni uses etcd for two purposes: storing the cluster state and leader election. If etcd is down, Patroni will demote the primary to read-only to prevent split-brain.
</details>

### Part C: Health Check Scripts

Write health check scripts that Patroni and external monitoring can use.

1. Write a script that checks if Patroni is running and the node is healthy.
2. Write a script that checks PostgreSQL replication lag.
3. Write a script that detects split-brain conditions (two nodes both thinking they are primary).
4. Write a systemd unit file or cron configuration to run health checks periodically.

<details>
<summary>Hint</summary>
Patroni exposes a REST API on port 8008. Use /health for basic checks, /replica for replica checks, and /primary for primary checks. The patronictl list command shows cluster state.
</details>

### Part D: Alerting Configuration

Configure alerting for failover events and degraded states.

1. Write Prometheus alert rules for: failover occurred, replication lag exceeded threshold, Patroni node down, etcd cluster unhealthy.
2. Write Alertmanager routing rules to send failover alerts to PagerDuty and degraded-state alerts to Slack.
3. Write a webhook receiver that logs failover events to an audit trail.

<details>
<summary>Hint</summary>
Patroni exports metrics to Prometheus via the patroni_exporter. Key metrics include patroni_master, patroni_replication_lag, and patroni_postgres_running.
</details>

## Success Criteria

- [ ] Patroni configuration is complete with all required sections.
- [ ] Synchronous replication is configured with at least one sync replica.
- [ ] etcd integration is configured with proper endpoints and authentication.
- [ ] Health check script covers liveness, replication lag, and split-brain detection.
- [ ] Prometheus alert rules cover 4 failure scenarios.
- [ ] Alertmanager routes failover alerts to PagerDuty.
- [ ] All configuration files are syntactically valid YAML.

## What You Should Understand

- Patroni automates failover by combining leader election (via etcd/Consul/ZooKeeper) with PostgreSQL streaming replication.
- The distributed consensus layer is the safety net: if it fails, Patroni errs on the side of caution (demotes primary).
- Health checks must distinguish between "slow" and "dead" to avoid false failovers.
- Automated failover reduces RTO but introduces the risk of unnecessary failovers if health checks are too aggressive.
