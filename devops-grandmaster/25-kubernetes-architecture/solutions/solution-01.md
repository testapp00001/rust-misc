# Solution 01: Map the Kubernetes Architecture

## Part A: Architecture Diagram

```
┌─────────────────────────────────────────────────────────┐
│                  Kubernetes Cluster                      │
│                                                         │
│  ┌───────────────────────────────────────────────────┐ │
│  │              Control Plane                         │ │
│  │  ┌──────────┬──────────┬──────────┬──────────┐    │ │
│  │  │ API      │ Scheduler│ Controller│ etcd    │    │ │
│  │  │ Server   │          │ Manager   │         │    │ │
│  │  └──────────┴──────────┴──────────┴──────────┘    │ │
│  └───────────────────────────────────────────────────┘ │
│                           │                             │
│  ┌────────────────────────┼─────────────────────────┐  │
│  │                        │                          │  │
│  │  ┌─────────────┐  ┌───▼───────────┐  ┌────────┐ │  │
│  │  │ Worker      │  │ Worker        │  │ Worker │ │  │
│  │  │ Node 1      │  │ Node 2        │  │ Node 3 │ │  │
│  │  │ ┌─────┐    │  │ ┌─────┐      │  │ ┌────┐ │ │  │
│  │  │ │Pod A│    │  │ │Pod D│      │  │ │Pod │ │ │  │
│  │  │ │Pod B│    │  │ │Pod E│      │  │ │ F  │ │ │  │
│  │  │ │Pod C│    │  │ └─────┘      │  │ └────┘ │ │  │
│  │  │ └─────┘    │  │  kubelet     │  │ kubelet│ │  │
│  │  │  kubelet   │  │  kube-proxy  │  │ kube-  │ │  │
│  │  │  kube-proxy│  │  containerd  │  │ proxy  │ │  │
│  │  │  containerd│  │              │  │contain-│ │  │
│  │  └─────────────┘  └──────────────┘  │erd     │ │  │
│  │                                      └────────┘ │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Why This Layout

The control plane is drawn as a single box because its components are
typically co-located on the same machine(s). Worker nodes are separate
machines, each running the same set of agent components. The vertical
arrow from control plane to worker nodes represents the watch/notify
relationship -- the API server is the hub, and worker nodes poll it
for changes.

## Part B: Component Table

| Component | Runs On | One-Sentence Role |
|-----------|---------|-------------------|
| API Server (kube-apiserver) | Control Plane | The single entry point for all cluster operations; validates and processes REST requests and stores results in etcd. |
| etcd | Control Plane | Distributed key-value store that holds the entire cluster state -- every object, every configuration, every secret. |
| Scheduler (kube-scheduler) | Control Plane | Watches for pods without a node assignment and selects the best node based on resource availability, constraints, and policies. |
| Controller Manager (kube-controller-manager) | Control Plane | Runs reconciliation loops that watch the actual cluster state and make changes to match the desired state declared in etcd. |
| kubelet | Worker Node | Agent that receives pod specifications from the API server, instructs the container runtime to start containers, and reports status back. |
| kube-proxy | Worker Node | Maintains network rules (iptables/IPVS) on the node so that Kubernetes Services can route traffic to the correct pods. |
| Container Runtime (containerd/CRI-O) | Worker Node | Pulls container images and runs containers according to the Container Runtime Interface (CRI) specification. |

### Why These Roles Are Distinct

Each component has a non-overlapping responsibility:
- **API Server** is the communication hub -- nothing talks directly to etcd except the API server.
- **etcd** is the persistence layer -- it stores state but makes no decisions.
- **Scheduler** decides *where* to run pods but does not actually run them.
- **Controller Manager** decides *what* pods to create/delete but does not schedule or run them.
- **kubelet** runs pods but does not decide which pods to create or where to put them.
- **kube-proxy** handles networking but does not run pods.
- **Container Runtime** is the lowest layer -- it only knows how to pull images and run containers.

## Part C: Request Flow

```
Developer runs: kubectl apply -f pod.yaml

Step 1: kubectl (developer's machine)
  └── Reads pod.yaml, sends HTTP POST to API server

Step 2: API Server (control plane)
  └── Authenticates the request
  └── Validates the YAML schema
  └── Stores the Pod object in etcd (phase: Pending, nodeName: "")

Step 3: etcd (control plane)
  └── Persists the Pod object
  └── Notifies API server of successful write

Step 4: Scheduler (control plane)
  └── Watches API server for pods with empty nodeName
  └── Finds the new pod
  └── Evaluates available nodes (resources, taints, affinity)
  └── Selects the best node
  └── Sends a binding request to API server: "Pod X → Node Y"

Step 5: API Server (control plane)
  └── Updates the Pod object in etcd with nodeName

Step 6: kubelet on Node Y (worker node)
  └── Watches API server for pods assigned to its node
  └── Sees the new pod assignment
  └── Pulls the container image (if not cached)
  └── Instructs container runtime to create and start the container

Step 7: Container Runtime on Node Y (worker node)
  └── Pulls image from registry
  └── Creates container
  └── Starts the process inside the container

Step 8: kubelet on Node Y (worker node)
  └── Monitors container health
  └── Reports pod status back to API server: "Running"

Step 9: API Server (control plane)
  └── Updates Pod status in etcd

Step 10: Developer runs kubectl get pods
  └── API server reads from etcd
  └── Returns: STATUS = Running
```

### Why This Sequence Matters

Every communication goes through the API server. The scheduler never
talks to kubelet. The kubelet never talks to etcd. This hub-and-spoke
model simplifies security (only the API server needs etcd credentials)
and makes the system auditable (all actions go through one point).

## Part D: "Without X, ..." Statements

- **Without the API server:** Nothing can communicate. kubectl commands
  fail. No new pods can be created, no status can be reported. The cluster
  is effectively frozen (existing pods keep running on their nodes, but
  the cluster cannot be managed).

- **Without etcd:** The API server has no state. It cannot tell you what
  pods exist, what deployments are configured, or what nodes are registered.
  The cluster loses all knowledge of itself.

- **Without the scheduler:** New pods stay in `Pending` forever. The
  scheduler is the only component that assigns pods to nodes. Existing
  pods continue running because the kubelet does not need the scheduler.

- **Without the controller manager:** Deployments stop self-healing. If a
  pod crashes, no one creates a replacement. If you scale a deployment,
  no one creates the new pods. The "desired state" is written to etcd but
  no one acts on it.

### Why Each Failure Is Unique

These four failure modes are distinct:
- API server failure = communication failure (no input/output)
- etcd failure = memory loss (no state)
- Scheduler failure = assignment failure (pods cannot be placed)
- Controller manager failure = automation failure (no reconciliation)

This is why each component is essential and cannot be replaced by another.

## Common Mistakes

- **Confusing the scheduler with the controller manager.** The scheduler
  decides *where* a pod runs (once, at creation time). The controller
  manager ensures the *right number* of pods exist (continuously). They
  do different jobs at different times.
- **Thinking the kubelet needs the control plane to keep pods running.**
  Once a pod is running, the kubelet manages it independently. If the
  control plane goes down, existing pods keep running. Only new changes
  require the control plane.
- **Drawing the API server as just another component.** The API server is
  the central hub. Every other component communicates through it. It is
  not a peer -- it is the gateway.
- **Omitting the container runtime.** Students often draw kubelet running
  containers directly. The kubelet talks to the container runtime (containerd
  or CRI-O) through the CRI, which then runs the containers.

## Relevant README Sections

- [The Big Picture](../README.md#the-big-picture) -- Architecture diagram
- [Control Plane Components](../README.md#control-plane-components) -- API server, etcd, scheduler, controller manager
- [Worker Node Components](../README.md#worker-node-components) -- kubelet, kube-proxy, container runtime
- [How a Pod Gets Created](../README.md#how-a-pod-gets-created) -- Step-by-step pod lifecycle
