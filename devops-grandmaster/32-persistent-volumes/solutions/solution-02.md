# Solution 02: Provisioning Storage by Hand -- PVs, PVCs, and Access Modes

---

## Task 1: Create a PV for PostgreSQL

```yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: pg-pv
  labels:
    app: postgres
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

```bash
kubectl apply -f pg-pv.yaml
kubectl get pv pg-pv
```

Expected output:

```
NAME    CAPACITY   ACCESS MODES   RECLAIM POLICY   STATUS      CLAIM   STORAGECLASS
pg-pv   5Gi        RWO            Retain           Available           manual
```

**Why this works:** The PV declares 5Gi of storage backed by a hostPath directory.
The `Available` status means no PVC has claimed it yet. The `Retain` reclaim policy
means the data persists even if the PVC is deleted.

---

## Task 2: Create a PVC and Bind It

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: pg-pvc
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 3Gi
  storageClassName: manual
```

```bash
kubectl apply -f pg-pvc.yaml

# Check PVC status
kubectl get pvc pg-pvc -o wide
```

Expected output:

```
NAME     STATUS   VOLUME   CAPACITY   ACCESS MODES   STORAGECLASS   AGE
pg-pvc   Bound    pg-pv    5Gi        RWO            manual         5s
```

Check the PV status:

```bash
kubectl get pv pg-pv
```

Expected output:

```
NAME    CAPACITY   ACCESS MODES   RECLAIM POLICY   STATUS   CLAIM              STORAGECLASS
pg-pv   5Gi        RWO            Retain           Bound    default/pg-pvc     manual
```

**Why this works:** The PVC requests 3Gi with RWO and `manual` storageClass. The
PV `pg-pv` has 5Gi with RWO and `manual` storageClass. Since the PV's capacity
(5Gi) is greater than or equal to the PVC's request (3Gi), and the access mode and
storageClass match, binding succeeds. The remaining 2Gi on the PV is reserved and
cannot be claimed by another PVC.

---

## Task 3: Create Storage for Shared Config

```yaml
# config-pv.yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: config-pv
  labels:
    app: shared-config
spec:
  capacity:
    storage: 2Gi
  accessModes:
  - ReadOnlyMany
  persistentVolumeReclaimPolicy: Retain
  storageClassName: manual
  hostPath:
    path: /mnt/config
```

```yaml
# config-pvc.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: config-pvc
spec:
  accessModes:
  - ReadOnlyMany
  resources:
    requests:
      storage: 2Gi
  storageClassName: manual
```

```bash
kubectl apply -f config-pv.yaml
kubectl apply -f config-pvc.yaml
kubectl get pv,pvc
```

**Why this works:** The PV and PVC both specify `ReadOnlyMany` (ROX) and
`storageClassName: manual`. The capacity matches exactly. Kubernetes binds them
immediately.

In practice, `hostPath` with ROX only works on a single node. On a multi-node
cluster, you would use NFS, CephFS, or a cloud-native shared filesystem for true
multi-node read access.

---

## Task 4: Test Capacity Mismatch

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: big-pvc
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: manual
```

```bash
kubectl apply -f big-pvc.yaml
kubectl get pvc big-pvc
```

Expected output:

```
NAME      STATUS    VOLUME   CAPACITY   ACCESS MODES   STORAGECLASS   AGE
big-pvc   Pending                                      manual         10s
```

```bash
kubectl describe pvc big-pvc
```

Events section will show:

```
Events:
  Type     Reason              Age   From                         Message
  ----     ------              ----  ----                         -------
  Warning  ProvisioningFailed  15s   persistentvolume-controller  storageclass "manual"
                                    not found
```

In this exercise, `manual` is not an actual StorageClass resource -- it is just a
label on the static PVs. The dynamic provisioner looks for a StorageClass named
`manual`, fails to find one, and logs the warning. Meanwhile, Kubernetes also checks
for static PVs with `storageClassName: manual` and capacity >= 10Gi, but none exist.
The PVC stays Pending.

If a StorageClass named `manual` did exist, the events would instead show the
controller waiting for dynamic provisioning or a PV match.

**Why this remains Pending:** The PVC requests 10Gi. The largest available PV with
`manual` storageClass and RWO access mode is `pg-pv` at 5Gi. Since 5Gi < 10Gi, no
match is found. For static provisioning, Kubernetes does not create storage -- it
can only bind to existing PVs.

```bash
# Cleanup
kubectl delete pvc big-pvc
```

---

## Task 5: Test Access Mode Mismatch

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: mode-pvc
spec:
  accessModes:
  - ReadWriteMany
  resources:
    requests:
      storage: 2Gi
  storageClassName: manual
```

```bash
kubectl apply -f mode-pvc.yaml
kubectl get pvc mode-pvc
```

Expected output:

```
NAME       STATUS    VOLUME   CAPACITY   ACCESS MODES   STORAGECLASS   AGE
mode-pvc   Pending                                      manual         10s
```

**Why it does not bind to config-pv:** `config-pv` has access mode `ReadOnlyMany`
(ROX). The PVC requests `ReadWriteMany` (RWX). These are different access modes.
A PV must support the exact access mode the PVC requests. ROX does not satisfy RWX.

**Why this works this way:** Access modes are a contract. If a PVC says "I need
ReadWriteMany," it means the application will write to the volume from multiple
nodes. A volume provisioned as read-only cannot safely support writes -- the
filesystem may not support concurrent writes, or writes may corrupt data.

```bash
# Cleanup
kubectl delete pvc mode-pvc
```

---

## Common Mistakes

1. **Expecting PVC to bind to a PV with different storageClassName.** The
   storageClassName must match exactly. A PVC with `storageClassName: fast` will
   never bind to a PV with `storageClassName: manual`, even if capacity and access
   modes are compatible.

2. **Deleting a PV before its PVC.** If you delete a PV while a PVC is still bound
   to it, the PV enters `Terminating` state and waits. Always delete PVCs first,
   then PVs.

3. **Confusing RWO with single-pod access.** ReadWriteOnce means one **node**, not
   one pod. Multiple pods on the same node can all read and write to an RWO volume.
   This is critical for StatefulSets where pods may be co-located.

4. **Not setting storageClassName on static PVs.** If you omit `storageClassName`
   on both the PV and PVC, Kubernetes uses the default StorageClass (if one exists)
   and may trigger dynamic provisioning instead of binding to your static PV. Always
   set an explicit storageClassName for static provisioning.

5. **Assuming remaining capacity is reusable.** When a 3Gi PVC binds to a 5Gi PV,
   the remaining 2Gi is locked. No other PVC can claim it. If you need multiple
   PVCs, create multiple PVs of the right size.
