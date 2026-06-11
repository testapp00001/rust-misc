# Solution 03: Dynamic Provisioning with StorageClasses -- Let the Cluster Handle It

---

## Task 1: Design Three StorageClasses

### fast-io (Payment Database)

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-io
provisioner: ebs.csi.aws.com
parameters:
  type: io1
  iopsPerGB: "50"
reclaimPolicy: Retain
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

### standard-gp3 (Application Logs)

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: standard-gp3
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
reclaimPolicy: Delete
allowVolumeExpansion: true
volumeBindingMode: Immediate
```

### ml-storage (ML Artifacts)

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: ml-storage
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
  encrypted: "true"
reclaimPolicy: Retain
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

**Why this works:** Each StorageClass encodes a storage tier. The provisioner
(`ebs.csi.aws.com`) knows how to create AWS EBS volumes with the specified
parameters. The reclaim policy and binding mode control lifecycle behavior.

---

## Task 2: Understand volumeBindingMode

### 1. When does Immediate mode create the underlying cloud disk?

**As soon as the PVC is created.** The provisioner does not wait for a pod. It
creates the EBS volume immediately and binds the PVC to the new PV.

### 2. When does WaitForFirstConsumer mode create the underlying cloud disk?

**When a pod that references the PVC is scheduled.** The PVC stays in `Pending`
until the scheduler picks a node for the pod. At that point, the provisioner
creates the volume in the same availability zone as the chosen node.

### 3. Why is WaitForFirstConsumer preferred for zone-aware storage?

AWS EBS volumes are **zone-locked** -- an EBS volume in `us-east-1a` cannot be
attached to a node in `us-east-1b`. With `Immediate` mode, the volume is created
in a random zone before any pod is scheduled. If the pod ends up in a different
zone, it cannot use the volume and stays in `ContainerCreating` forever.

`WaitForFirstConsumer` solves this by delaying volume creation until the scheduler
has chosen a node. The volume is then created in the same zone as the node.

### 4. What problem occurs with Immediate mode in a multi-zone cluster?

The volume may be created in a zone where no nodes can run the pod (due to
resource constraints, taints, or affinity rules). The PVC shows `Bound` but
the pod cannot start because the volume is in the wrong zone. The pod is stuck
in `ContainerCreating` with an error like:

```
FailedAttachVolume: Multi-Attach error for volume "pv-xxx"
```

Or:

```
FailedMount: Volume is already attached to a node in a different zone
```

---

## Task 3: Create PVCs and Verify Dynamic Provisioning

```yaml
# Payment DB PVC
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: payment-db-pvc
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: fast-io
  resources:
    requests:
      storage: 20Gi
---
# App Logs PVC
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: app-logs-pvc
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: standard-gp3
  resources:
    requests:
      storage: 50Gi
---
# ML Artifacts PVC
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: ml-artifacts-pvc
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: ml-storage
  resources:
    requests:
      storage: 100Gi
```

```bash
kubectl apply -f pvcs.yaml
kubectl get pvc
```

Expected output (on a real cluster):

```
NAME               STATUS    VOLUME   CAPACITY   ACCESS MODES   STORAGECLASS
app-logs-pvc       Bound     pvc-xx   50Gi       RWO           standard-gp3
ml-artifacts-pvc   Pending                                     ml-storage
payment-db-pvc     Pending                                     fast-io
```

### Observations:

- **`app-logs-pvc` (Immediate):** Status is `Bound`. A PV was created immediately
  because the storageClass uses `Immediate` binding mode.

- **`payment-db-pvc` and `ml-artifacts-pvc` (WaitForFirstConsumer):** Status is
  `Pending`. The PVs are not created yet because no pod references them. The PVCs
  are waiting for a pod to trigger scheduling.

```bash
kubectl get pv
# Only one PV exists (for app-logs-pvc)
```

**Why this works:** The provisioner responds to the binding mode. `Immediate`
triggers provisioning on PVC creation. `WaitForFirstConsumer` defers provisioning
until the kube-scheduler assigns the pod to a node.

---

## Task 4: Deploy Pods to Trigger Provisioning

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: payment-test
spec:
  containers:
  - name: writer
    image: busybox
    command: ["sh", "-c", "echo 'payment data' > /data/test.txt && sleep 3600"]
    volumeMounts:
    - mountPath: /data
      name: storage
  volumes:
  - name: storage
    persistentVolumeClaim:
      claimName: payment-db-pvc
```

```bash
kubectl apply -f payment-test.yaml
kubectl get pods payment-test
```

Expected output:

```
NAME            READY   STATUS    RESTARTS   AGE
payment-test    1/1     Running   0          30s
```

Check PVC and PV status:

```bash
kubectl get pvc payment-db-pvc
```

```
NAME              STATUS   VOLUME                                     CAPACITY
payment-db-pvc    Bound    pvc-a1b2c3d4-e5f6-7890-abcd-ef1234567890   20Gi
```

```bash
kubectl get pv
```

```
NAME                                       CAPACITY   RECLAIM POLICY
pvc-a1b2c3d4-e5f6-7890-abcd-ef1234567890   20Gi       Retain
pvc-x1y2z3w4-a5b6-c789-def0-123456789abc   50Gi       Delete
```

Verify data:

```bash
kubectl exec payment-test -- cat /data/test.txt
# Output: payment data
```

**Why this works:** When the pod is scheduled, Kubernetes:
1. Selects a node in a specific availability zone.
2. The `WaitForFirstConsumer` provisioner creates the EBS volume in that zone.
3. The volume is attached to the node.
4. The PVC transitions to `Bound`.
5. The pod starts and writes data to the mounted volume.

---

## Task 5: Verify Reclaim Policy Behavior

### Step 1-3: Delete PVC with Retain policy

```bash
kubectl delete pod payment-test
kubectl delete pvc payment-db-pvc
kubectl get pv
```

Expected output:

```
NAME                                       CAPACITY   RECLAIM POLICY   STATUS
pvc-a1b2c3d4-e5f6-7890-abcd-ef1234567890   20Gi       Retain           Released
```

The PV status is `Released`, not `Deleted`. The underlying EBS volume still exists.
An admin must manually decide what to do:

```bash
# Option A: Keep the data -- edit the PV to remove the claimRef
kubectl patch pv pvc-a1b2c3d4 -p '{"spec":{"claimRef":null}}'
# PV becomes Available again

# Option B: Delete the PV and the EBS volume
kubectl delete pv pvc-a1b2c3d4
```

### Step 4-5: Delete PVC with Delete policy

```bash
kubectl delete pvc app-logs-pvc
kubectl get pv
```

The PV that was bound to `app-logs-pvc` is **gone**. It does not appear in
`kubectl get pv` output. The underlying EBS volume was automatically deleted.

### Step 6: Explanation

| Policy | PVC deleted | PV status | EBS volume | Manual cleanup needed? |
|--------|------------|-----------|------------|----------------------|
| Retain | PV becomes `Released` | Released | Preserved | Yes -- admin must clean up or reclaim |
| Delete | PV is deleted | Gone | Deleted | No -- automatic cleanup |

**Retain** is for production data you cannot afford to lose. **Delete** is for
ephemeral data (logs, caches, dev environments) where automatic cleanup reduces
operational overhead.

---

## Common Mistakes

1. **Using Immediate mode with zone-locked storage.** This is the most common
   StorageClass misconfiguration on AWS. The volume is created in a random zone,
   and the pod may be scheduled in a different zone. Always use
   `WaitForFirstConsumer` for zone-attached storage (EBS, Persistent Disk).

2. **Not enabling volume expansion.** If `allowVolumeExpansion` is `false` (the
   default), you cannot grow a PVC after creation. The only option is to create a
   new PVC, migrate data, and update the pod. Always set `allowVolumeExpansion: true`
   for production StorageClasses.

3. **Setting Delete policy for databases.** If a developer accidentally deletes a
   PVC, the Delete policy destroys the underlying volume and all data. For databases
   and stateful applications, always use `Retain`. The extra manual cleanup step is
   a safety net.

4. **Forgetting that PVCs without storageClassName use the default class.** If your
   cluster has a default StorageClass, any PVC without an explicit
   `storageClassName` uses it. This can lead to unintended provisioning on the wrong
   storage tier.

5. **Not testing StorageClasses before production.** StorageClass parameters are
   passed to the CSI driver and are not validated by the Kubernetes API server.
   Invalid parameters (e.g., a wrong `type` value) cause provisioning failures at
   runtime, not at apply time.
