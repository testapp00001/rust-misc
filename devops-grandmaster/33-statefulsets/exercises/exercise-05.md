# Exercise 05: Design a Database HA Strategy with StatefulSets

## Objective

Design and implement a complete high-availability database architecture using StatefulSets. This integration exercise combines everything from previous exercises: StatefulSets, headless Services, persistent storage, update strategies, and scaling. You will design a production-ready PostgreSQL HA setup and document your architectural decisions.

## Background

Your team runs a critical application that requires a PostgreSQL database with the following requirements:
- Maximum 30 seconds of downtime during maintenance
- No data loss on failover (RPO = 0)
- Automatic failover when the primary fails
- Read scaling for reporting queries
- Safe rolling updates without service interruption
- Backup strategy integrated with the StatefulSet

You need to design the complete Kubernetes architecture.

## Instructions

### Part 1: Architecture Design Document

Write an architecture document that answers the following questions. This is a design exercise - focus on the reasoning, not just the YAML.

**1. StatefulSet Configuration**

- How many replicas? Why?
- What is the `serviceName` and why must it match the headless Service?
- What storage class and size would you choose?
- What resource requests and limits are appropriate for a database?

**2. Networking**

- How will the primary be identified? (Hint: always ordinal 0)
- How will replicas connect to the primary for streaming replication?
- What Services do you need? (Headless for StatefulSet DNS, ClusterIP for reads, separate Service for writes)
- How will clients discover the primary vs replicas?

**3. Update Strategy**

- What `updateStrategy` should you use?
- Should you update replicas or the primary first? Why?
- How would you perform a canary update?
- What is the rollback procedure?

**4. Failure Handling**

- What happens when the primary pod (postgres-0) fails?
- What liveness and readiness probes should you configure?
- What are the resource requests/limits trade-offs?
- How do PVCs behave during failover?

**5. Backup Strategy**

- How will you back up the database without downtime?
- Where will backups be stored?
- How do backups interact with the StatefulSet lifecycle?

### Part 2: Implementation

Implement the core components of your design:

1. A headless Service for StatefulSet DNS
2. A ClusterIP Service for write traffic (pointing to postgres-0)
3. A ClusterIP Service for read traffic (pointing to all replicas)
4. The StatefulSet with proper probes, resources, and update strategy
5. A ConfigMap or init script that configures replication

### Part 3: Operational Scenarios

Document the exact commands and procedures for these scenarios:

1. **Scaling from 3 to 5 replicas**: What happens to the new pods? How do they join the replication topology?

2. **Performing a major version upgrade** (PostgreSQL 16 to 17): What is the safe procedure using partitions?

3. **Handling a primary failure**: What manual or automatic steps are needed?

4. **Restoring from backup**: How do you restore a specific pod's data?

## Deliverables

1. `architecture.md` - Your design document with answers to Part 1
2. `headless-service.yaml` - The headless Service
3. `write-service.yaml` - Service for write traffic
4. `read-service.yaml` - Service for read traffic
5. `postgres-statefulset.yaml` - The StatefulSet
6. `replication-init.yaml` - ConfigMap or scripts for replication setup
7. `operational-runbook.md` - Procedures for Part 3 scenarios

## Success Criteria

- [ ] Architecture document explains WHY for each design decision
- [ ] StatefulSet has 3 replicas with proper probes and resource limits
- [ ] Headless Service provides stable DNS for all pods
- [ ] Write Service directs traffic to postgres-0
- [ ] Read Service load-balances across all pods
- [ ] Update strategy uses partitions for safe rollouts
- [ ] Operational runbook covers all 4 scenarios with exact commands

## Hints

<details>
<summary>Hint 1: Primary Identification</summary>

In a StatefulSet, the primary is always the pod with the lowest ordinal (postgres-0). This is a convention, not a Kubernetes feature. Your application or init scripts must enforce this.

A common pattern:
```bash
# In an init container or entrypoint script
if [ "$POD_NAME" = "db-0" ]; then
  # Configure as primary
  echo "primary_conninfo = ''" >> /var/lib/postgresql/data/postgresql.conf
else
  # Configure as replica
  echo "primary_conninfo = 'host=db-0.db.postgres.svc.cluster.local'" >> /var/lib/postgresql/data/postgresql.conf
fi
```

The DNS name `db-0.db.postgres.svc.cluster.local` is stable because of the StatefulSet + headless Service combination.
</details>

<details>
<summary>Hint 2: Multiple Services for Different Traffic</summary>

```yaml
# Write Service - targets only the primary
apiVersion: v1
kind: Service
metadata:
  name: postgres-write
spec:
  selector:
    app: postgres
    role: primary    # Label that identifies the primary
  ports:
  - port: 5432

---
# Read Service - targets all pods
apiVersion: v1
kind: Service
metadata:
  name: postgres-read
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
```

Note: The write Service requires the primary to have a specific label. Since StatefulSet pod templates are identical, you may need an external tool or operator to manage the `role` label.
</details>

<details>
<summary>Hint 3: Probe Configuration for Databases</summary>

```yaml
readinessProbe:
  exec:
    command:
    - pg_isready
    - -U
    - postgres
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3

livenessProbe:
  exec:
    command:
    - pg_isready
    - -U
    - postgres
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 6
```

Key differences:
- Liveness probe has a longer `initialDelaySeconds` (PostgreSQL takes time to start)
- Readiness probe runs more frequently (to detect unavailability quickly)
- Liveness probe has a higher `failureThreshold` (to avoid killing pods during temporary load spikes)
</details>

<details>
<summary>Hint 4: Safe Update Procedure</summary>

```
Step 1: Set partition to 2 (only update postgres-2)
Step 2: Update image
Step 3: Wait for postgres-2 to be Ready
Step 4: Verify postgres-2 is replicating from postgres-0
Step 5: Set partition to 1 (update postgres-1)
Step 6: Wait for postgres-1 to be Ready
Step 7: Verify postgres-1 is replicating
Step 8: During maintenance window: set partition to 0 (update postgres-0)
Step 9: Verify all pods are on new version
```

The key insight: update replicas first, primary last. The primary is the most critical node.
</details>

<details>
<summary>Hint 5: Backup Without Downtime</summary>

Use `pg_basebackup` from a replica (not the primary) to avoid impacting write performance:

```bash
# Run a backup from a replica
kubectl exec postgres-1 -n postgres -- \
  pg_basebackup -h localhost -U postgres -D /backups/base -Ft -z -P
```

Or use a CronJob that targets a replica pod by its stable DNS name:

```yaml
env:
- name: PGHOST
  value: postgres-1.postgres.postgres.svc.cluster.local
```

The stable DNS name from the StatefulSet makes this possible.
</details>

## Concepts to Review

After completing this exercise, you should understand:

- How to design a complete HA database architecture on Kubernetes
- The role of headless Services in StatefulSet networking
- How to use partitioned updates for safe database upgrades
- How PVCs interact with StatefulSet lifecycle (scale up/down, delete, recreate)
- How to plan operational procedures (failover, backup, restore) around StatefulSet behavior
