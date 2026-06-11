# Solution 04: StorageClass Migration Under Pressure -- Moving Data Between Tiers

---

## Task 1: Can You Change the StorageClassName of an Existing PVC?

```bash
kubectl edit pvc pg-data
# Change storageClassName from "standard" to "premium"
```

**Result:** The edit is **silently ignored or rejected** depending on the Kubernetes
version. Since Kubernetes 1.13, the API server treats `storageClassName` as
immutable on bound PVCs and rejects changes with a validation error:

```
error: persistentvolumeclaims "pg-data" could not be patched:
the PersistentVolumeClaim "pg-data" is invalid:
spec.storageClassName: Invalid value: "premium": field is immutable
```

**Why it is immutable:** The `storageClassName` determines which provisioner
manages the PV. Changing it would mean the existing PV (created by the `standard`
provisioner) no longer matches the PVC. Kubernetes cannot re-provision storage
under a running workload without risking data loss. The binding is a one-time
contract between PVC and PV.

**What is mutable on a PVC:**
- `resources.requests.storage` -- can be **increased** (if `allowVolumeExpansion`
  is true), never decreased.
- `metadata.labels` and `metadata.annotations` -- always mutable.
- `spec.accessModes` -- immutable after creation.

---

## Task 2: Design a Migration Strategy

### Migration Steps

1. **Create new PVC** (`pg-data-premium`) on the `premium` StorageClass.
2. **Scale down** the database pod to ensure data consistency.
3. **Migrate data** using either:
   - `pg_dump` / `pg_restore` (logical backup), or
   - File-level copy via a migration pod (physical copy).
4. **Update the Pod/Deployment** to mount the new PVC.
5. **Scale up** the database with the new storage.
6. **Verify** data integrity.
7. **Delete** old PVC (PV persists due to Retain policy).

### New PVC YAML

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pg-data-premium
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: premium
  resources:
    requests:
      storage: 100Gi
```

### Migration Commands (pg_dump approach)

```bash
# Step 1: Create the new PVC
kubectl apply -f pg-data-premium.yaml

# Step 2: Scale down the database
kubectl delete pod postgres

# Step 3: Dump the database (from a temporary pod with old PVC mounted)
kubectl run dump-pod --image=postgres:16 --restart=Never \
  --overrides='{
    "spec": {
      "containers": [{
        "name": "dump-pod",
        "image": "postgres:16",
        "command": ["sleep", "3600"],
        "volumeMounts": [{"mountPath": "/var/lib/postgresql/data", "name": "pgdata"}]
      }],
      "volumes": [{"name": "pgdata", "persistentVolumeClaim": {"claimName": "pg-data"}}]
    }
  }'

# Dump from inside the pod
kubectl exec dump-pod -- pg_dumpall -U postgres > /tmp/full-dump.sql

# Step 4: Create new pod with premium PVC and restore
kubectl run restore-pod --image=postgres:16 --restart=Never \
  --env="PGDATA=/var/lib/postgresql/data/pgdata" \
  --overrides='{
    "spec": {
      "containers": [{
        "name": "restore-pod",
        "image": "postgres:16",
        "env": [{"name": "PGDATA", "value": "/var/lib/postgresql/data/pgdata"}],
        "volumeMounts": [{"mountPath": "/var/lib/postgresql/data", "name": "pgdata"}]
      }],
      "volumes": [{"name": "pgdata", "persistentVolumeClaim": {"claimName": "pg-data-premium"}}]
    }
  }'

# Wait for postgres to initialize, then restore
kubectl exec -i restore-pod -- psql -U postgres < /tmp/full-dump.sql

# Step 5: Update the main pod to use the new PVC
# Step 6: Verify data integrity
# Step 7: Clean up old PVC
```

**Why this works:** The pg_dump approach creates a logical backup of the database
(SQL statements). The restore replays those statements on the new volume. This
works regardless of the underlying storage type, filesystem, or disk layout.

---

## Task 3: Implement the Migration

### Create the Premium StorageClass

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: premium
provisioner: ebs.csi.aws.com
parameters:
  type: io2
  iopsPerGB: "100"
reclaimPolicy: Retain
volumeBindingMode: WaitForFirstConsumer
```

### Create the New PVC

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pg-data-premium
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: premium
  resources:
    requests:
      storage: 100Gi
```

### Migration Pod (Physical Copy Approach)

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: pg-migration
spec:
  containers:
  - name: migrator
    image: busybox
    command:
    - sh
    - -c
    - |
      echo "Starting migration..."
      cp -av /old-data/. /new-data/
      echo "Migration complete."
    volumeMounts:
    - name: old-vol
      mountPath: /old-data
    - name: new-vol
      mountPath: /new-data
  volumes:
  - name: old-vol
    persistentVolumeClaim:
      claimName: pg-data
  - name: new-vol
    persistentVolumeClaim:
      claimName: pg-data-premium
```

### Full Migration Procedure

```bash
# 1. Create the new StorageClass and PVC
kubectl apply -f premium-storageclass.yaml
kubectl apply -f pg-data-premium.yaml

# 2. Scale down the database
kubectl delete pod postgres

# 3. Wait for the new PVC to bind (may need a pod to trigger WaitForFirstConsumer)
# The migration pod itself triggers the binding

# 4. Run the migration pod
kubectl apply -f migration-pod.yaml

# 5. Monitor migration progress and wait for completion
kubectl logs -f pg-migration

# 6. Verify the pod completed successfully
kubectl get pod pg-migration
# STATUS should show "Completed" (or "Succeeded")
# If it shows "Error", check: kubectl logs pg-migration

# 7. Delete the migration pod
kubectl delete pod pg-migration

# 8. Create the database pod with the new PVC
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Pod
metadata:
  name: postgres
spec:
  containers:
  - name: postgres
    image: postgres:16
    env:
    - name: PGDATA
      value: /var/lib/postgresql/data/pgdata
    volumeMounts:
    - mountPath: /var/lib/postgresql/data
      name: pgdata
  volumes:
  - name: pgdata
    persistentVolumeClaim:
      claimName: pg-data-premium
EOF

# 9. Verify data integrity
kubectl exec postgres -- psql -U postgres -c "SELECT count(*) FROM my_table;"
kubectl exec postgres -- psql -U postgres -c "\l"   # List databases
```

**Why this works:** The migration pod mounts both PVCs simultaneously. `cp -av`
preserves permissions, ownership, and timestamps. Since the database is scaled
down, no writes occur during the copy, ensuring consistency.

**Trade-off vs pg_dump:**
- Physical copy is faster for large databases (no SQL parsing).
- Physical copy preserves all PostgreSQL internals (indexes, statistics).
- pg_dump is more portable (works across PostgreSQL versions).
- pg_dump produces a smaller backup (logical, not physical).

---

## Task 4: Verify Reclaim Policy Safety

### Step 1: Confirm Retain Policy

```bash
kubectl get pv -o custom-columns=\
  NAME:.metadata.name,\
  RECLAIM:.spec.persistentVolumeReclaimPolicy,\
  STATUS:.status.phase
```

The PV bound to `pg-data` should show `Retain`.

### Step 2: Delete the Old PVC

```bash
kubectl delete pvc pg-data
```

### Step 3: Check PV Status

```bash
kubectl get pv
```

Expected output:

```
NAME           CAPACITY   RECLAIM POLICY   STATUS
pvc-old-xxx    100Gi      Retain           Released
```

The PV status is `Released`, not `Deleted`. The underlying EBS volume persists.

### Step 4: What Happens to the EBS Volume

The EBS volume is **preserved**. It is not deleted, not detached, and not
formatted. An admin can:

- **Recover the data** by re-binding the PV to a new PVC (remove `claimRef`).
- **Delete the volume** explicitly: `kubectl delete pv pvc-old-xxx`.
- **Leave it** as a backup for a defined retention period.

### Step 5: What If the Policy Were Delete?

If `reclaimPolicy: Delete`, deleting the PVC would trigger:

1. Automatic deletion of the PV object.
2. Automatic deletion of the underlying EBS volume.
3. **All data is permanently destroyed.**

This is catastrophic if the migration corrupted data and you need to roll back.

---

## Task 5: Alternative Strategy -- Volume Expansion

### 1. Required StorageClass field

```yaml
allowVolumeExpansion: true
```

This must be set on the StorageClass at creation time. You can add it to an
existing StorageClass, but it only affects PVCs created after the change.

### 2. Command to expand from 100Gi to 200Gi

```bash
kubectl patch pvc pg-data -p '{"spec":{"resources":{"requests":{"storage":"200Gi"}}}}'
```

### 3. Does the filesystem resize automatically?

**It depends on the CSI driver and the filesystem:**

- **CSI driver with node expansion support:** The volume is expanded at the cloud
  level, then the CSI node plugin resizes the filesystem when the pod restarts
  (or online if supported).
- **Older drivers:** The volume is expanded, but the filesystem stays at the old
  size. You must delete the pod (triggering a restart) to complete the resize.
- **XFS filesystems** may require `xfs_growfs` to be run inside the container.

Check PVC status for `FileSystemResizePending`:

```bash
kubectl describe pvc pg-data
```

### 4. Can you expand to a smaller size?

**No.** Volume expansion is **one-way (grow only)**. Shrinking a filesystem risks
data loss -- files may exist in the space being reclaimed. To shrink, you must
migrate to a new, smaller volume (same as the StorageClass migration strategy).

### 5. Can expansion change the storage tier (gp3 to io2)?

**No.** Expansion only increases the size of the existing volume. The underlying
disk type, IOPS configuration, and provisioner remain the same. To change tiers,
you must migrate to a new PVC backed by a different StorageClass.

---

## Common Mistakes

1. **Trying to edit storageClassName on a live PVC.** This field is immutable.
   Always create a new PVC and migrate data.

2. **Deleting the old PVC before verifying the new one.** If the migration
   corrupted data, you need the old volume to roll back. With Retain policy, the
   old PV persists after PVC deletion. With Delete policy, the data is gone.

3. **Not scaling down the database before copying files.** PostgreSQL uses a WAL
   (Write-Ahead Log) for crash recovery. Copying files while the database is
   running produces an inconsistent snapshot. Either stop the database or use
   `pg_dump` for a consistent logical backup.

4. **Expecting volume expansion to work on all CSI drivers.** Not all CSI drivers
   support `allowVolumeExpansion`. Check the driver documentation before relying
   on it. Also, expansion requires the PVC to be in `Bound` state.

5. **Forgetting that WaitForFirstConsumer delays the new PVC's provisioning.** The
   new PVC stays `Pending` until a pod references it. If you create the PVC but
   forget to create a pod, it looks like the migration is stuck.
