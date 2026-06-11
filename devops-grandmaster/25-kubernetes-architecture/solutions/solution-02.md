# Solution 02: Inspect the Control Plane

## Part A: Control Plane Pods

Output of `kubectl get pods -n kube-system` on a kind cluster:

```
NAME                                         READY   STATUS    RESTARTS   AGE
coredns-5dd5756b68-abc12                     1/1     Running   0          5m
coredns-5dd5756b68-def34                     1/1     Running   0          5m
etcd-kind-control-plane                      1/1     Running   0          5m
kindnet-xyz12                                1/1     Running   0          5m
kube-apiserver-kind-control-plane            1/1     Running   0          5m
kube-controller-manager-kind-control-plane   1/1     Running   0          5m
kube-proxy-abc12                             1/1     Running   0          5m
kube-scheduler-kind-control-plane            1/1     Running   0          5m
```

### Answers

1. **How many pods:** Typically 8 pods in `kube-system` on a kind cluster.
2. **Control plane pods:** etcd, kube-apiserver, kube-scheduler, kube-controller-manager.
3. **Other pods:**
   - CoreDNS (x2) -- provides DNS resolution for services and pods
   - kindnet -- CNI plugin that provides pod networking (specific to kind)
   - kube-proxy -- manages iptables/IPVS rules for Service load balancing
4. **Status:** All pods should be `Running`. If any show `CrashLoopBackOff`
   or `Error`, investigate with `kubectl logs <pod-name> -n kube-system`.

### Why This Matters

The control plane runs as static pods managed by the kubelet on the
control plane node. They are defined in `/etc/kubernetes/manifests/` and
the kubelet restarts them automatically if they crash. This is different
from regular pods managed by a Deployment.

## Part B: API Server Inspection

Typical `describe kube-apiserver` findings:

### Image
```
Image: registry.k8s.io/kube-apiserver:v1.28.0
```

### Key Flags
```
--secure-port=6443
--etcd-servers=https://127.0.0.1:2379
--authorization-mode=Node,RBAC
--client-ca-file=/etc/kubernetes/pki/ca.crt
--service-account-key-file=/etc/kubernetes/pki/sa.pub
--enable-admission-plugins=NodeRestriction,ServiceAccount
```

### Resource Requests
```
Requests:
  cpu: 250m
  memory: 100Mi
```

### Why This Matters

- **`--secure-port=6443`** is the HTTPS port. All kubectl traffic goes
  through this port. It is always HTTPS -- the API server does not
  accept plain HTTP.
- **`--etcd-servers`** shows the API server talks to etcd on localhost
  (single-node etcd). In HA setups, this would list multiple etcd endpoints.
- **`--authorization-mode=Node,RBAC`** means the API server first checks
  if the request is from a node (kubelet), then falls back to RBAC
  (Role-Based Access Control) rules.
- **`--client-ca-file`** is the CA certificate used to verify client
  certificates. Kubernetes uses mutual TLS (mTLS) for component
  authentication.

## Part C: etcd Inspection

### Image
```
Image: registry.k8s.io/etcd:3.5.9-0
```

### Key Flags
```
--data-dir=/var/lib/etcd
--listen-client-urls=https://127.0.0.1:2379
--advertise-client-urls=https://127.0.0.1:2379
--listen-peer-urls=https://127.0.0.1:2380
--client-cert-auth=true
```

### Ports
- **2379** -- Client communication (API server talks to etcd here)
- **2380** -- Peer communication (etcd nodes talk to each other here)

### Why This Matters

- **`--data-dir=/var/lib/etcd`** is where all cluster state is stored.
  If this directory is lost, the cluster loses everything. This is why
  etcd backups are critical and why etcd should use a dedicated SSD.
- **`--listen-client-urls=127.0.0.1`** means etcd only accepts connections
  from localhost. Only the API server on the same machine can reach it.
  In HA setups, etcd listens on all interfaces and uses TLS client
  certificates for authentication.
- **Port 2380** is for etcd-to-etcd communication in a cluster. With a
  single etcd node, this port is open but unused.

## Part D: Scheduler and Controller Manager

### Scheduler Key Flags
```
--leader-elect=true
--kubeconfig=/etc/kubernetes/scheduler.conf
```

### Controller Manager Key Flags
```
--leader-elect=true
--kubeconfig=/etc/kubernetes/controller-manager.conf
--service-cluster-ip-range=10.96.0.0/12
--cluster-cidr=10.244.0.0/16
```

### What `--leader-elect` Means

When `--leader-elect=true` is set:
- Multiple instances of the component can run simultaneously
- Only one is the "leader" and actively making decisions
- The others are in standby mode, watching the leader
- If the leader fails, a standby is elected as the new leader
- This prevents conflicting decisions (e.g., two schedulers assigning
  the same pod to different nodes)

### What `--kubeconfig` Does

The kubeconfig file contains:
- The API server endpoint URL
- The client certificate for authentication
- The CA certificate to verify the API server

Each component uses its own kubeconfig to authenticate to the API
server. This is how the scheduler and controller manager can watch
for and update cluster objects.

### Why Only Scheduler and Controller Manager Need Leader Election

The API server is **stateless** -- it processes requests and returns
results. Running multiple API servers behind a load balancer is safe
because any instance can handle any request.

The scheduler and controller manager are **stateful decision-makers**.
Two schedulers running simultaneously could assign the same pod to
two different nodes. Two controller managers could create duplicate
pods. Leader election ensures only one makes decisions at a time.

## Part E: Component Health

### API Server Health Check
```
kubectl get --raw='/readyz?verbose'
```

Expected output:
``[+]ping ok
[+]log ok
[+]etcd ok
[+]poststarthook/start-api-server-informers ok
[+]poststarthook/generic-apiserver-start-informers ok
[+]poststarthook/priority-and-fairness-filter ok
[+]poststarthook/bootstrap-controller ok
[+]poststarthook/rbac/bootstrap-roles ok
[+]poststarthook/scheduling/bootstrap-system-priority-classes ok
[+]poststarthook/start-system-namespaces-controller ok
[+]poststarthook/bootstrap-controller ok
[+]poststarthook/aggregator-reload-proxy-client-cert ok
[+]poststarthook/start-kube-aggregator-informers ok
[+]poststarthook/apiservice-registration-controller ok
[+]poststarthook/apiservice-status-available-controller ok
[+]poststarthook/kube-apiserver-autoregistration ok
[+]autoregister-completion ok
[+]poststarthook/apiservice-openapi-controller ok
readyz check passed``

### Component Status
```
kubectl get componentstatuses
```

Expected output (may show deprecation warning):
```
Warning: v1 ComponentStatus is deprecated in v1.19+
NAME                 STATUS    MESSAGE             ERROR
scheduler            Healthy   ok
controller-manager   Healthy   ok
etcd-0               Healthy   {"health":"true"}
```

### Why Component Status May Be Deprecated

`kubectl get componentstatuses` (or `kubectl get cs`) was deprecated
because it only checks the health of components that expose a `/healthz`
endpoint on a specific port. Modern Kubernetes prefers:
- Checking pod status in `kube-system`
- Using the `/readyz` endpoint on the API server
- Using monitoring tools (Prometheus, Grafana)

## Part F: Summary Table

| Component | Image | Port | Key Flag | Status |
|-----------|-------|------|----------|--------|
| API Server | registry.k8s.io/kube-apiserver:v1.28.0 | 6443 | `--authorization-mode=Node,RBAC` | Running |
| etcd | registry.k8s.io/etcd:3.5.9-0 | 2379, 2380 | `--data-dir=/var/lib/etcd` | Running |
| Scheduler | registry.k8s.io/kube-scheduler:v1.28.0 | 10259 | `--leader-elect=true` | Running |
| Controller Manager | registry.k8s.io/kube-controller-manager:v1.28.0 | 10257 | `--leader-elect=true` | Running |

### Common Mistakes

- **Confusing ports.** The API server uses 6443 (HTTPS). etcd uses 2379
  (client) and 2380 (peer). The scheduler uses 10259. The controller
  manager uses 10257. These are the default ports -- they can be changed
  with flags.
- **Not checking the image version.** The image version tells you the
  exact Kubernetes version running. This is important for compatibility
  checks and upgrade planning.
- **Ignoring the `--leader-elect` flag.** This flag is critical for HA.
  If it is `false` in a multi-node control plane, you will have split-brain
  problems.
- **Assuming `componentstatuses` is the only health check.** It is
  deprecated and does not check all components. Use pod status and the
  `/readyz` endpoint instead.

## Relevant README Sections

- [API Server (kube-apiserver)](../README.md#api-server-kube-apiserver) -- The front door
- [etcd](../README.md#etcd) -- The brain
- [Scheduler (kube-scheduler)](../README.md#scheduler-kube-scheduler) -- Node selection
- [Controller Manager (kube-controller-manager)](../README.md#controller-manager-kube-controller-manager) -- Reconciliation loops
