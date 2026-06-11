# Solution 04: Multi-Host Networking and Failover

## Part A: Network Architecture

### Overlay Network Creation

```bash
# 1. Encrypted overlay network for application traffic
docker network create \
  --driver overlay \
  --opt encrypted \
  --subnet 10.0.10.0/24 \
  secure-net

# 2. Internal overlay network for database traffic (no external access)
docker network create \
  --driver overlay \
  --internal \
  --subnet 10.0.20.0/24 \
  internal-db
```

### Firewall Rules

```bash
# Swarm management (TCP 2377) -- cluster join and management
sudo iptables -A INPUT -p tcp --dport 2377 -j ACCEPT

# VXLAN overlay traffic (UDP 4789) -- container-to-container across hosts
sudo iptables -A INPUT -p udp --dport 4789 -j ACCEPT

# Node communication (TCP 7946) -- gossip protocol for membership
sudo iptables -A INPUT -p tcp --dport 7946 -j ACCEPT

# Node communication (UDP 7946) -- gossip protocol for failure detection
sudo iptables -A INPUT -p udp --dport 7946 -j ACCEPT
```

### Why Encryption Matters

Overlay networks without encryption send traffic as plaintext VXLAN packets. On an untrusted network (or even between data centers), an attacker can sniff inter-container traffic. The `--opt encrypted` flag enables IPsec encryption (AES-GCM) for all overlay traffic. This adds a small performance overhead (~5-10%) but prevents eavesdropping.

VXLAN (Virtual Extensible LAN) encapsulates Layer 2 Ethernet frames in UDP packets on port 4789. This allows containers on different physical hosts to communicate as if they were on the same Layer 2 network.

### Common Mistakes to Avoid

- Forgetting UDP 4789. This is the most commonly missed firewall rule. Without it, overlay traffic cannot flow between hosts.
- Using `--opt encrypted` on a performance-critical path without benchmarking. IPsec adds latency.
- Not opening both TCP and UDP for port 7946. Swarm uses TCP for gossip communication and UDP for failure detection.

---

## Part B: Failover Simulation

### Step 1: Deploy the Service

```bash
docker service create \
  --name web \
  --replicas 5 \
  --network secure-net \
  -p 8080:80 \
  nginx:alpine
```

### Step 2: Verify Initial Distribution

```bash
docker service ps web
# Should show tasks distributed across worker1 and worker2
```

### Step 3: Drain worker1

```bash
# Get the node ID
docker node ls

# Drain worker1 (moves all tasks off the node)
docker node update --availability drain worker1
```

### Step 4: Verify Task Migration

```bash
docker service ps web
# All 5 tasks should now be on worker2
# Old tasks on worker1 show "Shutdown" state
docker service ps web --filter "desired-state=running"
```

### Step 5: Restore worker1

```bash
docker node update --availability active worker1
# New tasks will be scheduled back on worker1 as needed
```

### Step 6: Force-Remove a Manager

```bash
# On manager3, force leave
docker node update --availability drain manager3
docker swarm leave --force

# On manager1, remove the node
docker node rm manager3

# Verify cluster survives (2 managers, quorum of 3 still has 2)
docker node ls
docker info | grep -A5 "Raft"
```

### Common Mistakes to Avoid

- Confusing `drain` with `remove`. Drain keeps the node in the cluster but moves tasks off. Remove deletes the node.
- Not checking `docker service ps web --no-trunc` to see the full task history including shutdown tasks.
- Force-removing 2 of 3 managers. Raft quorum requires `(n/2) + 1` = 2 for 3 nodes. Losing 2 managers breaks quorum.

---

## Part C: Troubleshooting

### Problem: Containers on worker2 cannot reach containers on worker1 via overlay network.

**Cause 1: Firewall blocking VXLAN traffic (UDP 4789)**
```bash
# Check if port is open
sudo iptables -L -n | grep 4789
# Or test connectivity
nc -u -z worker1 4789
```

**Cause 2: Overlay network encryption key mismatch**
```bash
# Check network details on each node
docker network inspect secure-net
# Look for "Encryption" field
# Keys are rotated automatically, but a node that was offline during rotation may have stale keys
```

**Cause 3: Service not attached to the overlay network**
```bash
# Verify the service is on the correct network
docker service inspect web --format '{{json .Spec.TaskTemplate.Networks}}'
# Should show secure-net
```

**Cause 4: Node health -- worker1 is down or unreachable**
```bash
# Check node status
docker node ls
# Look for "Down" status
docker node inspect worker1 --format '{{.Status.State}}'
```

**Cause 5: DNS resolution failure within the overlay**
```bash
# From a container on worker2, try resolving the service name
docker exec <container_on_worker2> nslookup web
# If this fails, the overlay DNS is not working
```

**Cause 6: MTU mismatch**
```bash
# VXLAN adds 50 bytes of overhead. If the host MTU is 1500,
# the overlay MTU should be 1450.
# Check MTU on the overlay interface inside a container
docker exec <container> ip link show
```

### Common Mistakes to Avoid

- Only checking one cause. Multi-host networking failures are often multi-layered (firewall + MTU, or encryption + node health).
- Not checking node health first. It is the simplest diagnostic and rules out the most obvious cause.
- Assuming the overlay network works because `docker network inspect` shows it. The network existing does not mean traffic flows.

---

## Key Takeaway

Docker Swarm overlay networking creates a virtual Layer 2 network across hosts using VXLAN encapsulation. Encryption adds IPsec protection for untrusted networks. Failover is automatic when services have restart policies and sufficient replicas, but manager quorum (3 or 5 nodes) is critical for cluster survival. Troubleshooting multi-host networking requires a systematic approach: check node health, firewall rules, network attachment, DNS resolution, and MTU settings.
