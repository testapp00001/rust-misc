# Exercise 04: Multi-Host Networking and Failover

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

You will design and troubleshoot a multi-host Docker Swarm deployment with overlay networking, encrypted communication, and automatic failover. This exercise practices network architecture, firewall rules, node management, and failure recovery.

## Scenario

You have a 5-node Docker Swarm cluster:

- **manager1** (192.168.1.10) -- Manager
- **manager2** (192.168.1.11) -- Manager
- **manager3** (192.168.1.12) -- Manager
- **worker1** (192.168.1.20) -- Worker
- **worker2** (192.168.1.21) -- Worker

You need to deploy a service that spans all worker nodes with encrypted overlay networking, and you must handle node failures gracefully.

## Tasks

### Part A: Network Architecture

Design the overlay network configuration. Write the commands to:

1. Create an encrypted overlay network called `secure-net` with subnet `10.0.10.0/24`.
2. Create an internal overlay network called `internal-db` with subnet `10.0.20.0/24`.
3. List the required firewall rules for inter-host communication (Swarm management, VXLAN, node discovery).

Explain why encryption matters on the overlay network and what protocol it uses.

<details>
<summary>Hint</summary>
Docker Swarm overlay encryption uses IPsec (AES-GCM). The key ports are: 2377 (Swarm management TCP), 4789 (VXLAN UDP), 7946 (node communication TCP+UDP). Use `--opt encrypted` on the network create command.
</details>

### Part B: Failover Simulation

Write the sequence of commands to:

1. Deploy a service called `web` with 5 replicas on the `secure-net` network, accessible on port 8080.
2. Drain `worker1` from the cluster (simulate maintenance).
3. Verify that tasks have migrated to `worker2`.
4. Set `worker1` back to active.
5. Force-remove a manager node and verify the cluster survives (you have 3 managers, so Raft quorum is maintained).

<details>
<summary>Hint</summary>
Use `docker node update --availability drain <node>` to drain a node. Use `docker service ps web` to see task distribution. A 3-manager cluster tolerates 1 manager failure (quorum = 2).
</details>

### Part C: Troubleshooting

You deploy a service but containers on `worker2` cannot reach containers on `worker1` via the overlay network. List at least 4 possible causes and the diagnostic command for each.

<details>
<summary>Hint</summary>
Think about: firewall rules (iptables), VXLAN port (4789 UDP), overlay network encryption keys, node health status, and whether the service is actually attached to the overlay network.
</details>

## Success Criteria

- [ ] Overlay networks are created with correct drivers, subnets, and encryption.
- [ ] Firewall rules cover all required Swarm ports (2377, 4789, 7946 TCP and UDP).
- [ ] Failover commands demonstrate draining, task migration, and manager removal.
- [ ] Troubleshooting section identifies at least 4 distinct failure causes with diagnostic commands.
- [ ] You can explain why Raft requires an odd number of managers and what quorum means.

## What You Should Understand After This Exercise

Multi-host networking in Docker Swarm relies on VXLAN-based overlay networks that create a virtual Layer 2 network across hosts. Encryption protects traffic on untrusted networks. Failover is automatic when services have restart policies and enough nodes, but you must plan for manager quorum (always use 3 or 5 managers). Troubleshooting multi-host networking requires checking firewalls, node health, network attachment, and encryption state.
