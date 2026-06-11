# Exercise 05: Storage Architecture for a Production Platform -- Putting It All Together

**Type:** Integration
**Time:** 60 minutes
**Difficulty:** Hard

## Objective

Design a complete storage strategy for a production platform that includes a
PostgreSQL database, a Redis cache, shared application configuration, and
user-uploaded files. Apply PV, PVC, StorageClass, access modes, and reclaim
policies to meet real-world requirements.

---

## Background

You are the platform engineer for an e-commerce company. The application team
has submitted a storage requirements document:

| Component | Data Type | Size | Access Pattern | Durability | Growth |
|-----------|-----------|------|----------------|------------|--------|
| PostgreSQL (primary) | Transactional DB | 200Gi | Single writer | Critical | 20Gi/month |
| PostgreSQL (replica) | Read replica | 200Gi | Single reader | Critical | 20Gi/month |
| Redis | Cache + sessions | 10Gi | Single writer | Important | Slow |
| App config | Feature flags, endpoints | 1Gi | Many readers | Moderate | None |
| User uploads | Product images | 500Gi | Many readers, few writers | Critical | 50Gi/month |
| Search index | Elasticsearch | 100Gi | Single writer | Recoverable | 10Gi/month |

Additional constraints:

- The cluster runs on AWS EKS across 3 availability zones.
- PostgreSQL must survive PVC deletion (retention policy).
- User uploads must be accessible from multiple pods simultaneously.
- App config should be read-only for most pods.
- The search index can be rebuilt from the database, so durability is less critical.
- Storage costs should be optimized (not everything needs premium SSD).

---

## Tasks

### Task 1: Design StorageClasses

Create StorageClasses for each storage tier. For each class, specify:

- Name
- Provisioner and parameters
- Reclaim policy
- Volume binding mode
- Whether volume expansion is allowed

| Tier | Use Case | Disk Type | Reclaim | Binding |
|------|----------|-----------|---------|---------|
| `premium` | PostgreSQL | io2 | Retain | WaitForFirstConsumer |
| `standard` | Redis, Search | gp3 | Delete | WaitForFirstConsumer |
| `shared-fs` | User uploads | EFS (NFS) | Retain | Immediate |
| `config-ro` | App config | EFS (NFS) | Retain | Immediate |

Write all four StorageClass YAML manifests.

<details>
<summary>Hint 1: EFS StorageClass</summary>

AWS EFS uses a different provisioner than EBS:

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: shared-fs
provisioner: efs.csi.aws.com
parameters:
  provisioningMode: efs-ap
  fileSystemId: fs-0123456789abcdef0
  directoryPerms: "700"
reclaimPolicy: Retain
volumeBindingMode: Immediate
```

EFS supports ReadWriteMany (RWX) -- multiple pods on multiple nodes can read
and write simultaneously.

</details>

### Task 2: Design PVCs and Access Modes

Create PVCs for each component. Choose the correct access mode for each:

| Component | PVC Name | StorageClass | Size | Access Mode |
|-----------|----------|-------------|------|-------------|
| PostgreSQL primary | `pg-primary-data` | `premium` | 200Gi | ? |
| PostgreSQL replica | `pg-replica-data` | `premium` | 200Gi | ? |
| Redis | `redis-data` | `standard` | 10Gi | ? |
| App config | `app-config` | `config-ro` | 1Gi | ? |
| User uploads | `user-uploads` | `shared-fs` | 500Gi | ? |
| Search index | `search-data` | `standard` | 100Gi | ? |

Fill in the access mode column and explain your choice for each.

<details>
<summary>Hint 2: Access mode selection rules</summary>

- **Database primary:** One writer on one node -> `ReadWriteOnce`
- **Database replica:** One reader on one node -> `ReadWriteOnce`
- **Redis:** One instance owns the data -> `ReadWriteOnce`
- **App config:** Many pods read, one admin writes -> `ReadOnlyMany`
  (or `ReadWriteOnce` with a deployment that writes, then many pods that read)
- **User uploads:** Multiple pods serve images, multiple pods accept uploads
  -> `ReadWriteMany`
- **Search index:** One Elasticsearch node owns its shard -> `ReadWriteOnce`

</details>

### Task 3: Pod Manifests

Write the Pod (or Deployment) manifests for each component that mount the
correct PVCs. For PostgreSQL, use a simplified single-container Pod. For user
uploads, use a Deployment with 3 replicas.

Requirements:
- PostgreSQL primary mounts `pg-primary-data` at `/var/lib/postgresql/data`.
- PostgreSQL replica mounts `pg-replica-data` at `/var/lib/postgresql/data`.
- Redis mounts `redis-data` at `/data`.
- App config mounts `app-config` at `/etc/app-config` as read-only.
- User uploads Deployment (3 replicas) mounts `user-uploads` at `/uploads`.
- Elasticsearch mounts `search-data` at `/usr/share/elasticsearch/data`.

<details>
<summary>Hint 3: ReadOnly volume mount</summary>

You can mount a volume as read-only inside the container even if the PVC is RWO:

```yaml
volumeMounts:
- mountPath: /etc/app-config
  name: config
  readOnly: true
```

This is a container-level restriction, not a storage-level restriction. Other
containers (with read-write access) can still write to the volume.

</details>

### Task 4: Reclaim Policy and Lifecycle Planning

For each component, answer:

1. If the PVC is accidentally deleted, what happens to the data?
2. How would you recover?
3. What is the RPO (Recovery Point Objective) for each component?
4. Should you enable volume expansion? Why?

Fill in the table:

| Component | Reclaim Policy | PVC Deleted -> Data? | Recovery Method | RPO | Expansion? |
|-----------|---------------|---------------------|-----------------|-----|------------|
| PostgreSQL primary | Retain | ? | ? | ? | ? |
| PostgreSQL replica | Retain | ? | ? | ? | ? |
| Redis | Delete | ? | ? | ? | ? |
| App config | Retain | ? | ? | ? | ? |
| User uploads | Retain | ? | ? | ? | ? |
| Search index | Delete | ? | ? | ? | ? |

<details>
<summary>Hint 4: RPO thinking</summary>

RPO (Recovery Point Objective) is the maximum acceptable data loss measured in
time. If RPO is 0, you cannot lose any data. If RPO is 1 hour, you can lose up
to 1 hour of data.

- **PostgreSQL:** RPO ~0 (use WAL archiving and replication for point-in-time
  recovery).
- **Redis:** RPO depends on persistence config (RDB snapshots every N minutes, or
  AOF with fsync every second).
- **User uploads:** RPO ~0 (product images are revenue-critical).
- **Search index:** RPO = time to re-index from database (hours).

</details>

### Task 5: Cost Optimization

The finance team wants to reduce storage costs. Current monthly costs:

| StorageClass | Cost per GiB/month | Total GiB | Monthly Cost |
|-------------|-------------------|-----------|-------------|
| premium (io2) | $0.125 + IOPS charges | 400Gi | ~$80 |
| standard (gp3) | $0.08 | 110Gi | ~$9 |
| shared-fs (EFS) | $0.30 | 500Gi | ~$150 |
| config-ro (EFS) | $0.30 | 1Gi | ~$0.30 |

Total: ~$239/month

Propose cost optimizations that do not compromise the requirements. Consider:

1. Can any component use a cheaper storage tier?
2. Can the search index use gp3 instead of premium?
3. Is EFS the right choice for user uploads, or is there a cheaper alternative?
4. Can you reduce the PostgreSQL replica's storage class?

Write a brief proposal with estimated savings.

<details>
<summary>Hint 5: Storage cost levers</summary>

- **Disk type:** gp3 is 60% cheaper than io2 for most workloads. Use io2 only
  where IOPS guarantees are critical.
- **EFS vs S3:** EFS is expensive for large files that are read-heavy but
  write-light. S3 + a CDN is cheaper for static assets (images). EFS is better
  for POSIX-compliant shared filesystems.
- **Replica storage tier:** Read replicas often do not need premium IOPS. If the
  replica is read-only and latency-tolerant, gp3 may be sufficient.
- **Lifecycle policies:** Move old data to cheaper tiers (S3 Infrequent Access,
  EFS Infrequent Access).

</details>

---

## Success Criteria

- [ ] Four StorageClasses with appropriate parameters for each tier.
- [ ] Six PVCs with correct access modes for each workload.
- [ ] Pod/Deployment manifests that mount PVCs correctly.
- [ ] A reclaim policy and lifecycle plan for each component.
- [ ] A cost optimization proposal with concrete savings.

---

## Hints

<details>
<summary>Hint 6: StatefulSet consideration</summary>

For PostgreSQL, a StatefulSet (Module 33) is better than a bare Pod because it
provides:
- Stable pod names (postgres-0, postgres-1).
- Ordered startup/shutdown.
- Per-pod PVCs via `volumeClaimTemplates`.

In this exercise, we use bare Pods for simplicity, but in production you would
use StatefulSets.

</details>

<details>
<summary>Hint 7: Backup strategy</summary>

Storage durability (Retain policy) is not a backup strategy. A Retain policy
protects against accidental PVC deletion, but not against:
- Application-level data corruption.
- Accidental `DELETE` SQL statements.
- Ransomware or malicious access.

For production databases, implement:
- Automated daily backups (pg_dump or volume snapshots).
- Point-in-time recovery using WAL archiving.
- Cross-region backup replication.

</details>
