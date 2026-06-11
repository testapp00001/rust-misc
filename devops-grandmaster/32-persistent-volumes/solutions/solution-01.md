# Solution 01: The Storage Abstraction Puzzle -- PV vs PVC and the Binding Process

---

## Task 1: The Three-Layer Model

| Concept | Scope | Who creates it? | What it represents |
|---------|-------|-----------------|-------------------|
| PersistentVolume (PV) | **Cluster** | Cluster admin (static) or StorageClass (dynamic) | An actual piece of storage -- a cloud disk, NFS share, or local directory |
| PersistentVolumeClaim (PVC) | **Namespace** | Developer / application team | A request for storage -- "I need 10Gi of fast SSD" |
| StorageClass | **Cluster** | Cluster admin | A template that defines how to provision PVs on demand |

**Why this works:** Separating the "what exists" (PV) from "what I need" (PVC) lets
infrastructure teams manage storage independently of application teams. Developers
never need to know which cloud disk or SAN LUN backs their storage.

---

## Task 2: The Binding Process

1. Developer creates a PVC requesting **10Gi** storage with **ReadWriteOnce** access mode.
2. Kubernetes looks for a **PersistentVolume** that matches the PVC's requirements.
3. If using a StorageClass, the **provisioner** automatically creates a new PV.
4. The PVC is **bound** to the PV.
5. The PV status changes from **Available** to **Bound**.
6. The Pod mounts the PVC via a **persistentVolumeClaim** volume reference.

---

## Task 3: Why Separate PV from PVC?

### 1. Why doesn't Kubernetes let developers create PVs directly?

PVs represent actual infrastructure resources -- cloud disks, SAN volumes, NFS shares.
Creating them requires credentials, knowledge of the underlying storage system, and
capacity planning. Developers should not need cloud provider credentials or storage
expertise to request storage for their application.

### 2. What problem does the PVC abstraction solve for application developers?

PVCs let developers request storage in **abstract terms**: "I need X amount of
storage with Y access mode." They do not need to know the provisioner, the cloud
region, the disk type, or the encryption settings. The PVC is a portable request
that works the same way on any cluster.

### 3. What problem does the PV abstraction solve for cluster administrators?

PVs give administrators full control over the storage infrastructure. They can:
- Pre-provision storage from specific backends.
- Set reclaim policies (Retain vs Delete).
- Control which storage is available to which teams.
- Monitor capacity and performance at the infrastructure level.

### 4. Static PV preference scenario

**Shared read-only reference data** (e.g., a large dataset that multiple teams need
to read but nobody modifies). An admin creates a single PV backed by an NFS share
with `ReadOnlyMany` access mode. Multiple PVCs across namespaces bind to it. Dynamic
provisioning would create separate copies for each PVC, wasting storage and requiring
data synchronization.

---

## Task 4: Evaluate the Approaches

### Dev A: hostPath

| Question | Answer |
|----------|--------|
| Pod rescheduled to different node? | **Data is lost.** The new node has an empty or different directory at that path. |
| Portable across nodes? | **No.** Tied to a single node's local filesystem. |
| Production suitable? | **No.** Single point of failure, no backup, no replication. |
| Operational burden | Minimal setup, but requires manual data management, backup scripts, and node affinity rules to keep the pod on the same node. |

### Dev B: PV + PVC (static provisioning)

| Question | Answer |
|----------|--------|
| Pod rescheduled to different node? | **Data survives** if the PV is backed by network storage (NFS, cloud disk). If backed by hostPath, same problem as Dev A. |
| Portable across nodes? | **Yes**, if the PV uses network-attached storage. |
| Production suitable? | **Yes**, for well-understood, long-lived storage. |
| Operational burden | Admin must pre-create PVs, manage capacity, handle cleanup when PVCs are deleted. |

### Dev C: StorageClass (dynamic provisioning)

| Question | Answer |
|----------|--------|
| Pod rescheduled to different node? | **Data survives.** Cloud disks are network-attached and follow the pod. |
| Portable across nodes? | **Yes.** Cloud provider handles attachment to the node running the pod. |
| Production suitable? | **Yes.** Standard approach for cloud-native workloads. |
| Operational burden | Minimal. StorageClass handles provisioning, but admin must define StorageClasses and monitor cloud costs. |

---

## Common Mistakes

1. **Using hostPath for production data.** hostPath is tied to a single node. If
   that node fails, the data is gone. Use hostPath only for single-node development
   clusters or kubelet internals.

2. **Confusing PV scope with PVC scope.** PVs are cluster-scoped; PVCs are
   namespace-scoped. A PVC can only bind to a PV in the same storageClassName
   and with compatible access modes and capacity, regardless of namespace.

3. **Assuming PV and PVC sizes must match exactly.** A PVC requesting 3Gi can
   bind to a PV with 10Gi capacity. The PV's capacity must be **greater than or
   equal to** the PVC's request, not equal.

4. **Forgetting that RWO means one node, not one pod.** Multiple pods on the same
   node can share an RWO volume. This is important for StatefulSets where pods
   are co-located on the same node.

5. **Not setting a reclaim policy.** Default reclaim policy for statically
   provisioned PVs is `Retain`, which is safe. But dynamically provisioned PVs
   default to `Delete` -- deleting the PVC destroys the data. Always set
   `reclaimPolicy: Retain` for production workloads.
