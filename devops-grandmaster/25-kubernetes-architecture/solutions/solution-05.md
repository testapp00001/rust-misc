# Solution 05: Design a Highly Available Control Plane

## Part A: Failure Mode Analysis

| Component Down | Existing Pods | New Deployments | Scaling | Self-Healing | Cluster State |
|----------------|---------------|-----------------|---------|--------------|---------------|
| API Server | Keep running | Blocked | Blocked | Blocked | Frozen (cannot read/write) |
| etcd | Keep running (briefly) | Blocked | Blocked | Blocked | Lost (no state store) |
| Scheduler | Keep running | New pods stuck in Pending | Stuck (pods created but unscheduled) | Works (controller creates pods, scheduler assigns) | Intact |
| Controller Manager | Keep running | Works (API server accepts, scheduler assigns) | Blocked (no one creates/deletes pods) | Blocked (no one detects failures) | Intact |

### Detailed Analysis

**API Server Down:**
- Existing pods keep running because the kubelet caches the last known
  pod spec and manages containers independently.
- New deployments fail because kubectl cannot reach the API server.
- Scaling fails for the same reason.
- Self-healing fails because the controller manager cannot read or write
  through the API server.
- Cluster state is frozen in etcd but inaccessible.

**etcd Down:**
- The API server cannot read or write state. It returns errors for all
  requests.
- Existing pods keep running (kubelet has cached state).
- Everything that requires reading or writing cluster state is blocked.
- If etcd is down permanently, all state is lost.

**Scheduler Down:**
- Existing pods are unaffected.
- New pods are created by the controller manager but stay in `Pending`
  because no scheduler assigns them to nodes.
- Scaling creates pods that cannot be placed.
- Self-healing works partially: the controller manager detects failed
  pods and creates replacements, but the replacements cannot be scheduled.

**Controller Manager Down:**
- Existing pods are unaffected.
- New deployments work: the API server accepts the request, the scheduler
  assigns pods to nodes, and kubelets run them.
- Scaling via `kubectl scale` creates pods that the scheduler assigns.
- Self-healing is broken: no one detects pod failures or node failures.
  If a pod crashes, it stays crashed.

### Why This Table Matters

Each component has a distinct failure impact. The API server is the most
critical single point of failure because everything goes through it.
etcd is the second most critical because it holds all state. The scheduler
and controller manager are important but their failures have a narrower
impact.

## Part B: HA Architecture Diagram

```
                    Load Balancer (Virtual IP)
                    ┌──────────────────────────┐
                    │    192.168.1.100:6443     │
                    └────────────┬─────────────┘
              ┌──────────────────┼──────────────────┐
              │                  │                  │
     ┌────────▼────────┐ ┌──────▼──────────┐ ┌────▼────────────┐
     │ CP Node 1       │ │ CP Node 2       │ │ CP Node 3       │
     │ 192.168.1.1     │ │ 192.168.1.2     │ │ 192.168.1.3     │
     │                 │ │                 │ │                 │
     │ kube-apiserver  │ │ kube-apiserver  │ │ kube-apiserver  │
     │ kube-scheduler  │ │ kube-scheduler  │ │ kube-scheduler  │
     │ kube-controller │ │ kube-controller │ │ kube-controller │
     │ -manager        │ │ -manager        │ │ -manager        │
     │                 │ │                 │ │                 │
     │ etcd (LEADER)   │ │ etcd (follower) │ │ etcd (follower) │
     │ :2379,:2380     │ │ :2379,:2380     │ │ :2379,:2380     │
     └────────┬────────┘ └────────┬────────┘ └────────┬────────┘
              │                   │                   │
              └───────────────────┼───────────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
     ┌────────▼────────┐ ┌───────▼─────────┐ ┌──────▼──────────┐
     │ Worker Node 1   │ │ Worker Node 2   │ │ Worker Node 3   │
     │                 │ │                 │ │                 │
     │ kubelet         │ │ kubelet         │ │ kubelet         │
     │ kube-proxy      │ │ kube-proxy      │ │ kube-proxy      │
     │ containerd      │ │ containerd      │ │ containerd      │
     │                 │ │                 │ │                 │
     │ [Pods]          │ │ [Pods]          │ │ [Pods]          │
     └─────────────────┘ └─────────────────┘ └─────────────────┘
```

### Key Design Decisions

1. **Load balancer fronts all API servers.** Workers and kubectl connect
   to the VIP, not a specific node. If one API server goes down, the
   load balancer routes traffic to the remaining two.

2. **etcd forms a 3-node cluster.** One leader, two followers. Writes go
   through the leader and are replicated to followers. Reads can be
   served by any node.

3. **Each control plane node runs all four components.** This ensures
   that losing one node does not lose any component type.

4. **Worker nodes are separate.** They connect to the control plane
   through the load balancer VIP. If a control plane node fails, worker
   nodes seamlessly connect to another.

## Part C: etcd Cluster Design

### 1. Why Odd Numbers (3, 5, 7)?

etcd uses the Raft consensus algorithm, which requires a **majority
(quorum)** of nodes to agree on a write. The quorum formula is:

```
quorum = floor(n/2) + 1
```

| Nodes | Quorum | Can Tolerate |
|-------|--------|--------------|
| 1 | 1 | 0 failures |
| 2 | 2 | 0 failures |
| 3 | 2 | 1 failure |
| 4 | 3 | 1 failure |
| 5 | 3 | 2 failures |
| 6 | 4 | 2 failures |
| 7 | 4 | 3 failures |

A 4-node cluster has the same fault tolerance as a 3-node cluster (both
tolerate 1 failure) but costs more. That is why odd numbers are preferred.

### 2. Fault Tolerance for 3 Nodes

A 3-node etcd cluster can tolerate **1 node failure**. With 2 nodes
alive, quorum (2) is maintained. If 2 nodes fail, the cluster is
unavailable (cannot elect a leader or process writes).

### 3. Fault Tolerance for 5 Nodes

A 5-node etcd cluster can tolerate **2 node failures**. With 3 nodes
alive, quorum (3) is maintained.

### 4. Storage Location

etcd data should be stored on a **dedicated SSD**, separate from the
OS disk. Reasons:

- etcd is I/O sensitive -- slow disk = slow cluster
- OS disk failures should not affect etcd
- etcd data grows over time -- separate disk prevents disk full issues
- Easier to back up and monitor independently

Typical etcd data path: `/var/lib/etcd` (configured with `--data-dir`).

### 5. Backup Command

```bash
# Backup etcd
ETCDCTL_API=3 etcdctl snapshot save /backup/etcd-$(date +%Y%m%d-%H%M%S).db \
  --endpoints=https://127.0.0.1:2379 \
  --cacert=/etc/kubernetes/pki/etcd/ca.crt \
  --cert=/etc/kubernetes/pki/etcd/server.crt \
  --key=/etc/kubernetes/pki/etcd/server.key

# Verify backup
ETCDCTL_API=3 etcdctl snapshot status /backup/etcd-*.db --write-table
```

Backup frequency recommendation:
- **Production:** Every hour, with daily off-site copies
- **Before upgrades:** Always back up before any cluster change
- **After significant changes:** Back up after deploying new workloads

## Part D: API Server Load Balancing

### 1. Load Balancer Type

A **Layer 4 (TCP) load balancer** is recommended for the API server:

- The API server uses HTTPS (Layer 7 is not needed for simple passthrough)
- TCP load balancing is simpler and faster
- The API server handles its own TLS termination
- Options: HAProxy, nginx (stream mode), cloud load balancers (NLB, ILB)

```bash
# HAProxy configuration example
frontend k8s-api
    bind *:6443
    mode tcp
    option tcplog
    default_backend k8s-api-backend

backend k8s-api-backend
    mode tcp
    option tcp-check
    balance roundrobin
    server cp1 192.168.1.1:6443 check
    server cp2 192.168.1.2:6443 check
    server cp3 192.168.1.3:6443 check
```

### 2. Health Check

The load balancer should check the API server's `/readyz` endpoint:

```bash
# Health check command
curl -sk https://192.168.1.1:6443/readyz
# Returns "ok" if healthy
```

For HAProxy:
```
option httpchk GET /readyz
http-check expect status 200
```

### 3. Worker Node Discovery

Worker nodes use a `kubeconfig` file that contains the API server endpoint.
This should point to the load balancer VIP:

```yaml
# /etc/kubernetes/kubelet.conf
apiVersion: v1
kind: Config
clusters:
- cluster:
    certificate-authority-data: <ca-cert>
    server: https://192.168.1.100:6443  # Load balancer VIP, not a specific node
  name: kubernetes
```

### 4. Existing Connections on Node Failure

When a control plane node fails:
- The load balancer detects the failure via health check (typically 3-5
  second detection time)
- New connections are routed to healthy nodes
- Existing TCP connections to the failed node are broken
- kubelet reconnects to the load balancer, which routes to a healthy node
- The reconnection is automatic and typically takes 5-10 seconds

During the reconnection window:
- Existing pods keep running (kubelet caches state)
- Status reporting to the API server is delayed
- New pod assignments are delayed until reconnection

## Part E: Component Leader Election

### 1. What Is Leader Election?

Leader election is a distributed consensus mechanism where multiple
instances of a component compete to become the "leader." Only the leader
is active and makes decisions. The others are in standby, ready to take
over if the leader fails.

The implementation uses Kubernetes Lease objects in etcd:
- Each candidate tries to create or renew a Lease with its identity
- The first to succeed becomes the leader
- The Lease has a TTL (time-to-live)
- If the leader fails to renew before the TTL expires, another candidate
  takes over

### 2. Why Scheduler and Controller Manager Need It (But API Server Does Not)

**Scheduler:** Two schedulers running simultaneously could assign the
same pod to two different nodes. This would cause the pod to run on one
node while the API server thinks it is on another. Leader election
ensures only one scheduler makes assignments.

**Controller Manager:** Two controller managers could both detect that a
Deployment needs 3 replicas and both create a third pod, resulting in 4
pods. Or both could detect a failed pod and both create replacements,
resulting in duplicates. Leader election prevents this.

**API Server:** The API server is **stateless**. It receives a request,
processes it, and returns a response. Two API servers can handle two
requests simultaneously without conflict because they do not make
decisions -- they just read and write to etcd. The conflict resolution
happens in etcd (optimistic concurrency control).

### 3. What Happens When the Leader Fails

```
T+0s:    Leader fails (crashes, network partition, etc.)
T+0-15s: Other candidates do not see the Lease renewed
T+15s:   Lease expires (default TTL)
T+15s:   A standby candidate creates/renews the Lease
T+15s:   New leader starts processing work
T+16s:   Normal operation resumes
```

The failover time is determined by the Lease TTL (default 15 seconds).
This means there is a 15-second window where no scheduler or controller
manager is active.

### 4. Typical Failover Time

- **Lease duration:** 15 seconds (configurable)
- **Renew interval:** 10 seconds (2/3 of lease duration)
- **Retry period:** 2 seconds
- **Typical failover:** 15-30 seconds

During failover:
- New pods are not scheduled (scheduler is down)
- Self-healing is paused (controller manager is down)
- Existing pods keep running (kubelet is independent)
- The API server is unaffected (it is stateless)

## Part F: Implementation Plan

### Prerequisites

```
Hardware:
  - 3 control plane nodes: 4 CPU, 8 GB RAM, 100 GB SSD each
  - 3 worker nodes: 4 CPU, 16 GB RAM, 200 GB SSD each
  - 1 load balancer (can be a VM or cloud LB)

Networking:
  - All nodes on the same network (or routable networks)
  - Ports open:
    - 6443: API server
    - 2379-2380: etcd
    - 10250: kubelet
    - 10257: kube-controller-manager
    - 10259: kube-scheduler
  - Load balancer VIP assigned (e.g., 192.168.1.100)
```

### Step-by-Step Plan

```
Step 1: Set up the Load Balancer
════════════════════════════════
  - Deploy HAProxy or configure cloud LB
  - Point to all 3 control plane IPs on port 6443
  - Configure health check: GET /readyz on port 6443
  - Test: curl -k https://192.168.1.100:6443/readyz

Step 2: Initialize the First Control Plane Node
════════════════════════════════════════════════
  # On CP Node 1
  sudo kubeadm init \
    --control-plane-endpoint "192.168.1.100:6443" \
    --upload-certs \
    --pod-network-cidr=10.244.0.0/16

  # Save the join command output (includes token and cert key)
  # Set up kubectl
  mkdir -p $HOME/.kube
  sudo cp /etc/kubernetes/admin.conf $HOME/.kube/config

Step 3: Install CNI Plugin
═══════════════════════════
  kubectl apply -f https://docs.projectcalico.org/manifests/calico.yaml
  # Wait for calico pods to be Running

Step 4: Join the Second Control Plane Node
══════════════════════════════════════════
  # On CP Node 2
  sudo kubeadm join 192.168.1.100:6443 \
    --token <token> \
    --discovery-token-ca-cert-hash <hash> \
    --control-plane \
    --certificate-key <cert-key>

Step 5: Join the Third Control Plane Node
═════════════════════════════════════════
  # On CP Node 3
  sudo kubeadm join 192.168.1.100:6443 \
    --token <token> \
    --discovery-token-ca-cert-hash <hash> \
    --control-plane \
    --certificate-key <cert-key>

Step 6: Join Worker Nodes
═════════════════════════
  # On each Worker Node
  sudo kubeadm join 192.168.1.100:6443 \
    --token <token> \
    --discovery-token-ca-cert-hash <hash>

Step 7: Verify the Cluster
══════════════════════════
  # Check all nodes are Ready
  kubectl get nodes
  # Expected: 6 nodes (3 control-plane, 3 worker), all Ready

  # Check etcd cluster health
  kubectl exec -it etcd-cp1 -n kube-system -- \
    etcdctl --endpoints=https://127.0.0.1:2379 \
    --cacert=/etc/kubernetes/pki/etcd/ca.crt \
    --cert=/etc/kubernetes/pki/etcd/server.crt \
    --key=/etc/kubernetes/pki/etcd/server.key \
    endpoint health
  # Expected: all 3 endpoints healthy

  # Check leader election
  kubectl get lease kube-scheduler -n kube-system -o yaml
  kubectl get lease kube-controller-manager -n kube-system -o yaml

Step 8: Update Worker kubeconfig
════════════════════════════════
  # On each worker, update the API server endpoint to the LB VIP
  sudo sed -i 's|server: https://.*:6443|server: https://192.168.1.100:6443|' \
    /etc/kubernetes/kubelet.conf
  sudo systemctl restart kubelet
```

### Failover Test Procedure

```
Test 1: Kill a Control Plane Node
═════════════════════════════════
  # On CP Node 1, shut down all control plane components
  sudo systemctl stop kubelet

  # Verify:
  # - kubectl still works (through LB to CP Node 2 or 3)
  # - New pods can be created and scheduled
  # - Existing pods keep running
  # - etcd cluster shows 2/3 healthy

  # Restart
  sudo systemctl start kubelet

Test 2: Kill etcd on One Node
═════════════════════════════
  # On CP Node 2, stop etcd
  docker stop etcd  # or kill the etcd process

  # Verify:
  # - etcd cluster shows 2/3 healthy
  # - Cluster operations continue
  # - No data loss

  # Restart
  docker start etcd

Test 3: Simulate Network Partition
═══════════════════════════════════
  # On CP Node 1, block network to other CP nodes
  sudo iptables -A OUTPUT -d 192.168.1.2 -j DROP
  sudo iptables -A OUTPUT -d 192.168.1.3 -j DROP

  # Verify:
  # - etcd leader re-election happens on CP Node 2 or 3
  # - Scheduler and controller manager leader election happens
  # - Cluster continues operating on the majority partition

  # Restore
  sudo iptables -D OUTPUT -d 192.168.1.2 -j DROP
  sudo iptables -D OUTPUT -d 192.168.1.3 -j DROP

Test 4: Load Balancer Failover
══════════════════════════════
  # Block traffic to one API server at the LB level
  # Verify LB health check detects the failure
  # Verify kubectl commands still work through remaining API servers
```

### Common Mistakes

- **Using `--control-plane-endpoint` with a specific node IP.** This
  defeats the purpose of HA. Always use the load balancer VIP.
- **Not backing up etcd before the migration.** If something goes wrong,
  you need a restore point.
- **Forgetting to update worker kubeconfig files.** Workers still point
  to the old single API server. After HA setup, update them to point to
  the load balancer VIP.
- **Not testing failover.** An HA setup that has never been tested is
  not HA -- it is a hope. Run the failover tests before declaring victory.
- **Using an even number of etcd nodes.** 4 nodes has the same fault
  tolerance as 3 but costs more. Always use odd numbers.
- **Placing etcd on the same disk as the OS.** etcd is I/O sensitive.
  A busy OS disk will slow etcd and cause leader elections.
- **Not monitoring etcd health.** etcd disk full or slow etcd will
  cascade into cluster-wide failures. Monitor etcd disk usage, latency,
  and leader election frequency.
- **Ignoring the 15-second failover window.** During leader election,
  the scheduler and controller manager are inactive. Design your
  applications to tolerate brief periods without scheduling or
  self-healing.

## Relevant README Sections

- [Control Plane Components](../README.md#control-plane-components) -- API server, etcd, scheduler, controller manager
- [etcd](../README.md#etcd) -- Distributed key-value store, Raft consensus
- [Cluster Setup Options](../README.md#cluster-setup-options) -- kubeadm setup
- [kubelet](../README.md#kubelet) -- Caching behavior during control plane outages
