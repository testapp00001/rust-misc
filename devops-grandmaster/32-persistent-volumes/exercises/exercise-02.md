# Exercise 02: Provisioning Storage by Hand -- PVs, PVCs, and Access Modes

**Type:** Guided
**Time:** 35 minutes
**Difficulty:** Easy-Medium

## Objective

Create PersistentVolumes and PersistentVolumeClaims manually (static provisioning),
observe the binding process, and understand how access modes and capacity affect
matching.

---

## Background

Your team runs a development cluster on minikube. You need to provision storage
for three different applications:

| Application | Storage Need | Access Pattern |
|-------------|-------------|----------------|
| PostgreSQL database | 5Gi | Single writer, single node |
| Shared config files | 2Gi | Multiple readers, single writer |
| Static website assets | 1Gi | Multiple readers, no writers |

You will create PVs and PVCs for each application using `hostPath` as the backing
store (since you are on minikube).

---

## Tasks

### Task 1: Create a PV for PostgreSQL

Create a PersistentVolume named `pg-pv` with:
- 5Gi capacity
- ReadWriteOnce access mode
- `Retain` reclaim policy
- StorageClassName: `manual`
- hostPath backing: `/mnt/pgdata`

Apply it and verify the status is `Available`.

<details>
<summary>Hint 1: PV YAML structure</summary>

```yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: pg-pv
spec:
  capacity:
    storage: 5Gi
  accessModes:
  - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  storageClassName: manual
  hostPath:
    path: /mnt/pgdata
```

</details>

### Task 2: Create a PVC and Bind It

Create a PersistentVolumeClaim named `pg-pvc` that requests:
- 3Gi of storage (less than the PV's 5Gi)
- ReadWriteOnce access mode
- StorageClassName: `manual`

Apply it, then verify:
1. The PVC status is `Bound`.
2. The PV status changed from `Available` to `Bound`.
3. The PVC is bound to `pg-pv` (check `kubectl get pvc -o wide`).

<details>
<summary>Hint 2: PVC matching</summary>

A PVC can bind to a PV with **equal or greater** capacity. A 3Gi PVC binds to a
5Gi PV. The remaining 2Gi is unused but reserved -- no other PVC can claim it.

</details>

### Task 3: Create Storage for Shared Config

Create a PV named `config-pv` and a PVC named `config-pvc` for the shared config
application:
- 2Gi capacity
- ReadOnlyMany (ROX) access mode
- StorageClassName: `manual`
- hostPath: `/mnt/config`

Apply both and verify binding.

<details>
<summary>Hint 3: ReadOnlyMany with hostPath</summary>

hostPath supports ROX at the Kubernetes API level, but in practice it only works
on a single node. For true multi-node read-only access, you need NFS or a similar
network filesystem. On minikube (single node), hostPath works fine for learning.

</details>

### Task 4: Test Capacity Mismatch

Attempt to create a PVC named `big-pvc` that requests:
- 10Gi of storage
- ReadWriteOnce access mode
- StorageClassName: `manual`

This should fail to bind because no PV with `manual` storageClass has 10Gi
available.

1. Apply the PVC.
2. Check its status (`kubectl get pvc big-pvc`).
3. Describe the PVC to see the events (`kubectl describe pvc big-pvc`).
4. Explain why it remains in `Pending` state.
5. Delete the PVC after observing the behavior.

<details>
<summary>Hint 4: Pending state</summary>

When no PV matches a PVC's requirements, the PVC stays in `Pending`. The PVC's
events will show `FailedScheduling` or similar messages indicating no matching
volume was found. This is the expected behavior -- Kubernetes does not create
storage out of thin air for static provisioning.

</details>

### Task 5: Test Access Mode Mismatch

Attempt to create a PVC named `mode-pvc` that requests:
- 2Gi of storage
- ReadWriteMany (RWX) access mode
- StorageClassName: `manual`

This should also remain Pending because none of the existing PVs with
`manual` storageClass have `ReadWriteMany` access mode.

1. Apply the PVC.
2. Check its status and describe it.
3. Explain why it does not bind to `config-pv` (which is ROX, not RWX).
4. Delete the PVC.

---

## Success Criteria

- [ ] `pg-pv` and `pg-pvc` are created and bound.
- [ ] `config-pv` and `config-pvc` are created and bound.
- [ ] `big-pvc` remains Pending due to insufficient capacity.
- [ ] `mode-pvc` remains Pending due to access mode mismatch.
- [ ] You can explain the PVC-to-PV matching criteria (storageClassName, access
      modes, capacity).

---

## Hints

<details>
<summary>Hint 5: Checking binding details</summary>

Use these commands to inspect storage resources:

```bash
kubectl get pv                     # List all PVs with status
kubectl get pvc                    # List PVCs in current namespace
kubectl get pvc -o wide            # Shows the bound PV name
kubectl describe pv <name>         # Full details including bound PVC
kubectl describe pvc <name>        # Events, bound PV, access mode
```

</details>

<details>
<summary>Hint 6: Cleanup order</summary>

Delete resources in this order to avoid issues:
1. Pods that mount the PVC.
2. PVCs.
3. PVs.

If you delete a PV while a PVC is still bound to it, the PV enters `Terminating`
state and waits for the PVC to be released first.

</details>
