# Solution 05: Storage Architecture for a Production Platform -- Putting It All Together

---

## Task 1: Design StorageClasses

### premium (PostgreSQL)

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: premium
provisioner: ebs.csi.aws.com
parameters:
  type: io2
  iopsPerGB: "50"
  encrypted: "true"
reclaimPolicy: Retain
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

### standard (Redis, Search)

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: standard
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
  encrypted: "true"
reclaimPolicy: Delete
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

### shared-fs (User Uploads)

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
allowVolumeExpansion: true
volumeBindingMode: Immediate
```

### config-ro (App Config)

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: config-ro
provisioner: efs.csi.aws.com
parameters:
  provisioningMode: efs-ap
  fileSystemId: fs-0123456789abcdef0
  directoryPerms: "755"
reclaimPolicy: Retain
allowVolumeExpansion: false
volumeBindingMode: Immediate
```

**Why this works:** Each StorageClass maps to a storage tier with the right
balance of performance, durability, and cost. EBS classes use `WaitForFirstConsumer`
to respect zone affinity. EFS classes use `Immediate` because EFS is a regional
service accessible from all zones.

---

## Task 2: Design PVCs and Access Modes

| Component | PVC Name | StorageClass | Size | Access Mode | Reason |
|-----------|----------|-------------|------|-------------|--------|
| PostgreSQL primary | `pg-primary-data` | `premium` | 200Gi | **ReadWriteOnce** | One writer on one node. RWO is the standard for databases. |
| PostgreSQL replica | `pg-replica-data` | `premium` | 200Gi | **ReadWriteOnce** | One reader on one node. The replica writes WAL data locally. |
| Redis | `redis-data` | `standard` | 10Gi | **ReadWriteOnce** | Single Redis instance owns the data. |
| App config | `app-config` | `config-ro` | 1Gi | **ReadOnlyMany** | Many pods read the config; one admin process writes. ROX allows multi-node reads. |
| User uploads | `user-uploads` | `shared-fs` | 500Gi | **ReadWriteMany** | Multiple pods on multiple nodes serve and accept uploads simultaneously. |
| Search index | `search-data` | `standard` | 100Gi | **ReadWriteOnce** | One Elasticsearch node owns its shard data. |

### PVC Manifests

```yaml
# PostgreSQL primary
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pg-primary-data
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: premium
  resources:
    requests:
      storage: 200Gi
---
# PostgreSQL replica
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pg-replica-data
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: premium
  resources:
    requests:
      storage: 200Gi
---
# Redis
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: redis-data
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: standard
  resources:
    requests:
      storage: 10Gi
---
# App config
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: app-config
spec:
  accessModes:
  - ReadOnlyMany
  storageClassName: config-ro
  resources:
    requests:
      storage: 1Gi
---
# User uploads
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: user-uploads
spec:
  accessModes:
  - ReadWriteMany
  storageClassName: shared-fs
  resources:
    requests:
      storage: 500Gi
---
# Search index
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: search-data
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: standard
  resources:
    requests:
      storage: 100Gi
```

**Why this works:** Access modes are chosen based on the actual read/write pattern
of each component. Using the strictest appropriate mode prevents accidental
concurrent writes that could corrupt data.

---

## Task 3: Pod Manifests

### PostgreSQL Primary

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: postgres-primary
  labels:
    app: postgres
    role: primary
spec:
  containers:
  - name: postgres
    image: postgres:16
    env:
    - name: POSTGRES_PASSWORD
      valueFrom:
        secretKeyRef:
          name: pg-credentials
          key: password
    - name: PGDATA
      value: /var/lib/postgresql/data/pgdata
    ports:
    - containerPort: 5432
    volumeMounts:
    - mountPath: /var/lib/postgresql/data
      name: pgdata
  volumes:
  - name: pgdata
    persistentVolumeClaim:
      claimName: pg-primary-data
```

### PostgreSQL Replica

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: postgres-replica
  labels:
    app: postgres
    role: replica
spec:
  containers:
  - name: postgres
    image: postgres:16
    env:
    - name: POSTGRES_PASSWORD
      valueFrom:
        secretKeyRef:
          name: pg-credentials
          key: password
    - name: PGDATA
      value: /var/lib/postgresql/data/pgdata
    ports:
    - containerPort: 5432
    volumeMounts:
    - mountPath: /var/lib/postgresql/data
      name: pgdata
  volumes:
  - name: pgdata
    persistentVolumeClaim:
      claimName: pg-replica-data
```

### Redis

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: redis
spec:
  containers:
  - name: redis
    image: redis:7-alpine
    command: ["redis-server", "--appendonly", "yes"]
    ports:
    - containerPort: 6379
    volumeMounts:
    - mountPath: /data
      name: redis-data
  volumes:
  - name: redis-data
    persistentVolumeClaim:
      claimName: redis-data
```

### App Config (read-only mount)

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: app-config-writer
spec:
  containers:
  - name: writer
    image: busybox
    command: ["sh", "-c", "echo '{\"feature_x\": true}' > /config/flags.json && sleep 3600"]
    volumeMounts:
    - mountPath: /config
      name: config
  volumes:
  - name: config
    persistentVolumeClaim:
      claimName: app-config
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
spec:
  replicas: 5
  selector:
    matchLabels:
      app: ecommerce
  template:
    metadata:
      labels:
        app: ecommerce
    spec:
      containers:
      - name: app
        image: myapp:latest
        volumeMounts:
        - mountPath: /etc/app-config
          name: config
          readOnly: true       # <-- read-only inside the container
      volumes:
      - name: config
        persistentVolumeClaim:
          claimName: app-config
```

### User Uploads (3 replicas, ReadWriteMany)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: upload-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: uploads
  template:
    metadata:
      labels:
        app: uploads
    spec:
      containers:
      - name: nginx
        image: nginx:alpine
        volumeMounts:
        - mountPath: /uploads
          name: uploads
      volumes:
      - name: uploads
        persistentVolumeClaim:
          claimName: user-uploads
```

### Elasticsearch

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: elasticsearch
spec:
  containers:
  - name: es
    image: elasticsearch:8.12.0
    env:
    - name: discovery.type
      value: single-node
    - name: xpack.security.enabled
      value: "false"
    ports:
    - containerPort: 9200
    volumeMounts:
    - mountPath: /usr/share/elasticsearch/data
      name: es-data
  volumes:
  - name: es-data
    persistentVolumeClaim:
      claimName: search-data
```

**Why this works:** Each pod mounts its PVC at the appropriate path. The app config
Deployment uses `readOnly: true` at the container level to prevent application pods
from modifying configuration. The upload service uses RWX via EFS so all three
replicas can serve and accept uploads.

---

## Task 4: Reclaim Policy and Lifecycle Planning

| Component | Reclaim | PVC Deleted -> Data? | Recovery Method | RPO | Expansion? |
|-----------|---------|---------------------|-----------------|-----|------------|
| PostgreSQL primary | **Retain** | PV becomes `Released`. EBS volume preserved. | Rebind PV to new PVC (remove claimRef) or restore from backup. | ~0 (WAL archiving) | **Yes** -- database grows 20Gi/month |
| PostgreSQL replica | **Retain** | PV becomes `Released`. EBS volume preserved. | Rebuild replica from primary (pg_basebackup). | ~0 (streaming replication) | **Yes** -- tracks primary growth |
| Redis | **Delete** | PV and EBS volume deleted. Data is gone. | Rebuild from application state. Sessions lost; cache repopulated. | Minutes (AOF persistence) | **Yes** -- may grow slowly |
| App config | **Retain** | PV becomes `Released`. EFS data preserved. | Rebind PV or redeploy from Git (config is in version control). | N/A (config is declarative) | **No** -- 1Gi is sufficient |
| User uploads | **Retain** | PV becomes `Released`. EFS data preserved. | Rebind PV. EFS data persists independently of PV lifecycle. | ~0 (EFS is durable) | **Yes** -- grows 50Gi/month |
| Search index | **Delete** | PV and EBS volume deleted. Index gone. | Re-index from PostgreSQL database. | Hours (re-index time) | **Yes** -- grows 10Gi/month |

### Key Decisions

**Retain for PostgreSQL and uploads:** These contain irreplaceable data. A Retain
policy gives the operations team time to detect accidental PVC deletion and
reattach the volume before it is too late.

**Delete for Redis and search:** Redis data is semi-ephemeral (sessions can be
re-established, cache repopulates). The search index is rebuildable from the
database. Delete policy reduces cleanup overhead.

**Volume expansion enabled for growing workloads:** PostgreSQL and uploads grow
continuously. With `allowVolumeExpansion: true`, you can resize online without
migration. Config and Redis do not need expansion.

---

## Task 5: Cost Optimization

### Current Cost Breakdown

| Tier | Monthly Cost |
|------|-------------|
| premium (io2, 400Gi) | ~$80 |
| standard (gp3, 110Gi) | ~$9 |
| shared-fs (EFS, 500Gi) | ~$150 |
| config-ro (EFS, 1Gi) | ~$0.30 |
| **Total** | **~$239** |

### Optimization Proposals

#### 1. Move PostgreSQL Replica to gp3 (Save ~$35/month)

The replica is read-only and can tolerate slightly higher latency. gp3 at 200Gi
costs ~$16/month vs ~$50+ for io2. The replica does not need the guaranteed IOPS
of io2 because reads are less latency-sensitive than writes.

**Risk:** Slightly higher read latency on the replica. Monitor query performance
after migration.

#### 2. Move User Uploads from EFS to S3 + CloudFront (Save ~$120/month)

EFS at $0.30/GiB is expensive for large, read-heavy datasets. S3 at $0.023/GiB
is 13x cheaper. CloudFront serves images with low latency globally.

```
Current: EFS (500Gi) = $150/month
Proposed: S3 (500Gi) = $11.50 + CloudFront = ~$20/month
Savings: ~$120/month
```

**Trade-off:** Requires application changes. S3 is object storage (not POSIX).
Upload service must use S3 API instead of filesystem writes. Existing code that
reads files from a local path must be refactored.

#### 3. Use gp3 for Search Index (Already Done)

The search index already uses the `standard` (gp3) StorageClass. No change needed.

#### 4. Lifecycle Policies for Old Data

- **User uploads:** Move images older than 90 days to S3 Infrequent Access
  ($0.0125/GiB) or S3 Glacier ($0.004/GiB).
- **Search index:** Delete indices older than 6 months.
- **PostgreSQL:** Archive old data to S3 and remove from the primary database.

### Revised Cost Estimate

| Tier | Optimization | New Monthly Cost |
|------|-------------|-----------------|
| premium (io2, 200Gi primary) | Unchanged | ~$50 |
| standard (gp3, 200Gi replica + 110Gi) | Replica moved to gp3 | ~$25 |
| shared-fs -> S3 + CloudFront | Move uploads to S3 | ~$20 |
| config-ro (EFS, 1Gi) | Unchanged | ~$0.30 |
| **Total** | | **~$95** |

**Savings: ~$144/month (60% reduction)**

---

## Common Mistakes

1. **Using RWO for shared filesystems.** If multiple pods need to write to the
   same volume, RWO fails when pods land on different nodes. Use RWX (EFS, CephFS)
   for shared writes. Using RWO forces all replicas to the same node, defeating
   high availability.

2. **Using Delete policy for irreplaceable data.** If a developer accidentally
   deletes a PVC, Delete policy destroys the underlying volume. For databases and
   user data, always use Retain. The manual cleanup step is a safety net.

3. **Over-provisioning IOPS.** io2 volumes charge per IOPS provisioned. A 200Gi
   volume with `iopsPerGB: 50` gets 10,000 IOPS. If the application only uses
   2,000 IOPS, you are paying for 8,000 unused IOPS. Monitor actual IOPS usage
   and right-size.

4. **Not enabling volume expansion from the start.** If you forget
   `allowVolumeExpansion: true` when creating the StorageClass, you cannot add it
   retroactively for existing PVCs (you must recreate the StorageClass). Always
   enable it -- the cost is zero, and you will eventually need it.

5. **Treating storage durability as a backup strategy.** Retain policy protects
   against PVC deletion, but not against application bugs, accidental SQL DELETEs,
   or ransomware. Implement application-level backups (pg_dump, volume snapshots)
   with off-site replication for true disaster recovery.

6. **Ignoring zone affinity for EBS.** Using `Immediate` binding mode with EBS
   creates volumes in a random zone. If the pod is scheduled in a different zone,
   it cannot attach the volume. Always use `WaitForFirstConsumer` for EBS
   StorageClasses.
