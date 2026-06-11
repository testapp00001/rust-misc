# Module 32: Persistent Volumes — Stateful Workloads in Kubernetes

## The Problem: Pods Are Ephemeral

Every pod in Kubernetes is **disposable**. When a pod dies, everything inside it is gone — including your data.

```
Pod starts → writes data to local filesystem → pod crashes
                                                    ↓
                                            ALL DATA IS LOST
                                                    ↓
                                            New pod starts empty
```

This is fine for stateless apps (web servers, API gateways). But the moment you run:
- A database (PostgreSQL, MySQL, MongoDB)
- A message queue (RabbitMQ, Kafka)
- A file-processing service
- An Elasticsearch cluster

...you need data to **survive pod restarts, rescheduling, and node failures**.

### The Core Tension

Kubernetes wants to treat pods like cattle, not pets. Databases want to be pets with stable identities and persistent storage. Persistent Volumes bridge this gap.

## The Naive Way: hostPath Volumes

The simplest approach: mount a directory from the host node directly into the pod.

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: naive-db
spec:
  containers:
  - name: postgres
    image: postgres:16
    volumeMounts:
    - mountPath: /var/lib/postgresql/data
      name: db-storage
  volumes:
  - name: db-storage
    hostPath:
      path: /data/postgres
      type: DirectoryOrCreate
```

**Why this fails in production:**

```
Node A (pod running here)
├── /data/postgres/  ← data lives HERE on Node A's disk
│
Node B
├── /data/postgres/  ← empty, different disk entirely

Pod gets rescheduled to Node B → ALL DATA IS GONE
```

- Data is tied to a specific node
- If the node dies, data is gone
- No backup, no replication, no portability
- No way to request storage dynamically
- Security: any pod on the node could access the path

## The Right Way: PersistentVolumes and PersistentVolumeClaims

Kubernetes separates storage into two roles:

```
┌──────────────────────────────────────────────────────┐
│                  Storage Provisioning                   │
│                                                         │
│  Cluster Admin (infrastructure)                         │
│  ┌─────────────────────────────────┐                   │
│  │     PersistentVolume (PV)        │                   │
│  │  "Here is 100GB of fast SSD      │                   │
│  │   storage on the SAN"            │                   │
│  └──────────────┬──────────────────┘                   │
│                 │ bound                                  │
│  ┌──────────────▼──────────────────┐                   │
│  │  PersistentVolumeClaim (PVC)     │                   │
│  │  "I need 10GB of fast storage"   │                   │
│  └──────────────┬──────────────────┘                   │
│                 │ mounted                                │
│  ┌──────────────▼──────────────────┐                   │
│  │         Pod / Container          │                   │
│  │  "Mount at /var/lib/pgdata"      │                   │
│  └─────────────────────────────────┘                   │
│                                                         │
│  Developer (app consumer)                               │
└──────────────────────────────────────────────────────┘
```

### PersistentVolume (PV) — The Storage Resource

A PV is a **cluster-level resource** that represents actual storage. It is provisioned by an admin (or dynamically by a StorageClass).

```yaml
apiVersion: v1
kind: PersistentVolume
metadata:
  name: fast-ssd-pv
spec:
  capacity:
    storage: 100Gi
  accessModes:
  - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  storageClassName: fast
  hostPath:
    path: /mnt/data
```

### PersistentVolumeClaim (PVC) — The Storage Request

A PVC is a **namespace-level request** for storage. It is what developers create to consume storage.

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: postgres-pvc
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
  storageClassName: fast
```

### Using a PVC in a Pod

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: postgres
spec:
  containers:
  - name: postgres
    image: postgres:16
    env:
    - name: POSTGRES_PASSWORD
      value: "secretpassword"
    - name: PGDATA
      value: /var/lib/postgresql/data/pgdata
    volumeMounts:
    - mountPath: /var/lib/postgresql/data
      name: pgdata
  volumes:
  - name: pgdata
    persistentVolumeClaim:
      claimName: postgres-pvc
```

The flow:
1. Developer creates a PVC requesting 10Gi of storage
2. Kubernetes finds a PV that matches (capacity, access mode, storage class)
3. The PVC is **bound** to the PV
4. The pod mounts the PVC, and the storage is available inside the container

## Access Modes: Who Can Read and Write?

Access modes control **how many nodes can use the volume simultaneously**:

```
┌─────────────────────┬────────────────────────────────────────────┐
│ Access Mode          │ Description                                │
├─────────────────────┼────────────────────────────────────────────┤
│ ReadWriteOnce (RWO)  │ Single node can read/write                 │
│                      │ Most common for databases                  │
│                      │ Example: EBS, Persistent Disk              │
├─────────────────────┼────────────────────────────────────────────┤
│ ReadOnlyMany (ROX)   │ Many nodes can read, nobody writes         │
│                      │ Good for shared config, static assets      │
│                      │ Example: NFS, Azure File (read-only)       │
├─────────────────────┼────────────────────────────────────────────┤
│ ReadWriteMany (RWX)  │ Many nodes can read/write simultaneously   │
│                      │ Needed for shared filesystems              │
│                      │ Example: NFS, CephFS, Azure File           │
├─────────────────────┼────────────────────────────────────────────┤
│ ReadWriteOncePod     │ Single POD (not node) can read/write       │
│                      │ K8s 1.22+, strictest mode                  │
│                      │ Example: CSI drivers with fencing          │
└─────────────────────┴────────────────────────────────────────────┘
```

**Critical distinction:** RWO means one **node**, not one pod. Multiple pods on the same node can share an RWO volume. This matters for StatefulSets (Module 33).

```yaml
# A PVC requesting ReadWriteMany (shared filesystem)
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: shared-data
spec:
  accessModes:
  - ReadWriteMany
  resources:
    requests:
      storage: 50Gi
  storageClassName: nfs-csi
```

## Reclaim Policies: What Happens When You Delete a PVC?

When a PVC is deleted, what happens to the underlying PV?

```
┌────────────────────┬──────────────────────────────────────────┐
│ Reclaim Policy      │ Behavior                                 │
├────────────────────┼──────────────────────────────────────────┤
│ Retain (default)    │ PV is kept, data persists                │
│ for static PVs      │ Admin must manually clean up             │
│                     │ PV status becomes "Released"             │
│                     │ Safe for production data                 │
├────────────────────┼──────────────────────────────────────────┤
│ Delete              │ PV and underlying storage are deleted    │
│ for dynamic PVs     │ Automatic cleanup                        │
│                     │ Data is GONE                             │
│                     │ Good for dev/test environments           │
├────────────────────┼──────────────────────────────────────────┤
│ Recycle             │ scrub: rm -rf /volume/*                  │
│ (deprecated)        │ PV becomes Available again               │
│                     │ Dangerous, do not use                    │
└────────────────────┴──────────────────────────────────────────┘
```

```yaml
# PV with Retain policy — data survives PVC deletion
apiVersion: v1
kind: PersistentVolume
metadata:
  name: critical-data-pv
spec:
  capacity:
    storage: 50Gi
  persistentVolumeReclaimPolicy: Retain    # <-- data survives
  accessModes:
  - ReadWriteOnce
  storageClassName: standard
  awsElasticBlockStore:
    volumeID: vol-0abc123def456
    fsType: ext4
```

## StorageClass: Dynamic Provisioning

Manually creating PVs for every database is tedious. **StorageClass** automates this.

```
Without StorageClass (manual):
  Admin creates PV → Developer creates PVC → Binding happens

With StorageClass (dynamic):
  Developer creates PVC → StorageClass automatically creates PV → Binding happens
```

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: ebs.csi.aws.com        # AWS EBS CSI driver
parameters:
  type: gp3
  iops: "3000"
  throughput: "125"
  encrypted: "true"
reclaimPolicy: Delete
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer   # don't create until a pod needs it
```

Key StorageClass fields:

```
┌─────────────────────────┬───────────────────────────────────────────────┐
│ Field                    │ Purpose                                       │
├─────────────────────────┼───────────────────────────────────────────────┤
│ provisioner              │ Which CSI driver creates volumes              │
│ parameters               │ Cloud-specific settings (type, encryption)    │
│ reclaimPolicy            │ Retain or Delete when PVC is deleted          │
│ allowVolumeExpansion     │ Can the volume grow after creation?            │
│ volumeBindingMode        │ Immediate or WaitForFirstConsumer             │
│ mountOptions             │ Filesystem mount options                      │
└─────────────────────────┴───────────────────────────────────────────────┘
```

A PVC references a StorageClass:

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: app-data
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: fast-ssd          # <-- references the StorageClass
  resources:
    requests:
      storage: 20Gi
```

The moment this PVC is created, the `fast-ssd` StorageClass provisions a new EBS volume automatically.

## The Production Way: Cloud Storage Integration

### AWS — EBS CSI Driver

```bash
# Install the EBS CSI driver (if not already installed)
kubectl apply -k "github.com/kubernetes-sigs/aws-ebs-csi-driver/deploy/kubernetes/overlays/stable/?ref=release-1.28"
```

```yaml
# StorageClass for gp3 EBS volumes
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: gp3
  annotations:
    storageclass.kubernetes.io/is-default-class: "true"
provisioner: ebs.csi.aws.com
parameters:
  type: gp3
  fsType: ext4
  encrypted: "true"
reclaimPolicy: Delete
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

### GCP — Persistent Disk CSI Driver

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: ssd-storage
provisioner: pd.csi.storage.gke.io
parameters:
  type: pd-ssd
  replication-type: regional-pd       # replicate across zones
reclaimPolicy: Retain
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

### Azure — Azure Disk CSI Driver

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: managed-premium
provisioner: disk.csi.azure.com
parameters:
  skuName: Premium_LRS
  cachingmode: ReadOnly
  kind: Managed
reclaimPolicy: Delete
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
```

### Volume Expansion in Production

Storage needs grow. With `allowVolumeExpansion: true`, you can resize without recreating:

```bash
# Edit the PVC to request more storage
kubectl patch pvc postgres-pvc -p '{"spec":{"resources":{"requests":{"storage":"50Gi"}}}}'

# Check expansion status
kubectl get pvc postgres-pvc
# Status will show FileSystemResizePending until the pod restarts

# Restart the pod to complete filesystem resize
kubectl delete pod postgres-0
```

## Volume Lifecycle States

```
PV Lifecycle:
  Available → Bound → Released → (Retain) → Available (after admin intervention)
                                → (Delete) → Deleted

PVC Lifecycle:
  Pending → Bound → (deleted by user) → PV released
```

```
┌─────────────┐     claim     ┌─────────────┐     delete PVC    ┌─────────────┐
│  Available   │──────────────▶│    Bound     │─────────────────▶│  Released    │
│  (no claim)  │               │ (in use)     │                   │ (data kept   │
└─────────────┘               └─────────────┘                   │  if Retain)  │
                                                                  └──────┬──────┘
                                                                         │
                                                         ┌───────────────┤
                                                         ▼               ▼
                                                   ┌──────────┐   ┌──────────┐
                                                   │ Available │   │  Deleted  │
                                                   │ (cleaned) │   │ (gone)    │
                                                   └──────────┘   └──────────┘
```

## Hands-On Lab: Mount a PV to a Pod

### Lab 1: Static Provisioning

```bash
# Create a PV manually
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolume
metadata:
  name: lab-pv
  labels:
    type: local
spec:
  capacity:
    storage: 5Gi
  accessModes:
  - ReadWriteOnce
  persistentVolumeReclaimPolicy: Retain
  storageClassName: manual
  hostPath:
    path: "/mnt/lab-data"
EOF

# Verify the PV
kubectl get pv lab-pv
# STATUS should be Available

# Create a PVC that binds to it
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: lab-pvc
spec:
  accessModes:
  - ReadWriteOnce
  storageClassName: manual
  resources:
    requests:
      storage: 3Gi
EOF

# Verify binding
kubectl get pvc lab-pvc
# STATUS should be Bound
kubectl get pv lab-pv
# STATUS should be Bound (was Available)

# Create a pod that uses the PVC
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Pod
metadata:
  name: writer-pod
spec:
  containers:
  - name: writer
    image: busybox
    command: ["sh", "-c", "echo 'Hello from PV!' > /data/hello.txt && sleep 3600"]
    volumeMounts:
    - mountPath: /data
      name: persistent-storage
  volumes:
  - name: persistent-storage
    persistentVolumeClaim:
      claimName: lab-pvc
EOF

# Verify data was written
kubectl exec writer-pod -- cat /data/hello.txt
# Output: Hello from PV!

# Delete the pod
kubectl delete pod writer-pod

# Create a new pod with the same PVC
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: Pod
metadata:
  name: reader-pod
spec:
  containers:
  - name: reader
    image: busybox
    command: ["sh", "-c", "cat /data/hello.txt && sleep 3600"]
    volumeMounts:
    - mountPath: /data
      name: persistent-storage
  volumes:
  - name: persistent-storage
    persistentVolumeClaim:
      claimName: lab-pvc
EOF

# Data survives pod restart!
kubectl logs reader-pod
# Output: Hello from PV!

# Cleanup
kubectl delete pod reader-pod
kubectl delete pvc lab-pvc
kubectl delete pv lab-pv
```

### Lab 2: Dynamic Provisioning with StorageClass

```bash
# If using kind, check available StorageClasses
kubectl get storageclass

# If using minikube, the default StorageClass is already provisioned
# If on a cloud provider, you'll have cloud-specific StorageClasses

# Create a PVC (PV is created automatically)
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: dynamic-pvc
spec:
  accessModes:
  - ReadWriteOnce
  resources:
    requests:
      storage: 2Gi
EOF

# Watch the PV get created
kubectl get pv,pvc

# Deploy a simple web app with persistent storage
cat <<EOF | kubectl apply -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: file-server
spec:
  replicas: 1
  selector:
    matchLabels:
      app: file-server
  template:
    metadata:
      labels:
        app: file-server
    spec:
      containers:
      - name: nginx
        image: nginx:alpine
        volumeMounts:
        - mountPath: /usr/share/nginx/html
          name: web-content
      volumes:
      - name: web-content
        persistentVolumeClaim:
          claimName: dynamic-pvc
EOF

# Write content into the volume
kubectl exec deploy/file-server -- sh -c "echo '<h1>Persistent Content</h1>' > /usr/share/nginx/html/index.html"

# Access the content
kubectl port-forward deploy/file-server 8080:80
# Visit http://localhost:8080

# Delete the deployment and recreate — data persists
kubectl delete deployment file-server
kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: file-server
spec:
  replicas: 1
  selector:
    matchLabels:
      app: file-server
  template:
    metadata:
      labels:
        app: file-server
    spec:
      containers:
      - name: nginx
        image: nginx:alpine
        volumeMounts:
        - mountPath: /usr/share/nginx/html
          name: web-content
      volumes:
      - name: web-content
        persistentVolumeClaim:
          claimName: dynamic-pvc
EOF

# Content is still there
kubectl port-forward deploy/file-server 8080:80
```

### Lab 3: Inspect Storage Resources

```bash
# List all PVs with details
kubectl get pv -o wide

# List all PVCs
kubectl get pvc -A

# Describe a PV for full details (bound PVC, reclaim policy, etc.)
kubectl describe pv <pv-name>

# Describe a PVC (bound PV, access mode, status)
kubectl describe pvc <pvc-name>

# Check StorageClasses
kubectl get storageclass

# See which pods are using which PVCs
kubectl get pods -o custom-columns=\
  NAME:.metadata.name,\
  PVC:.spec.volumes[*].persistentVolumeClaim.claimName
```

## Limitation: You Have Storage, But Databases Need Stable Identity

You can now persist data across pod restarts. But databases like PostgreSQL need more:
- A **stable hostname** (db-0, db-1, not random-xyz-abc)
- **Ordered** startup (primary before replicas)
- **Stable storage** that follows the pod (not shared between pods)
- **Network identity** that clients can depend on

A Deployment gives you random pod names, random scaling order, and shared PVCs. That breaks databases.

**Next problem:** How do you run stateful applications that need identity, ordering, and per-pod storage?

→ **Next module:** [33-statefulsets](../33-statefulsets/) — Databases and stateful services

## Checklist

- [ ] I understand why pods are ephemeral and why that breaks stateful apps
- [ ] I can explain the difference between PV, PVC, and StorageClass
- [ ] I know the four access modes and when to use each
- [ ] I understand reclaim policies (Retain vs Delete) and their consequences
- [ ] I can provision storage dynamically with a StorageClass
- [ ] I can mount a PVC to a pod and verify data survives restarts
- [ ] I know how to expand a volume when storage needs grow
- [ ] I understand the PV lifecycle states (Available, Bound, Released, Deleted)
