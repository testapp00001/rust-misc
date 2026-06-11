# Solution 05: Design a Database HA Strategy with StatefulSets

## Architecture Design Document

### 1. StatefulSet Configuration

**Replicas: 3**

Rationale:
- 1 primary (postgres-0) handles all write traffic
- 2 replicas handle read traffic and provide failover capability
- 3 is the minimum for HA: if one node fails, you still have a primary and a replica
- More replicas increase read capacity but also increase replication lag and resource costs

**Storage: 50Gi, gp3 StorageClass (or equivalent)**

Rationale:
- Database storage needs grow over time; 50Gi provides headroom
- `allowVolumeExpansion: true` on the StorageClass allows online resizing
- SSD-backed storage (gp3, pd-ssd) provides consistent IOPS for database workloads
- `ReadWriteOnce` access mode is correct since each pod has its own PVC

**Resources:**
```
requests: 500m CPU, 1Gi memory
limits:   2 CPU, 4Gi memory
```

Rationale:
- PostgreSQL uses memory heavily for `shared_buffers`, `work_mem`, and OS page cache
- CPU limits should be generous for query processing
- Requests should be set to ensure the pod is not evicted under load

### 2. Networking

**Primary Identification: postgres-0**

The primary is always the pod with ordinal 0. This is a convention enforced by the application's init script, not by Kubernetes itself. The init container checks the pod name and configures the node as primary or replica accordingly.

**Replica Connection to Primary:**

Replicas connect to the primary using its stable DNS name:
```
postgres-0.db.database.svc.cluster.local:5432
```

This DNS name is permanent and survives pod restarts. It is provided by the combination of the StatefulSet and the headless Service.

**Services Required:**

| Service | Type | Purpose | Selector |
|---------|------|---------|----------|
| `db` | Headless (clusterIP: None) | StatefulSet DNS | `app: postgres` |
| `db-write` | ClusterIP | Write traffic to primary only | `app: postgres, role: primary` |
| `db-read` | ClusterIP | Read traffic to all pods | `app: postgres` |

The write Service requires a `role: primary` label on postgres-0. Since StatefulSet pod templates are identical, this label must be managed by an init container or operator that promotes the lowest-ordinal pod to primary.

**Client Discovery:**
```
Writes: db-write.database.svc.cluster.local → postgres-0 only
Reads:  db-read.database.svc.cluster.local  → all pods (round-robin)
Direct: postgres-0.db.database.svc.cluster.local → specific pod
```

### 3. Update Strategy

**Strategy: RollingUpdate with Partition**

```yaml
updateStrategy:
  type: RollingUpdate
  rollingUpdate:
    partition: 0
```

**Update Order:** Updates proceed from highest ordinal to lowest (2, 1, 0). This means replicas update before the primary. This is the correct order because:
- Replicas are less critical (they only serve reads)
- If the update introduces a bug, it is caught on a replica first
- The primary updates last, minimizing write disruption

**Canary Update Procedure:**
```
1. Set partition to 2
2. Update image
3. Wait for postgres-2 to be Ready
4. Verify postgres-2 is replicating from postgres-0
5. Monitor for 15 minutes
6. If healthy: set partition to 0
7. If unhealthy: roll back image, set partition to 0
```

**Rollback Procedure:**
```bash
# Check history
kubectl rollout history statefulset/db -n database

# Roll back
kubectl rollout undo statefulset/db -n database

# Or manually set the old image
kubectl set image statefulset/db postgres=postgres:16 -n database
```

### 4. Failure Handling

**Primary Pod (postgres-0) Failure:**

When postgres-0 fails:
1. Kubernetes detects the failure via liveness probe (after `failureThreshold * periodSeconds` = 60 seconds)
2. Kubernetes restarts the pod (same name, same PVC)
3. PostgreSQL performs crash recovery on startup (replays WAL)
4. Replicas detect the disconnect and wait for the primary to return
5. Once postgres-0 is Ready, replicas automatically reconnect

**Important:** Kubernetes StatefulSets do NOT provide automatic primary failover. If postgres-0 is down, writes are unavailable until it recovers. For automatic failover, you need:
- A PostgreSQL operator (CloudNativePG, Zalando, CrunchyData)
- Patroni for leader election
- A custom failover controller

**Probes:**

```yaml
readinessProbe:
  exec:
    command: ["pg_isready", "-U", "postgres"]
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3

livenessProbe:
  exec:
    command: ["pg_isready", "-U", "postgres"]
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 6
```

Rationale:
- Readiness probe is frequent (5s) to quickly remove unhealthy pods from Service endpoints
- Liveness probe has a long initial delay (30s) because PostgreSQL startup can be slow
- Liveness probe has a high failure threshold (6) to avoid killing pods during temporary load spikes
- `pg_isready` checks if PostgreSQL can accept connections

**PVC Behavior During Failover:**
- PVC is NOT deleted when the pod is deleted
- When the pod is recreated, the same PVC is re-attached
- Data survives as long as the underlying storage is intact
- If the node is permanently lost and storage is local (hostPath), data is lost
- Network-attached storage (EBS, PD) survives node failures

### 5. Backup Strategy

**Method: pg_basebackup from a replica**

Backups should run from a replica (not the primary) to avoid impacting write performance:

```bash
# Target postgres-1 (a replica) for backups
kubectl exec postgres-1 -n database -- \
  pg_basebackup -h localhost -U postgres -D /backups/base -Ft -z -P
```

**Backup Storage:**

Use a separate PVC for backups (not the data PVC):
```yaml
volumeClaimTemplates:
- metadata:
    name: data
  spec:
    accessModes: ["ReadWriteOnce"]
    resources:
      requests:
        storage: 50Gi
- metadata:
    name: backups
  spec:
    accessModes: ["ReadWriteOnce"]
    resources:
      requests:
        storage: 100Gi
```

**Backup Schedule:**

Use a CronJob that targets a specific replica by its stable DNS name:
```yaml
env:
- name: PGHOST
  value: postgres-1.db.database.svc.cluster.local
```

The stable DNS name from the StatefulSet makes it possible to reliably target a specific replica for backups.

**Backup Lifecycle:**
- PVCs for backups are per-pod (just like data PVCs)
- If the backup pod is deleted, the backup PVC persists
- Use `Retain` reclaim policy on backup PVCs to prevent accidental deletion
- Consider copying backups to object storage (S3, GCS) for off-site retention

---

## Implementation Files

### headless-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: db
  namespace: database
spec:
  clusterIP: None
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
    name: postgres
```

### write-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: db-write
  namespace: database
spec:
  selector:
    app: postgres
    role: primary
  ports:
  - port: 5432
    targetPort: 5432
    name: postgres
```

Note: The `role: primary` label must be managed externally (by an init script or operator). The StatefulSet pod template applies the same labels to all pods.

### read-service.yaml

```yaml
apiVersion: v1
kind: Service
metadata:
  name: db-read
  namespace: database
spec:
  selector:
    app: postgres
  ports:
  - port: 5432
    targetPort: 5432
    name: postgres
```

### postgres-statefulset.yaml

```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: db
  namespace: database
spec:
  serviceName: db
  replicas: 3
  selector:
    matchLabels:
      app: postgres
  updateStrategy:
    type: RollingUpdate
    rollingUpdate:
      partition: 0
  template:
    metadata:
      labels:
        app: postgres
    spec:
      initContainers:
      - name: init-db
        image: postgres:16
        command:
        - bash
        - -c
        - |
          # Determine role based on ordinal
          ORDINAL=${HOSTNAME##*-}
          if [ "$ORDINAL" = "0" ]; then
            echo "primary" > /etc/pg-role/role
          else
            echo "replica" > /etc/pg-role/role
          fi
        env:
        - name: HOSTNAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        volumeMounts:
        - name: role
          mountPath: /etc/pg-role
      containers:
      - name: postgres
        image: postgres:16
        ports:
        - containerPort: 5432
        env:
        - name: POSTGRES_PASSWORD
          valueFrom:
            secretKeyRef:
              name: pg-secret
              key: password
        - name: PGDATA
          value: /var/lib/postgresql/data/pgdata
        volumeMounts:
        - name: data
          mountPath: /var/lib/postgresql/data
        - name: role
          mountPath: /etc/pg-role
        readinessProbe:
          exec:
            command: ["pg_isready", "-U", "postgres"]
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
        livenessProbe:
          exec:
            command: ["pg_isready", "-U", "postgres"]
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 6
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: "2"
            memory: 4Gi
      volumes:
      - name: role
        emptyDir: {}
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 50Gi
```

### replication-init.yaml (ConfigMap)

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: pg-init-scripts
  namespace: database
data:
  setup-replication.sh: |
    #!/bin/bash
    set -e

    ROLE=$(cat /etc/pg-role/role)
    PGDATA="/var/lib/postgresql/data/pgdata"

    if [ "$ROLE" = "primary" ]; then
      echo "Configuring as primary..."
      # Enable WAL replication
      cat >> "$PGDATA/postgresql.conf" <<EOF
    wal_level = replica
    max_wal_senders = 3
    wal_keep_size = 64MB
    EOF

      # Allow replication connections from replicas
      cat >> "$PGDATA/pg_hba.conf" <<EOF
    host replication postgres 0.0.0.0/0 md5
    EOF
    else
      echo "Configuring as replica..."
      PRIMARY_HOST="db-0.db.database.svc.cluster.local"

      # Stop PostgreSQL, replace data directory with base backup
      pg_ctl stop -D "$PGDATA"
      rm -rf "$PGDATA"/*

      # Take base backup from primary
      PGPASSWORD=$POSTGRES_PASSWORD pg_basebackup \
        -h "$PRIMARY_HOST" -U postgres -D "$PGDATA" -Fp -Xs -P -R

      # The -R flag creates standby.signal and sets primary_conninfo
      echo "primary_conninfo = 'host=$PRIMARY_HOST port=5432 user=postgres password=$POSTGRES_PASSWORD'" \
        >> "$PGDATA/postgresql.auto.conf"
    fi
```

---

## Operational Runbook

### Scenario 1: Scaling from 3 to 5 Replicas

```bash
# Scale the StatefulSet
kubectl scale statefulset db -n database --replicas=5

# Watch the new pods come up (in order)
kubectl get pods -n database -w
# postgres-3 is created first, then postgres-4

# The new pods start as standalone PostgreSQL instances
# They need to be configured as replicas and joined to the replication topology

# Option A: Use the init script (if configured in the StatefulSet)
# The init container will detect it is not ordinal 0 and configure as a replica

# Option B: Manual configuration
# Connect to postgres-3 and configure it as a replica
kubectl exec postgres-3 -n database -- bash -c "
  pg_ctl stop -D /var/lib/postgresql/data/pgdata
  rm -rf /var/lib/postgresql/data/pgdata/*
  PGPASSWORD=\$POSTGRES_PASSWORD pg_basebackup \
    -h db-0.db.database.svc.cluster.local \
    -U postgres \
    -D /var/lib/postgresql/data/pgdata \
    -Fp -Xs -P -R
  pg_ctl start -D /var/lib/postgresql/data/pgdata
"
```

**What happens:**
- PVCs `data-postgres-3` and `data-postgres-4` are created automatically
- Pods start in order (3 first, then 4)
- Each new pod gets its own PVC
- Replication must be configured manually or by init scripts

### Scenario 2: Major Version Upgrade (PostgreSQL 16 to 17)

```bash
# Step 1: Set partition to 2 (canary on postgres-2)
kubectl patch statefulset db -n database \
  -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":2}}}}'

# Step 2: Update image
kubectl set image statefulset/db postgres=postgres:17 -n database

# Step 3: Wait for postgres-2 to be Ready
kubectl wait --for=condition=Ready pod/db-2 -n database --timeout=300s

# Step 4: Verify postgres-2 is working
kubectl exec db-2 -n database -- psql -U postgres -c "SELECT version();"

# Step 5: Check for replication lag
kubectl exec db-2 -n database -- psql -U postgres -c "SELECT now() - pg_last_xact_replay_timestamp() AS lag;"

# Step 6: Monitor for 15 minutes
# Check logs, replication status, query performance

# Step 7: If healthy, roll out to replicas
kubectl patch statefulset db -n database \
  -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":1}}}}'

# Step 8: Wait for postgres-1 to update and verify
kubectl wait --for=condition=Ready pod/db-1 -n database --timeout=300s

# Step 9: Update the primary (during maintenance window)
kubectl patch statefulset db -n database \
  -p '{"spec":{"updateStrategy":{"rollingUpdate":{"partition":0}}}}'

# Step 10: Verify all pods are on postgres:17
kubectl get pods -n database -o custom-columns=\
  NAME:.metadata.name,IMAGE:.spec.containers[0].image
```

**Key points:**
- Always update replicas first, primary last
- The primary is the most critical node; update it during a maintenance window
- If the canary fails, roll back by setting the old image and resetting the partition

### Scenario 3: Handling a Primary Failure

```bash
# When postgres-0 fails:
# 1. Kubernetes detects the failure via liveness probe
# 2. Kubernetes restarts postgres-0 (same name, same PVC)
# 3. PostgreSQL performs crash recovery (replays WAL)

# Monitor the recovery
kubectl get pods -n database -w
kubectl logs db-0 -n database -f

# Check if postgres-0 recovered
kubectl exec db-0 -n database -- pg_isready -U postgres

# Verify replicas reconnected
kubectl exec db-1 -n database -- psql -U postgres -c "SELECT pg_is_in_recovery();"
# Should return: true (it is a replica)

# If postgres-0 cannot recover (corrupted data):
# Option A: Restore from backup
# Option B: Promote a replica to primary
kubectl exec db-1 -n database -- psql -U postgres -c "SELECT pg_promote();"
# Then reconfigure postgres-0 as a replica
```

**Automatic failover requires additional tooling:**
- **Patroni**: Leader election using etcd/consul/ZooKeeper
- **CloudNativePG**: Kubernetes operator with built-in failover
- **Zalando Postgres Operator**: Operator with automatic failover

### Scenario 4: Restoring from Backup

```bash
# Identify the backup to restore
kubectl exec db-1 -n database -- ls -la /backups/base/

# Stop the target pod
kubectl exec db-0 -n database -- pg_ctl stop -D /var/lib/postgresql/data/pgdata

# Clear the data directory
kubectl exec db-0 -n database -- rm -rf /var/lib/postgresql/data/pgdata/*

# Restore from backup (copy from the backup PVC of another pod)
kubectl exec db-0 -n database -- bash -c "
  cp /backups-from-replica/base.tar.gz /var/lib/postgresql/data/
  cd /var/lib/postgresql/data
  tar xzf base.tar.gz -C pgdata/
"

# Or restore from a pg_basebackup taken from a replica
kubectl exec db-0 -n database -- bash -c "
  PGPASSWORD=\$POSTGRES_PASSWORD pg_basebackup \
    -h db-1.db.database.svc.cluster.local \
    -U postgres \
    -D /var/lib/postgresql/data/pgdata \
    -Fp -Xs -P -R
"

# Start PostgreSQL
kubectl exec db-0 -n database -- pg_ctl start -D /var/lib/postgresql/data/pgdata
```

**Key points:**
- Always restore to a replica first, not the primary
- Verify the restored data before promoting
- PVCs are preserved during pod deletion, so the backup PVC is always available
- For point-in-time recovery, use WAL archiving with `archive_command`

---

## Common Mistakes to Avoid

1. **Expecting automatic failover from StatefulSets**
   - StatefulSets provide identity and ordering, NOT application-level failover
   - Use an operator or Patroni for automatic primary promotion

2. **Updating the primary first**
   - Always update replicas first using partitions
   - The primary should be the last node to update

3. **Not using persistent storage for cluster state**
   - PostgreSQL's `pgdata` must be on a PVC
   - Without it, pod restarts lose all data

4. **Forgetting to configure replication**
   - StatefulSets do not configure PostgreSQL replication automatically
   - You must set up `wal_level`, `max_wal_senders`, `pg_hba.conf`, and `primary_conninfo`

5. **Running backups on the primary**
   - Backups should run on replicas to avoid impacting write performance
   - Use the stable DNS name of a replica for backup targeting

6. **Not setting resource limits**
   - PostgreSQL can consume all available memory
   - Set limits to prevent OOM kills and noisy neighbor issues

7. **Deleting PVCs when scaling down**
   - PVCs are intentionally preserved
   - Scaling back up will reattach to existing PVCs with data intact
