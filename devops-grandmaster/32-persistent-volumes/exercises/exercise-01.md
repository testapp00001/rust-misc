# Exercise 01: The Storage Abstraction Puzzle -- PV vs PVC and the Binding Process

**Type:** Conceptual
**Time:** 20 minutes
**Difficulty:** Beginner

## Objective

Understand the relationship between PersistentVolumes (PVs), PersistentVolumeClaims
(PVCs), and StorageClasses. Explain the binding process and why Kubernetes separates
storage provisioning from storage consumption.

---

## Background

Your team is migrating a PostgreSQL database from a single-server setup to Kubernetes.
The database needs 50Gi of SSD-backed storage that survives pod restarts and node
failures. Three team members propose different approaches:

| Person | Approach |
|--------|----------|
| Dev A | "Just use a `hostPath` volume. It's simple." |
| Dev B | "Create a PV and PVC, then mount the PVC in the Pod." |
| Dev C | "Let the cloud provider handle it with a StorageClass." |

You need to evaluate each approach and explain the Kubernetes storage model.

---

## Tasks

### Task 1: The Three-Layer Model

Fill in the table describing each storage layer in Kubernetes.

| Concept | Scope (cluster / namespace) | Who creates it? | What it represents |
|---------|----------------------------|-----------------|-------------------|
| PersistentVolume (PV) | ? | ? | ? |
| PersistentVolumeClaim (PVC) | ? | ? | ? |
| StorageClass | ? | ? | ? |

### Task 2: The Binding Process

Describe what happens at each step when a developer creates a PVC that references
a StorageClass. Fill in the blanks.

1. Developer creates a PVC requesting _____ storage with _____ access mode.
2. Kubernetes looks for a _____ that matches the PVC's requirements.
3. If using a StorageClass, the _____ automatically creates a new PV.
4. The PVC is _____ to the PV.
5. The PV status changes from _____ to _____.
6. The Pod mounts the PVC via a _____ volume reference.

### Task 3: Why Separate PV from PVC?

Answer the following questions:

1. Why doesn't Kubernetes let developers create PVs directly?
2. What problem does the PVC abstraction solve for application developers?
3. What problem does the PV abstraction solve for cluster administrators?
4. Give one scenario where a manually-created (static) PV is preferred over
   dynamic provisioning.

### Task 4: Evaluate the Approaches

For each approach from the Background table, explain:

1. What happens when the pod is rescheduled to a different node?
2. Is the storage portable across nodes?
3. Is this approach suitable for production?
4. What operational burden does it create?

---

## Success Criteria

- [ ] You can explain the three-layer storage model (PV, PVC, StorageClass).
- [ ] You can describe the binding process from PVC creation to Pod mount.
- [ ] You understand why Kubernetes separates provisioning from consumption.
- [ ] You can evaluate trade-offs of hostPath, static PV, and dynamic provisioning.

---

## Hints

<details>
<summary>Hint 1: Scope Difference</summary>

PVs are **cluster-scoped** -- they do not belong to any namespace. PVCs and Pods
are **namespace-scoped**. A PVC in namespace "dev" can bind to a cluster-scoped PV.
StorageClasses are also cluster-scoped.

</details>

<details>
<summary>Hint 2: Binding Match Criteria</summary>

Kubernetes matches a PVC to a PV based on three criteria:
1. **StorageClassName** must match.
2. **Access modes** must be compatible.
3. **Requested capacity** must be less than or equal to the PV's capacity.

If no PV matches and a StorageClass is specified, the StorageClass provisions one.

</details>

<details>
<summary>Hint 3: Static vs Dynamic</summary>

**Static provisioning:** Admin pre-creates PVs from existing storage (e.g., an NFS
share, a pre-provisioned cloud disk). Good for shared read-only data or when the
storage infrastructure is managed outside Kubernetes.

**Dynamic provisioning:** A StorageClass creates PVs on demand when PVCs are created.
Good for most workloads where storage should be created and destroyed automatically.

</details>

<details>
<summary>Hint 4: hostPath Limitations</summary>

`hostPath` mounts a directory from the **local node's filesystem**. If the pod moves
to another node, it will see a different directory (or nothing at all). It is useful
for single-node development clusters and for kubelet internals, but not for production
data.

</details>
