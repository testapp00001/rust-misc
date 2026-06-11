# Exercise 05: Design a Highly Available Control Plane

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design a highly available (HA) Kubernetes control plane for a production
cluster. This exercise integrates your understanding of control plane
components with concepts from [Module 22: Load Balancing](../22-load-balancing/)
and [Module 19: Networking Fundamentals](../19-networking-fundamentals/).

## Scenario

**FinTrack Analytics** runs a real-time financial data processing platform.
Their Kubernetes cluster currently has a single control plane node. Last
week, the etcd disk failed and the entire cluster became unresponsive for
47 minutes. The CTO has issued an ultimatum: "Never again."

```
Current Architecture (Single Control Plane)
============================================

┌──────────────────────────────────┐
│     Control Plane Node (1)       │
│  ┌────────┬────────┬────────┐   │
│  │ API    │ Sched- │ Control│   │
│  │ Server │ uler   │ Manager│   │
│  └────────┴────────┴────────┘   │
│  ┌──────────────────────────┐   │
│  │         etcd             │   │
│  └──────────────────────────┘   │
└──────────────┬───────────────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
┌───▼───┐ ┌───▼───┐ ┌───▼───┐
│Worker │ │Worker │ │Worker │
│Node 1 │ │Node 2 │ │Node 3 │
└───────┘ └───────┘ └───────┘
```

Requirements:
- Zero downtime if one control plane node fails
- etcd must survive the loss of one node
- API server must remain accessible during node failure
- Target: 3 control plane nodes (minimum for quorum)

## Tasks

### Part A: Understand the Failure Modes

For each control plane component, describe what happens when it goes down:

1. **API server down:** What can still work? What breaks immediately?
2. **etcd down:** What can still work? What breaks immediately?
3. **Scheduler down:** What can still work? What breaks immediately?
4. **Controller manager down:** What can still work? What breaks immediately?

Fill in this table:

| Component Down | Existing Pods | New Deployments | Scaling | Self-Healing | Cluster State |
|----------------|---------------|-----------------|---------|--------------|---------------|
| API Server | ? | ? | ? | ? | ? |
| etcd | ? | ? | ? | ? | ? |
| Scheduler | ? | ? | ? | ? | ? |
| Controller Manager | ? | ? | ? | ? | ? |

<details>
<summary>Hint</summary>

Think about which component is needed for each operation:
- **Existing pods** run on the kubelet, which does not need the control
  plane for normal operation (it caches the last known state)
- **New deployments** require the API server to accept the request
- **Scaling** requires the controller manager to create/delete pods
- **Self-healing** requires the controller manager to detect and replace failed pods
- **Cluster state** is stored in etcd

</details>

### Part B: Design the HA Architecture

Draw an ASCII diagram of a 3-node control plane showing:

1. All three control plane nodes
2. Which components run on each node
3. How etcd forms a cluster (Raft consensus)
4. How the API server is load balanced
5. How worker nodes connect to the control plane

<details>
<summary>Hint</summary>

```
                 Load Balancer (VIP)
                 ┌──────────────────┐
                 │  192.168.1.100   │
                 └────────┬─────────┘
            ┌─────────────┼─────────────┐
            │             │             │
     ┌──────▼──────┐ ┌───▼────────┐ ┌──▼───────────┐
     │ CP Node 1   │ │ CP Node 2  │ │ CP Node 3    │
     │             │ │            │ │              │
     │ API Server  │ │ API Server │ │ API Server   │
     │ Scheduler   │ │ Scheduler  │ │ Scheduler    │
     │ Ctrl Mgr    │ │ Ctrl Mgr   │ │ Ctrl Mgr     │
     │ etcd (leader)│ │ etcd(follower)│ │ etcd(follower)│
     └──────┬──────┘ └──────┬─────┘ └──────┬───────┘
            │               │              │
            └───────────────┼──────────────┘
                            │
              ┌─────────────┼─────────────┐
              │             │             │
         ┌────▼───┐   ┌────▼───┐   ┌────▼───┐
         │Worker 1│   │Worker 2│   │Worker 3│
         └────────┘   └────────┘   └────────┘
```

Key points:
- etcd uses Raft consensus: one leader, two followers
- Writes go to the leader, reads can go to any node
- API servers are stateless -- run one per control plane node
- Scheduler and controller manager use leader election -- only one
  is active at a time, others are standby

</details>

### Part C: etcd Cluster Design

Answer these questions about the etcd cluster:

1. Why does etcd need an odd number of nodes (3, 5, 7)?
2. How many node failures can a 3-node etcd cluster tolerate?
3. How many node failures can a 5-node etcd cluster tolerate?
4. Where should etcd data be stored -- on the same disk as the OS, or a separate disk?
5. How often should etcd be backed up, and what command backs it up?

<details>
<summary>Hint</summary>

etcd uses the Raft consensus algorithm, which requires a majority
(quorum) of nodes to agree on a write. With 3 nodes, quorum is 2.
With 5 nodes, quorum is 3.

An even number of nodes (4) has the same fault tolerance as the odd
number below it (3), but costs more. That is why odd numbers are used.

etcd is I/O sensitive. A separate SSD for etcd data dramatically
improves performance and reliability.

</details>

### Part D: API Server Load Balancing

The API server must remain accessible even if one control plane node
fails. Design the load balancing approach:

1. What type of load balancer should front the API servers?
2. What health check should the load balancer use?
3. How do worker nodes discover the API server endpoint?
4. What happens to existing kubelet connections when a control plane node fails?

<details>
<summary>Hint</summary>

The API servers are stateless -- any API server can handle any request.
This makes load balancing straightforward.

For the health check, the API server exposes `/readyz` and `/livez`
endpoints. The load balancer should check `/readyz` on port 6443.

Worker nodes use the `kubeconfig` file which contains the API server
endpoint. This should point to the load balancer VIP, not a specific
control plane node.

</details>

### Part E: Component Leader Election

The scheduler and controller manager use leader election. Explain:

1. What is leader election?
2. Why do the scheduler and controller manager need it, but the API server does not?
3. What happens when the active leader fails?
4. How long does failover typically take?

<details>
<summary>Hint</summary>

Leader election ensures only one instance of a component is active at
a time. This prevents conflicting decisions (e.g., two schedulers
assigning the same pod to different nodes).

The API server does not need leader election because it is stateless --
multiple instances can serve requests simultaneously. The scheduler and
controller manager are stateful in the sense that they make decisions
that must not conflict.

Failover time depends on the lease duration, typically 15-30 seconds.

</details>

### Part F: Write the Implementation Plan

Write a step-by-step plan to convert the single control plane cluster
to a 3-node HA cluster. Include:

1. Prerequisites (hardware, networking, etc.)
2. Steps to add each control plane node
3. How to set up the load balancer
4. How to update worker nodes to point to the load balancer
5. How to verify the HA setup works
6. How to test failover (simulate a node failure)

<details>
<summary>Hint</summary>

For kubeadm-based clusters:
```bash
# On the first control plane node
kubeadm init --control-plane-endpoint "lb-vip:6443" --upload-certs

# On subsequent control plane nodes
kubeadm join lb-vip:6443 \
  --token <token> \
  --discovery-token-ca-cert-hash <hash> \
  --control-plane \
  --certificate-key <cert-key>
```

The `--control-plane-endpoint` flag is the key -- it tells kubeadm
to set up the cluster with HA in mind.

</details>

## Success Criteria

- [ ] Failure mode table is filled in with accurate impact analysis
- [ ] HA diagram shows 3 control plane nodes with all components
- [ ] etcd quorum rules are correctly explained (odd numbers, fault tolerance)
- [ ] Load balancing design includes health checks and discovery
- [ ] Leader election is explained with correct reasoning
- [ ] Implementation plan has clear, ordered steps
- [ ] The plan includes a failover test procedure
- [ ] You can explain why each design decision was made

## What You Should Understand After This Exercise

High availability in Kubernetes requires redundancy at every layer of
the control plane. etcd needs a quorum-based cluster (odd number of
nodes). The API server is stateless and can be load balanced. The
scheduler and controller manager use leader election to prevent
conflicting decisions. Worker nodes connect to the control plane through
a load balancer VIP, not a specific node. Testing failover is as
important as implementing it -- a setup that has never been tested is
not HA, it is a hope.
