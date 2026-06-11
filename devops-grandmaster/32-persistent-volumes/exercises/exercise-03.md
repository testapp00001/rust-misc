# Exercise 03: Dynamic Provisioning with StorageClasses -- Let the Cluster Handle It

**Type:** Independent
**Time:** 40 minutes
**Difficulty:** Medium

## Objective

Define StorageClasses with different characteristics, use them to dynamically
provision PersistentVolumes, and understand how volumeBindingMode, reclaimPolicy,
and allowVolumeExpansion affect production workloads.

---

## Background

Your company is launching three new services on an AWS EKS cluster. Each has
different storage requirements:

| Service | IOPS Need | Durability | Expected Growth | Access |
|---------|-----------|------------|-----------------|--------|
| Payment database | High (io1) | Must survive PVC deletion | Moderate | Single node |
| Application logs | Low (gp3) | Can be deleted | High | Single node |
| Machine learning artifacts | Medium (gp3) | Must survive PVC deletion | Moderate | Single node |

You need to create StorageClasses for each and verify dynamic provisioning works.

---

## Tasks

### Task 1: Design Three StorageClasses

Create three StorageClass manifests with the following specifications:

**Class 1: `fast-io`** (for the payment database)
- Provisioner: `ebs.csi.aws.com`
- Parameters: `type: io1`, `iopsPerGB: "50"`
- Reclaim policy: `Retain`
- Allow volume expansion: `true`
- Volume binding mode: `WaitForFirstConsumer`

**Class 2: `standard-gp3`** (for application logs)
- Provisioner: `ebs.csi.aws.com`
- Parameters: `type: gp3`
- Reclaim policy: `Delete`
- Allow volume expansion: `true`
- Volume binding mode: `Immediate`

**Class 3: `ml-storage`** (for ML artifacts)
- Provisioner: `ebs.csi.aws.com`
- Parameters: `type: gp3`, `encrypted: "true"`
- Reclaim policy: `Retain`
- Allow volume expansion: `true`
- Volume binding mode: `WaitForFirstConsumer`

Write all three YAML manifests.

<details>
<summary>Hint 1: StorageClass YAML template</summary>

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: <name>
provisioner: ebs.csi.aws.com
parameters:
  type: <disk-type>
reclaimPolicy: <Retain|Delete>
allowVolumeExpansion: <true|false>
volumeBindingMode: <Immediate|WaitForFirstConsumer>
```

</details>

### Task 2: Understand volumeBindingMode

Answer these questions about `Immediate` vs `WaitForFirstConsumer`:

1. When does `Immediate` mode create the underlying cloud disk?
2. When does `WaitForFirstConsumer` mode create the underlying cloud disk?
3. Why is `WaitForFirstConsumer` preferred for zone-aware storage (like EBS)?
4. What problem can occur if you use `Immediate` with a multi-zone cluster?

<details>
<summary>Hint 2: Zone affinity</summary>

AWS EBS volumes are tied to a single availability zone. If Kubernetes creates the
volume in zone `us-east-1a` but the pod is scheduled in zone `us-east-1b`, the
volume cannot be attached. `WaitForFirstConsumer` delays creation until the scheduler
picks a node, so the volume is created in the same zone as the pod.

</details>

### Task 3: Create PVCs and Verify Dynamic Provisioning

Create a PVC for each service:

| PVC Name | StorageClass | Size |
|----------|-------------|------|
| `payment-db-pvc` | `fast-io` | 20Gi |
| `app-logs-pvc` | `standard-gp3` | 50Gi |
| `ml-artifacts-pvc` | `ml-storage` | 100Gi |

After applying all PVCs:

1. Check PVC status: `kubectl get pvc`.
2. Check if PVs were automatically created: `kubectl get pv`.
3. For the `standard-gp3` PVC (Immediate mode), verify the PV exists even without
   a pod.
4. For the `fast-io` and `ml-storage` PVCs (WaitForFirstConsumer), check if the
   PVC is still Pending. Explain why.

<details>
<summary>Hint 3: Checking provisioning status</summary>

```bash
kubectl get pvc -o wide
kubectl get pv
kubectl describe pvc payment-db-pvc   # Check events
```

For WaitForFirstConsumer PVCs, the events will show that volume creation is
waiting for a pod to use the claim.

</details>

### Task 4: Deploy Pods to Trigger Provisioning

Create a simple busybox Pod that mounts `payment-db-pvc` at `/data`:

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

After applying:
1. Verify the pod is Running.
2. Verify the PVC is now Bound.
3. Verify a PV was dynamically created.
4. Read the data: `kubectl exec payment-test -- cat /data/test.txt`.

<details>
<summary>Hint 4: WaitForFirstConsumer in action</summary>

When the pod is scheduled, Kubernetes:
1. Picks a node (and therefore an availability zone).
2. Tells the StorageClass provisioner to create the volume in that zone.
3. Attaches the volume to the node.
4. Mounts it in the pod.

The PVC transitions from Pending to Bound only after the pod is scheduled.

</details>

### Task 5: Verify Reclaim Policy Behavior

Clean up to observe reclaim policy differences:

1. Delete the `payment-test` pod.
2. Delete the `payment-db-pvc` PVC (which uses `fast-io` with Retain policy).
3. Check the PV status: `kubectl get pv`. What status does it show?
4. Delete the `app-logs-pvc` PVC (which uses `standard-gp3` with Delete policy).
5. Check the PV status: `kubectl get pv`. Is the PV still there?
6. Explain the difference.

<details>
<summary>Hint 5: Retain vs Delete lifecycle</summary>

- **Retain:** PVC deletion changes PV status to `Released`. The PV and underlying
  storage persist. An admin must manually clean up and change the PV back to
  `Available` (or delete and recreate it).
- **Delete:** PVC deletion triggers automatic deletion of the PV and the underlying
  cloud disk. The storage is gone.

</details>

---

## Success Criteria

- [ ] Three StorageClasses created with correct parameters and policies.
- [ ] PVCs are created and dynamically provision PVs.
- [ ] WaitForFirstConsumer PVCs remain Pending until a Pod uses them.
- [ ] Immediate PVC provisions a PV immediately.
- [ ] Retain policy keeps the PV after PVC deletion.
- [ ] Delete policy removes the PV after PVC deletion.

---

## Hints

<details>
<summary>Hint 6: Setting a default StorageClass</summary>

You can mark one StorageClass as the default:

```yaml
metadata:
  name: standard-gp3
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
```

PVCs without an explicit `storageClassName` will use the default. This is convenient
but can lead to unintended provisioning if developers forget to specify the class.

</details>

<details>
<summary>Hint 7: Volume expansion</summary>

With `allowVolumeExpansion: true`, you can grow a PVC after creation:

```bash
kubectl patch pvc payment-db-pvc -p '{"spec":{"resources":{"requests":{"storage":"50Gi"}}}}'
```

The underlying cloud disk is expanded. The filesystem resize happens either
automatically or after the pod restarts, depending on the CSI driver.

</details>
