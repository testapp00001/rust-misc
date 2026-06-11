# Exercise 04: StorageClass Migration Under Pressure -- Moving Data Between Tiers

**Type:** Challenge
**Time:** 50 minutes
**Difficulty:** Medium-Hard

## Objective

Migrate a running PostgreSQL database from one StorageClass to another without
data loss. This requires understanding PV lifecycle, PVC immutability, and
strategies for data migration in Kubernetes.

---

## Background

Your production PostgreSQL database currently runs on `standard` storage (gp3
SSD). The database has grown, and performance is degrading. Management has approved
a migration to `premium` storage (io2 Block Express). The database must remain
available (or have minimal downtime) during the migration.

Current state:

```yaml
# Current StorageClass
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: standard
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
reclaimPolicy: Retain
volumeBindingMode: WaitForFirstConsumer
---
# Current PVC
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pg-data
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: standard
  resources:
    requests:
      storage: 100Gi
---
# Current Pod (simplified)
apiVersion: v1
kind: Pod
metadata:
  name: postgres
spec:
  containers:
  - name: postgres
    image: postgres:16
    volumeMounts:
    - mountPath: /var/lib/postgresql/data
      name: pgdata
  volumes:
  - name: pgdata
    persistentVolumeClaim:
      claimName: pg-data
```

Target StorageClass:

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

---

## Tasks

### Task 1: Can You Change the StorageClassName of an Existing PVC?

Attempt to edit the existing PVC to change its storageClassName from `standard`
to `premium`.

1. Run: `kubectl edit pvc pg-data` and change the storageClassName.
2. What happens? Does Kubernetes accept the change?
3. Explain why PVC storageClassName is (or is not) immutable.

<details>
<summary>Hint 1: PVC immutability</summary>

The `storageClassName` field on a PVC is **set once at creation and cannot be
changed**. Kubernetes treats it as an immutable field. Even if you force-edit it,
the underlying PV is not replaced. The PVC remains bound to the original PV.

Some fields on a PVC are mutable (e.g., `resources.requests.storage` for expansion),
but `storageClassName` is not one of them.

</details>

### Task 2: Design a Migration Strategy

Since you cannot change the storageClassName, you need a different approach.
Design a migration strategy using a **second PVC and pg_dump/pg_restore**.

Fill in the migration steps:

1. Create a new PVC named `pg-data-premium` using the `premium` StorageClass.
2. Scale down the database pod (or create a maintenance window).
3. Dump the database from the old volume.
4. Restore the database to the new volume.
5. Update the pod/Deployment to use the new PVC.
6. Verify data integrity.
7. Delete the old PVC (but keep the PV with Retain policy).

Write the YAML for the new PVC and the commands for each step.

<details>
<summary>Hint 2: pg_dump approach</summary>

```bash
# Dump from the running pod
kubectl exec postgres -- pg_dump -U postgres -d mydb > /tmp/dump.sql

# Restore to the new volume
kubectl exec -i postgres-new -- psql -U postgres -d mydb < /tmp/dump.sql
```

For large databases, consider using `pg_dump --format=custom` for compressed
output and parallel restore with `pg_restore -j N`.

</details>

### Task 3: Implement the Migration

Execute the migration:

1. Create the `premium` StorageClass and the `pg-data-premium` PVC.
2. Scale down the database pod.
3. Create a temporary "migration pod" that has access to **both** PVCs
   simultaneously. Use it to copy data from the old volume to the new one.
4. Scale up the database pod using the new PVC.
5. Verify the data.

Write all the YAML manifests and commands.

<details>
<summary>Hint 3: Migration pod with two volumes</summary>

A pod can mount multiple PVCs simultaneously:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: migration
spec:
  containers:
  - name: migrator
    image: busybox
    command: ["sleep", "3600"]
    volumeMounts:
    - mountPath: /old-data
      name: old
    - mountPath: /new-data
      name: new
  volumes:
  - name: old
    persistentVolumeClaim:
      claimName: pg-data
  - name: new
    persistentVolumeClaim:
      claimName: pg-data-premium
```

Then `cp -a` from `/old-data` to `/new-data`.

</details>

### Task 4: Verify Reclaim Policy Safety

After migration, verify that deleting the old PVC does not destroy the data
(important for disaster recovery):

1. Confirm the old PV has `reclaimPolicy: Retain`.
2. Delete the old PVC.
3. Check the PV status.
4. Explain what happens to the underlying EBS volume.
5. What would happen if the reclaim policy were `Delete`?

<details>
<summary>Hint 4: Retain vs Delete after migration</summary>

With `Retain`, the PV enters `Released` state. The EBS volume persists. You can
reattach it to a new PVC by removing the `claimRef` field from the PV. This is a
safety net -- if the migration fails, you can rebind to the old volume.

With `Delete`, the PV and EBS volume are destroyed immediately. If the migration
corrupted data, recovery is impossible.

</details>

### Task 5: Alternative Strategy -- Volume Expansion

If the only issue is storage size (not IOPS), you could expand the existing
volume instead of migrating. Answer:

1. What field must be set on the StorageClass to allow expansion?
2. What command expands a PVC from 100Gi to 200Gi?
3. Does the filesystem resize happen automatically?
4. Can you expand a PVC to a **smaller** size? Why or why not?
5. Can expansion change the storage tier (gp3 to io2)?

<details>
<summary>Hint 5: Expansion limitations</summary>

Volume expansion can only **grow** a volume, never shrink it. Shrinking would
require copying data to a smaller volume (same as the migration strategy).
Expansion also cannot change the underlying disk type -- you cannot turn a gp3
volume into an io2 volume. For tier changes, you must migrate.

</details>

---

## Success Criteria

- [ ] You understand that PVC storageClassName is immutable.
- [ ] You can design a migration strategy using a second PVC.
- [ ] You can create a migration pod that mounts both old and new PVCs.
- [ ] You understand reclaim policy implications during migration.
- [ ] You know when volume expansion is a viable alternative to migration.

---

## Hints

<details>
<summary>Hint 6: Minimizing downtime</summary>

For minimal downtime:

1. Create the new PVC and migration pod **before** scaling down the database.
2. Use `pg_dump --format=custom` for faster backup/restore.
3. Consider logical replication for near-zero-downtime migration (PostgreSQL
   native feature).
4. For the file-copy approach, the database must be stopped to ensure consistency.

</details>

<details>
<summary>Hint 7: Data integrity verification</summary>

After migration, verify:

```bash
# Check table counts
kubectl exec postgres-new -- psql -U postgres -d mydb -c \
  "SELECT schemaname, tablename, n_tup_ins FROM pg_stat_user_tables;"

# Check database size
kubectl exec postgres-new -- psql -U postgres -d mydb -c \
  "SELECT pg_size_pretty(pg_database_size('mydb'));"

# Run application smoke tests
```

</details>
