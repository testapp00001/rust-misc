# Module 08: Container Networking

## The Problem: How Do Containers Talk to Each Other?

In Module 07, you learned to wire up multi-container applications with Docker Compose. A typical stack looks like this:

```yaml
services:
  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
  backend:
    build: ./backend
    ports:
      - "8080:8080"
  database:
    image: postgres:16
    ports:
      - "5432:5432"
```

It works. But you never stopped to ask **why** it works. How does `frontend` reach `backend`? How does `backend` reach `database`? What is that `ports` mapping actually doing? And why does `frontend` talk to `backend` using the hostname `backend` instead of an IP address?

If you cannot answer these questions, you cannot debug networking problems. And networking problems are the number one issue people hit when moving from "it works on my laptop" to production.

### The Questions You Need to Answer

1. How does Docker create networks for containers?
2. What is the difference between port mapping (`-p`) and containers being on the same network?
3. How does one container find another container by name?
4. How do you isolate containers so they cannot reach each other?
5. What happens when you need containers on different physical hosts to communicate?

---

## The Naive Way: Just Use Port Mapping

The first instinct when containers need to communicate is to map every port to the host:

```bash
# Start a database
docker run -d --name database -p 5432:5432 -e POSTGRES_PASSWORD=secret postgres:16

# Start a backend that connects to the database
docker run -d --name backend -p 8080:8080 \
  -e DATABASE_URL=postgresql://postgres:secret@localhost:5432/postgres \
  my-backend:1.0
```

This works because both containers map ports to the host. The backend connects to `localhost:5432`, which reaches the database through the host's port mapping.

### Why This Fails at Scale

**Problem 1: Port conflicts.**
You can only map one container to host port 5432. What if you need two databases?

```bash
# First database works
docker run -d --name db1 -p 5432:5432 postgres:16

# Second database fails
docker run -d --name db2 -p 5432:5432 postgres:16
# Error: Bind for 0.0.0.0:5432 failed: port is already allocated
```

**Problem 2: Security exposure.**
Every mapped port is accessible from the outside world (or at minimum, from the host network). Your database should never be exposed to the internet, but `-p 5432:5432` exposes it on every network interface.

**Problem 3: Host dependency.**
Containers communicate through the host, not directly. If the host changes its IP, or if you move to a different host, everything breaks. You are hardcoding `localhost` into your configuration.

**Problem 4: Unnecessary overhead.**
Every packet from backend to database goes: container -> host network stack -> container. That is two extra hops for containers that could talk directly.

---

## The Right Way: Docker Networks

Docker has a networking subsystem that lets containers communicate directly without involving the host's ports. This is the foundation of container networking.

### Network Drivers

Docker supports several network drivers. Each serves a different purpose.

```
┌─────────────────────────────────────────────────────────────────┐
│                     Docker Network Drivers                       │
├──────────────┬──────────────────────────────────────────────────┤
│   bridge     │ Default. Containers on the same host can talk.   │
│              │ Isolated from the host's network.                 │
├──────────────┼──────────────────────────────────────────────────┤
│   host       │ Container shares the host's network stack.       │
│              │ No isolation. Best performance.                   │
├──────────────┼──────────────────────────────────────────────────┤
│   none       │ No networking at all. Container is isolated.     │
│              │ Useful for batch jobs that need no network.       │
├──────────────┼──────────────────────────────────────────────────┤
│   overlay    │ Spans multiple hosts. Used with Docker Swarm.    │
│              │ Required for cross-host container communication.  │
├──────────────┼──────────────────────────────────────────────────┤
│   macvlan    │ Assigns a real MAC address to the container.     │
│              │ Container appears as a physical device on the LAN.│
├──────────────┼──────────────────────────────────────────────────┤
│   ipvlan     │ Similar to macvlan but uses the parent's MAC.    │
│              │ Better for environments that restrict MAC spoofing│
└──────────────┴──────────────────────────────────────────────────┘
```

### The Default Bridge Network

When you install Docker, it creates a network called `bridge` automatically. Every container that starts without specifying a network gets attached to this default bridge.

```bash
# See all networks
docker network ls

# Output:
# NETWORK ID     NAME      DRIVER    SCOPE
# a1b2c3d4e5f6   bridge    bridge    local
# f6e5d4c3b2a1   host      host      local
# 1a2b3c4d5e6f   none      null      local
```

Start two containers without specifying a network:

```bash
# These go on the default bridge network
docker run -d --name app1 nginx:alpine
docker run -d --name app2 nginx:alpine

# They CAN communicate, but only by IP address
docker exec app1 ping -c 2 172.17.0.3   # Works
docker exec app1 ping -c 2 app2          # Fails! DNS does not work on default bridge
```

**The default bridge does NOT support DNS resolution by container name.** This is the single biggest reason to never use it for multi-container applications. You have to use IP addresses, which change every time a container restarts.

### Custom Bridge Networks

Creating your own bridge network solves the DNS problem:

```bash
# Create a custom bridge network
docker network create my-network

# Start containers on the custom network
docker run -d --name app1 --network my-network nginx:alpine
docker run -d --name app2 --network my-network nginx:alpine

# Now DNS works by container name
docker exec app1 ping -c 2 app2          # Works!
docker exec app1 ping -c 2 app1          # Works!

# You can also use the full container name or aliases
docker exec app1 curl -s http://app2:80  # Works!
```

**Why does DNS work on custom networks but not the default bridge?** Docker runs an embedded DNS server (at 127.0.0.11) on custom networks. This server resolves container names to their IP addresses automatically. The default bridge does not have this feature because it predates the DNS server and exists only for backward compatibility.

### Default Bridge vs Custom Bridge

```
┌────────────────────────────────────────────────────────────────────┐
│                     Default Bridge                                  │
│                                                                    │
│   ┌──────────┐      ┌──────────┐      ┌──────────┐               │
│   │  app1    │      │  app2    │      │  app3    │               │
│   │ 172.17.  │      │ 172.17.  │      │ 172.17.  │               │
│   │  0.2     │      │  0.3     │      │  0.4     │               │
│   └────┬─────┘      └────┬─────┘      └────┬─────┘               │
│        │                 │                  │                      │
│        └────────┬────────┴──────────────────┘                      │
│                 │                                                   │
│          ┌──────┴──────┐                                           │
│          │ docker0     │  No DNS. Must use IP addresses.          │
│          │ 172.17.0.1  │  All containers on same flat network.    │
│          └─────────────┘                                           │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│                     Custom Bridge: "web-net"                        │
│                                                                    │
│   ┌──────────┐      ┌──────────┐                                  │
│   │ frontend │      │ backend  │    DNS resolves names.           │
│   │          │──────│          │    Containers isolated by        │
│   └──────────┘      └──────────┘    network.                     │
│                                                                    │
│          ┌─────────────┐                                           │
│          │  web-net    │  Embedded DNS at 127.0.0.11              │
│          │ 172.18.0.1  │  Name resolution works.                  │
│          └─────────────┘                                           │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│                     Custom Bridge: "db-net"                         │
│                                                                    │
│   ┌──────────┐      ┌──────────┐                                  │
│   │ postgres │      │ redis    │    Completely isolated from       │
│   │          │──────│          │    web-net.                      │
│   └──────────┘      └──────────┘                                  │
│                                                                    │
│          ┌─────────────┐                                           │
│          │  db-net     │  Separate subnet.                        │
│          │ 172.19.0.1  │  Cannot reach web-net containers.        │
│          └─────────────┘                                           │
└────────────────────────────────────────────────────────────────────┘
```

**Rule of thumb:** Always create custom networks. Never rely on the default bridge.

---

## Port Mapping vs Network Connectivity

These are two completely different concepts that people confuse constantly.

### Port Mapping (`-p`)

Port mapping publishes a container's port to the **host machine**. It is for external access -- people or systems outside of Docker reaching into a container.

```bash
# Map container port 80 to host port 8080
docker run -d -p 8080:80 --name web nginx:alpine

# Now you can access the container from outside Docker:
curl http://localhost:8080     # From the host machine
curl http://192.168.1.50:8080  # From another machine on the network

# Format: -p <host_port>:<container_port>
```

### Network Connectivity

Network connectivity is about containers talking to **each other** within Docker networks. No host ports involved.

```bash
docker network create app-net

docker run -d --name api --network app-net my-api:1.0
docker run -d --name db --network app-net postgres:16

# api connects to db using the container name as hostname
# No port mapping needed for this communication
# api -> db happens entirely inside the Docker network
```

### When to Use Which

```
┌─────────────────────────────────────────────────────────────────┐
│                     Communication Path                           │
│                                                                  │
│   Outside World                                                  │
│        │                                                         │
│        │  <-- Port mapping (-p) for external access              │
│        ▼                                                         │
│   ┌──────────┐                                                   │
│   │ frontend │                                                   │
│   │ :3000    │                                                   │
│   └────┬─────┘                                                   │
│        │                                                         │
│        │  <-- Network connectivity (same Docker network)         │
│        ▼                                                         │
│   ┌──────────┐                                                   │
│   │ backend  │                                                   │
│   │ :8080    │                                                   │
│   └────┬─────┘                                                   │
│        │                                                         │
│        │  <-- Network connectivity (same Docker network)         │
│        ▼                                                         │
│   ┌──────────┐                                                   │
│   │ database │  Never needs port mapping                        │
│   │ :5432    │  Only reachable from within the network          │
│   └──────────┘                                                   │
└─────────────────────────────────────────────────────────────────┘
```

**The pattern:**
- Frontend needs `-p` because users access it from outside Docker.
- Backend does NOT need `-p` because only the frontend calls it.
- Database does NOT need `-p` because only the backend calls it.

Mapping database ports to the host is a security risk. Only do it during development for debugging with tools like pgAdmin or DBeaver.

---

## Container DNS and Service Discovery

Docker's embedded DNS server is the glue that makes service discovery work. Here is exactly how it operates.

### How DNS Resolution Works Inside a Container

```bash
# Create a network and start containers
docker network create demo-net
docker run -d --name web --network demo-net nginx:alpine
docker run -d --name api --network demo-net nginx:alpine

# From inside 'api', resolve 'web'
docker exec api nslookup web

# Output:
# Name:      web
# Address 1: 172.18.0.2 web.demo-net
```

The DNS server at `127.0.0.11` resolves `web` to `172.18.0.2`. The full qualified name is `web.demo-net` (container-name.network-name).

### Network Aliases

A container can have multiple DNS names using `--network-alias`:

```bash
# Start a database with an alias
docker run -d --name postgres-prod-abc123 \
  --network app-net \
  --network-alias database \
  postgres:16

# Other containers can reach it by either name:
# postgres-prod-abc123  (actual container name)
# database              (alias)

docker exec some-app nslookup database
# Resolves to the same container
```

Aliases are useful when you want a stable DNS name regardless of the container's actual name, which might include random suffixes in orchestrated environments.

### DNS Resolution Order

When a container tries to resolve a hostname, Docker checks in this order:

1. **Container name** on the same network
2. **Network alias** on the same network
3. **External DNS** (the host's DNS servers, like 8.8.8.8)

```bash
# Container can resolve other containers by name
docker exec api nslookup web        # -> 172.18.0.2

# Container can also resolve external domains
docker exec api nslookup google.com # -> 142.250.x.x
```

### Multiple Networks and Selective Connectivity

A container can be on multiple networks, but it can only reach containers on shared networks:

```bash
docker network create frontend-net
docker network create backend-net

# Gateway is on BOTH networks
docker run -d --name gateway --network frontend-net nginx:alpine
docker network connect backend-net gateway

# Web server is only on frontend-net
docker run -d --name web --network frontend-net nginx:alpine

# Database is only on backend-net
docker run -d --name db --network backend-net postgres:16

# Results:
# gateway can reach web       (shared: frontend-net)
# gateway can reach db        (shared: backend-net)
# web can reach gateway       (shared: frontend-net)
# web CANNOT reach db         (no shared network)
# db can reach gateway        (shared: backend-net)
# db CANNOT reach web         (no shared network)
```

This is how you build network isolation. The gateway acts as a bridge between the two networks, and the frontend never has direct access to the database.

---

## The Host and None Network Drivers

### Host Network

The `host` driver removes all network isolation. The container shares the host's network stack directly.

```bash
docker run -d --name web --network host nginx:alpine

# No port mapping needed -- the container IS on the host network
# If nginx listens on port 80, it is immediately available at host:80
curl http://localhost:80  # Works without -p
```

**Advantages:**
- Best possible network performance (no NAT, no virtual bridge)
- No port mapping overhead

**Disadvantages:**
- No network isolation between container and host
- Port conflicts between containers (two containers cannot both listen on port 80)
- Only works on Linux (Docker Desktop on Mac/Windows runs inside a VM, so `host` means the VM's network, not your machine)

**When to use:** High-performance applications where network latency matters, or containers that need to interact with host-level network services.

### None Network

The `none` driver disables networking entirely.

```bash
docker run -d --name isolated --network none alpine sleep 3600

# No network interfaces except loopback
docker exec isolated ip addr
# Only lo (127.0.0.1) -- no eth0

docker exec isolated ping google.com
# ping: bad address 'google.com'
```

**When to use:** Batch processing jobs that only read from mounted files and produce output to mounted volumes. Password cracking, video rendering, data transformation -- anything that does not need network access.

---

## Macvlan and IPvlan Networks

Sometimes you need containers to appear as first-class citizens on your physical network. Bridge networks are isolated virtual networks; macvlan and ipvlan make containers visible on the actual LAN.

### Macvlan

Macvlan assigns a unique MAC address to each container. From the network's perspective, the container looks like a physical machine.

```bash
# Create a macvlan network
# Parent is the host's physical interface
docker network create -d macvlan \
  --subnet=192.168.1.0/24 \
  --gateway=192.168.1.1 \
  -o parent=eth0 \
  my-macvlan-net

# Start a container with a specific IP
docker run -d --name direct-container \
  --network my-macvlan-net \
  --ip 192.168.1.100 \
  nginx:alpine

# This container is now accessible at 192.168.1.100
# From ANY machine on the 192.168.1.0/24 network
# No port mapping needed
curl http://192.168.1.100  # From any machine on the LAN
```

**Use case:** Legacy applications that expect to be on the same subnet as other services. Monitoring tools that need to see traffic on the physical network. Applications that use multicast or broadcast.

**Limitation:** Not all network infrastructure allows many MAC addresses per port. Cloud providers and some switches restrict the number of MAC addresses, which makes macvlan impractical in those environments.

### IPvlan

IPvlan is similar to macvlan but uses the parent interface's MAC address for all containers. Only the IP addresses differ.

```bash
docker network create -d ipvlan \
  --subnet=192.168.1.0/24 \
  --gateway=192.168.1.1 \
  -o parent=eth0 \
  -o ipvlan_mode=l2 \
  my-ipvlan-net

docker run -d --name ipvlan-container \
  --network my-ipvlan-net \
  --ip 192.168.1.101 \
  nginx:alpine
```

**Why use ipvlan over macvlan?** When your network infrastructure restricts MAC addresses per port. Since ipvlan uses one MAC for all containers, it works in more environments.

**L2 vs L3 mode:**
- `l2`: Containers are on the same subnet as the host (like macvlan)
- `l3`: Containers are on a separate subnet, routed through the host (works across subnets)

---

## Network Isolation and Security

Docker networks are isolated by default. Containers on different networks cannot communicate unless you explicitly connect them. This is a security feature.

### Building Secure Multi-Tier Architecture

```bash
# Create isolated networks for each tier
docker network create dmz        # Frontend tier
docker network create app        # Application tier
docker network create data       # Database tier

# Frontend: exposed to the internet
docker run -d --name nginx \
  --network dmz \
  -p 80:80 -p 443:443 \
  nginx:alpine

# Backend: accessible only from frontend
docker run -d --name api \
  --network app \
  my-api:1.0

# Database: accessible only from backend
docker run -d --name postgres \
  --network data \
  -e POSTGRES_PASSWORD=secret \
  postgres:16

# Gateway bridges the tiers
docker network connect dmz gateway
docker network connect app gateway
docker network connect data gateway
```

```
┌─────────────────────────────────────────────────────────────┐
│                      Internet                                │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────────┐
│                    DMZ Network                               │
│  ┌──────────┐      ┌──────────┐                             │
│  │  nginx   │      │ gateway  │  nginx is exposed via -p    │
│  │  :80     │──────│          │                             │
│  └──────────┘      └────┬─────┘                             │
└──────────────────────────┼──────────────────────────────────┘
                           │
┌──────────────────────────┼──────────────────────────────────┐
│                    App Network                               │
│                      ┌────┴─────┐                            │
│                      │ gateway  │                            │
│                      │          │                            │
│                      └────┬─────┘                            │
│                           │                                  │
│                      ┌────┴─────┐                            │
│                      │   api    │  api has NO port mapping   │
│                      │  :8080   │  invisible to outside      │
│                      └──────────┘                            │
└──────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────┐
│                    Data Network                              │
│                      ┌──────────┐                            │
│                      │ gateway  │                            │
│                      │          │                            │
│                      └────┬─────┘                            │
│                           │                                  │
│                      ┌────┴─────┐                            │
│                      │ postgres │  postgres has NO port      │
│                      │  :5432   │  mapping. Invisible.       │
│                      └──────────┘                            │
└──────────────────────────────────────────────────────────────┘
```

**Security properties:**
- The internet can only reach `nginx`.
- `nginx` can only reach `gateway` (not `api` or `postgres` directly).
- `api` can only reach `gateway` (not `nginx` or `postgres` directly).
- `postgres` can only reach `gateway` (not `nginx` or `api` directly).
- `gateway` is the only container on all three networks.

This is the principle of least privilege applied to networking.

### Restricting Inter-Container Traffic

By default, containers on the same network can freely communicate. You can restrict this with the `--icc` flag on the Docker daemon:

```json
// /etc/docker/daemon.json
{
  "icc": false
}
```

With `icc: false`, containers on the same bridge network cannot talk to each other unless explicitly linked or using `--link` (deprecated). You must use user-defined networks and be explicit about connectivity.

---

## Inspecting Networks

Docker gives you several commands to see exactly what is happening on your networks.

### List Networks

```bash
docker network ls

# Output:
# NETWORK ID     NAME          DRIVER    SCOPE
# a1b2c3d4e5f6   bridge        bridge    local
# b2c3d4e5f6a1   frontend-net  bridge    local
# c3d4e5f6a1b2   backend-net   bridge    local
# d4e5f6a1b2c3   host          host      local
# e5f6a1b2c3d4   none          null      local
```

### Inspect a Network

```bash
docker network inspect frontend-net
```

This returns JSON with everything about the network:

```json
[
    {
        "Name": "frontend-net",
        "Id": "b2c3d4e5f6a1...",
        "Created": "2024-01-15T10:30:00.000000000Z",
        "Scope": "local",
        "Driver": "bridge",
        "EnableIPv6": false,
        "IPAM": {
            "Driver": "default",
            "Config": [
                {
                    "Subnet": "172.18.0.0/16",
                    "Gateway": "172.18.0.1"
                }
            ]
        },
        "Containers": {
            "abc123...": {
                "Name": "nginx",
                "EndpointID": "...",
                "MacAddress": "02:42:ac:12:00:02",
                "IPv4Address": "172.18.0.2/16"
            },
            "def456...": {
                "Name": "gateway",
                "EndpointID": "...",
                "MacAddress": "02:42:ac:12:00:03",
                "IPv4Address": "172.18.0.3/16"
            }
        }
    }
]
```

Key fields:
- **Subnet/Gateway**: The IP range for this network
- **Containers**: Every container on this network with its IP and MAC address

### Inspect a Container's Network Settings

```bash
docker inspect --format='{{json .NetworkSettings.Networks}}' nginx
```

```json
{
    "frontend-net": {
        "IPAMConfig": null,
        "Links": null,
        "Aliases": null,
        "NetworkID": "b2c3d4e5f6a1...",
        "EndpointID": "...",
        "Gateway": "172.18.0.1",
        "IPAddress": "172.18.0.2",
        "IPPrefixLen": 16,
        "MacAddress": "02:42:ac:12:00:02"
    }
}
```

### List Containers on a Network

```bash
docker network inspect --format='{{range .Containers}}{{.Name}} {{end}}' frontend-net
# Output: nginx gateway
```

---

## Debugging Network Issues

When networking does not work, you need to systematically diagnose the problem. Here is the troubleshooting playbook.

### Step 1: Are the Containers on the Same Network?

```bash
# Check which networks a container is on
docker inspect --format='{{range $net, $config := .NetworkSettings.Networks}}{{$net}} {{end}}' container-name

# Example:
docker inspect --format='{{range $net, $config := .NetworkSettings.Networks}}{{$net}} {{end}}' backend
# Output: frontend-net

docker inspect --format='{{range $net, $config := .NetworkSettings.Networks}}{{$net}} {{end}}' database
# Output: backend-net

# These are DIFFERENT networks -- they cannot communicate!
```

**Fix:** Put them on the same network, or connect one to the other's network:
```bash
docker network connect backend-net backend
```

### Step 2: Can Containers Reach Each Other by IP?

```bash
# Get the target container's IP
docker inspect --format='{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' database
# Output: 172.19.0.2

# Ping from the source container
docker exec backend ping -c 3 172.19.0.2

# If ping works: DNS is the problem, not connectivity
# If ping fails: network or firewall issue
```

### Step 3: Does DNS Resolution Work?

```bash
# Check if DNS resolves
docker exec backend nslookup database
# or
docker exec backend getent hosts database

# If DNS fails but IP ping works:
# - Are they on the same custom network? (not default bridge)
# - Is the target container running?
# - Is the container name correct?
```

### Step 4: Is the Target Service Actually Listening?

```bash
# Check what ports the target container is listening on
docker exec database netstat -tlnp
# or
docker exec database ss -tlnp

# Connect to the target port
docker exec backend nc -zv database 5432
# Connection to database (172.19.0.2) 5432 port [tcp/postgresql] succeeded!

# If the port is not listening:
# - Check the application logs: docker logs database
# - Is the service configured to listen on 0.0.0.0 (not just 127.0.0.1)?
# - Is the service actually started?
```

### Step 5: Check Container Logs and Firewall

```bash
# Application logs
docker logs database

# Check if iptables rules are blocking traffic
# (run on the host)
sudo iptables -L -n -v

# Check if Docker's userland proxy is working
# Look for the docker-proxy process
ps aux | grep docker-proxy
```

### Common Debugging Tools Inside Containers

Many minimal containers (Alpine, distroless) lack debugging tools. You can install them temporarily:

```bash
# For Alpine-based containers
docker exec -it backend sh -c "apk add --no-cache curl bind-tools netcat-openbsd"

# For Debian/Ubuntu-based containers
docker exec -it backend sh -c "apt-get update && apt-get install -y curl dnsutils netcat-openbsd"

# For quick debugging, use a dedicated debug container on the same network
docker run -it --network my-network nicolaka/netshoot
# This image has: ping, curl, dig, nslookup, tcpdump, iperf, wireshark, etc.
```

### The netshoot Debug Container

The `nicolaka/netshoot` image is the standard debugging toolkit for container networking:

```bash
# Launch it on the network you want to debug
docker run -it --network my-network nicolaka/netshoot

# Inside netshoot, you have every tool:
ping backend
nslookup database
curl http://backend:8080/health
dig database.my-network
tcpdump -i eth0
iperf3 -c backend
traceroute database
```

---

## Real-World Pattern: Frontend-Backend-Database

Here is a complete, production-style networking setup for a typical web application.

### The Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                        Host Machine                               │
│                                                                   │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                   frontend-net (172.20.0.0/16)             │  │
│  │                                                            │  │
│  │   ┌──────────┐        ┌──────────┐        ┌──────────┐   │  │
│  │   │  nginx   │        │   app    │        │  redis   │   │  │
│  │   │ 172.20.  │────────│ 172.20.  │────────│ 172.20.  │   │  │
│  │   │ 0.2      │        │ 0.3      │        │ 0.4      │   │  │
│  │   └──────────┘        └────┬─────┘        └──────────┘   │  │
│  │                             │                              │  │
│  └─────────────────────────────┼──────────────────────────────┘  │
│                                │                                  │
│  ┌─────────────────────────────┼──────────────────────────────┐  │
│  │                   backend-net (172.21.0.0/16)               │  │
│  │                             │                              │  │
│  │                      ┌──────┴─────┐       ┌──────────┐   │  │
│  │                      │    app     │       │ postgres │   │  │
│  │                      │ 172.21.    │───────│ 172.21.  │   │  │
│  │                      │ 0.2        │       │ 0.3      │   │  │
│  │                      └────────────┘       └──────────┘   │  │
│  │                                                            │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                   │
│   Port mapping: -p 80:80 on nginx only                           │
│   postgres: NO port mapping (invisible to outside)               │
│   redis: NO port mapping (invisible to outside)                  │
└──────────────────────────────────────────────────────────────────┘
```

### Implementation

```bash
#!/bin/bash
# setup-networks.sh

# Create networks
docker network create frontend-net
docker network create backend-net

# Database: only on backend network, no port mapping
docker run -d \
  --name postgres \
  --network backend-net \
  --network-alias database \
  -e POSTGRES_USER=app \
  -e POSTGRES_PASSWORD=secret123 \
  -e POSTGRES_DB=myapp \
  postgres:16-alpine

# Redis: only on frontend network (for caching)
docker run -d \
  --name redis \
  --network frontend-net \
  --network-alias cache \
  redis:7-alpine

# Application: on BOTH networks (can reach database and redis)
docker run -d \
  --name app \
  --network frontend-net \
  -e DATABASE_URL=postgresql://app:secret123@database:5432/myapp \
  -e REDIS_URL=redis://cache:6379 \
  my-app:1.0

# Connect app to backend network (now it can reach postgres)
docker network connect backend-net app

# Nginx: frontend network only, port mapping for external access
docker run -d \
  --name nginx \
  --network frontend-net \
  -p 80:80 -p 443:443 \
  -v ./nginx.conf:/etc/nginx/nginx.conf:ro \
  nginx:alpine
```

### The Key Insight

The `app` container is on two networks. It can reach `redis` (via `frontend-net`) and `postgres` (via `backend-net`). But `nginx` can only reach `app` and `redis` -- it cannot see `postgres` at all. And `postgres` is completely invisible from outside Docker.

This is defense in depth. Even if someone compromises `nginx`, they have no direct path to the database.

### With Docker Compose

Docker Compose makes this much simpler:

```yaml
# docker-compose.yml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    networks:
      - frontend

  app:
    build: ./app
    environment:
      - DATABASE_URL=postgresql://app:secret123@database:5432/myapp
      - REDIS_URL=redis://cache:6379
    networks:
      - frontend
      - backend

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: app
      POSTGRES_PASSWORD: secret123
      POSTGRES_DB: myapp
    networks:
      backend:
        aliases:
          - database

  redis:
    image: redis:7-alpine
    networks:
      frontend:
        aliases:
          - cache

networks:
  frontend:
  backend:
```

Compose automatically creates `frontend` and `backend` networks and connects each service to the right one. The `app` service is on both networks because it lists both. The `aliases` section provides stable DNS names.

---

## Hands-On Lab

### Lab 1: Network Isolation

**Goal:** Prove that containers on different networks cannot communicate.

```bash
# Create two separate networks
docker network create isolated-a
docker network create isolated-b

# Start containers on different networks
docker run -d --name container-a --network isolated-a alpine sleep 3600
docker run -d --name container-b --network isolated-b alpine sleep 3600

# Try to ping from A to B (should fail)
docker exec container-a ping -c 2 -W 2 container-b
# ping: bad address 'container-b'

# Try by IP (get B's IP first)
docker inspect --format='{{range .NetworkSettings.Networks}}{{.IPAddress}}{{end}}' container-b
# e.g., 172.20.0.2

docker exec container-a ping -c 2 -W 2 172.20.0.2
# Network is unreachable

# Clean up
docker rm -f container-a container-b
docker network rm isolated-a isolated-b
```

### Lab 2: Custom Bridge with DNS

**Goal:** Demonstrate DNS resolution between containers on a custom network.

```bash
# Create a custom network
docker network create lab-net

# Start an nginx container
docker run -d --name web --network lab-net nginx:alpine

# Start an Alpine container and resolve 'web'
docker run --rm --network lab-net alpine sh -c "
  apk add --no-cache bind-tools curl > /dev/null 2>&1
  echo '=== DNS Resolution ==='
  nslookup web
  echo ''
  echo '=== HTTP Request ==='
  curl -s http://web | head -5
  echo ''
  echo '=== Connection Test ==='
  nc -zv web 80
"

# Clean up
docker rm -f web
docker network rm lab-net
```

### Lab 3: Multi-Network Connectivity

**Goal:** Connect a gateway container to multiple networks.

```bash
# Create two networks
docker network create public-net
docker network create private-net

# Database: private network only
docker run -d --name db --network private-net \
  -e POSTGRES_PASSWORD=test \
  postgres:16-alpine

# App: starts on private network, then connect to public
docker run -d --name app --network private-net alpine sleep 3600
docker network connect public-net app

# Frontend: public network only
docker run -d --name frontend --network public-net alpine sleep 3600

# Verify connectivity
echo "=== frontend -> app (should work, both on public-net) ==="
docker exec frontend ping -c 1 -W 2 app

echo "=== frontend -> db (should FAIL, no shared network) ==="
docker exec frontend ping -c 1 -W 2 db || echo "Failed as expected"

echo "=== app -> db (should work, both on private-net) ==="
docker exec app ping -c 1 -W 2 db

echo "=== app -> frontend (should work, both on public-net) ==="
docker exec app ping -c 1 -W 2 frontend

# Clean up
docker rm -f db app frontend
docker network rm public-net private-net
```

### Lab 4: Network Aliases

**Goal:** Use aliases for stable DNS names.

```bash
docker network create alias-net

# Container with a random name but stable alias
docker run -d --name postgres-abc123xyz \
  --network alias-net \
  --network-alias database \
  --network-alias db \
  alpine sleep 3600

# Resolve by different names
docker run --rm --network alias-net alpine sh -c "
  apk add --no-cache bind-tools > /dev/null 2>&1
  echo 'By container name:'
  getent hosts postgres-abc123xyz
  echo ''
  echo 'By alias database:'
  getent hosts database
  echo ''
  echo 'By alias db:'
  getent hosts db
  echo ''
  echo 'All point to the same IP!'
"

# Clean up
docker rm -f postgres-abc123xyz
docker network rm alias-net
```

### Lab 5: Debugging with netshoot

**Goal:** Use professional debugging tools to diagnose a networking problem.

```bash
# Setup: create a network with a service
docker network create debug-net
docker run -d --name myservice --network debug-net nginx:alpine

# Launch netshoot for debugging
docker run -it --rm --network debug-net nicolaka/netshoot sh -c "
  echo '=== DNS Resolution ==='
  dig myservice +short

  echo ''
  echo '=== HTTP Connectivity ==='
  curl -s -o /dev/null -w 'HTTP Status: %{http_code}\n' http://myservice

  echo ''
  echo '=== Network Path ==='
  traceroute -n myservice

  echo ''
  echo '=== Port Scan ==='
  nmap -sT -p 80 myservice 2>/dev/null | grep -A 3 'PORT'

  echo ''
  echo '=== ARP Table ==='
  arp -a
"

# Clean up
docker rm -f myservice
docker network rm debug-net
```

### Lab 6: Complete Application Stack

**Goal:** Build a real three-tier application with proper network isolation.

```bash
# Create the networks
docker network create web-tier
docker network create app-tier

# Database tier
docker run -d \
  --name postgres \
  --network app-tier \
  --network-alias db \
  -e POSTGRES_USER=appuser \
  -e POSTGRES_PASSWORD=apppass \
  -e POSTGRES_DB=appdb \
  postgres:16-alpine

# Application tier (connected to both)
docker run -d \
  --name api \
  --network app-tier \
  --network-alias backend \
  hashicorp/http-echo:0.2.3 \
  -listen=:8080 \
  -text='{"status":"ok","message":"API running"}'

docker network connect web-tier api

# Web tier (exposed to host)
docker run -d \
  --name web \
  --network web-tier \
  -p 8080:80 \
  nginx:alpine

# Test the stack
echo "=== From the host ==="
curl -s http://localhost:8080

echo ""
echo "=== Network layout ==="
echo "web  -> $(docker inspect --format='{{range .NetworkSettings.Networks}}{{.IPAddress}} {{end}}' web)"
echo "api  -> $(docker inspect --format='{{range .NetworkSettings.Networks}}{{.IPAddress}} {{end}}' api)"
echo "db   -> $(docker inspect --format='{{range .NetworkSettings.Networks}}{{.IPAddress}} {{end}}' postgres)"

echo ""
echo "=== Isolation test: web cannot reach db ==="
docker exec web ping -c 1 -W 2 db 2>&1 || echo "Correctly isolated!"

# Clean up
docker rm -f web api postgres
docker network rm web-tier app-tier
```

---

## Quick Reference

### Essential Commands

```bash
# Network management
docker network ls                           # List all networks
docker network create my-net                # Create a bridge network
docker network create -d macvlan ...        # Create with specific driver
docker network rm my-net                    # Remove a network
docker network prune                        # Remove all unused networks

# Inspect
docker network inspect my-net               # Full network details
docker network inspect my-net --format='{{.Containers}}'

# Connect/disconnect containers
docker network connect my-net container     # Add container to network
docker network connect --ip 172.18.0.100 my-net container  # With specific IP
docker network disconnect my-net container  # Remove from network

# Debugging
docker exec container ping target           # Test connectivity
docker exec container nslookup target       # Test DNS
docker exec container curl http://target    # Test HTTP
docker exec container nc -zv target 80      # Test specific port
docker logs container                       # Check application logs
```

### Network Driver Cheat Sheet

| Driver | Scope | Use Case | DNS Support |
|--------|-------|----------|-------------|
| `bridge` (custom) | Single host | Default for multi-container apps | Yes |
| `bridge` (default) | Single host | Legacy, avoid for new work | No |
| `host` | Single host | Maximum performance, Linux only | N/A |
| `none` | Single host | Completely isolated workloads | N/A |
| `overlay` | Multi-host | Docker Swarm cross-host networking | Yes |
| `macvlan` | Single host | Container on physical network | Yes |
| `ipvlan` | Single host | Like macvlan, shared MAC address | Yes |

---

## Limitation: Networking Works, But Data Disappears

You now understand how containers find each other, communicate, and stay isolated. Your multi-tier application can talk across networks with proper security boundaries.

But there is a problem. Watch what happens:

```bash
docker run -d --name postgres -e POSTGRES_PASSWORD=secret postgres:16
# ... application stores data ...
docker rm -f postgres
# All data is GONE.
```

Every container has its own filesystem. When the container is removed, its filesystem is destroyed. Volumes, database records, uploaded files, log data -- everything vanishes.

In production, this is unacceptable. You need data to survive container restarts, removals, and even host failures.

**Next module:** [09-volumes-and-data](../09-volumes-and-data/) -- Persist data across container lifecycles with Docker volumes and bind mounts.

---

## Key Terms

| Term | Definition |
|------|-----------|
| **Bridge Network** | A virtual network on a single host that connects containers |
| **Network Driver** | The mechanism Docker uses to create a network (bridge, host, overlay, etc.) |
| **Port Mapping** | Publishing a container's port to the host machine (`-p`) |
| **DNS Resolution** | Translating container names to IP addresses inside Docker networks |
| **Network Alias** | An alternative DNS name for a container on a network |
| **Network Isolation** | Containers on different networks cannot communicate |
| **Macvlan** | Network driver that makes containers appear as physical devices on the LAN |
| **IPvlan** | Like macvlan but shares the parent interface's MAC address |
| **Overlay Network** | A network that spans multiple Docker hosts (used with Swarm) |
| **ICC** | Inter-Container Communication -- whether containers on the same bridge can talk |
| **netshoot** | A debugging container image with comprehensive network tools |

## Checklist

- [ ] I understand the difference between port mapping and network connectivity
- [ ] I know why the default bridge network is inadequate for multi-container apps
- [ ] I can create custom bridge networks and connect containers to them
- [ ] I understand how Docker DNS resolves container names to IP addresses
- [ ] I can use network aliases for stable service discovery
- [ ] I can connect a container to multiple networks for tiered architecture
- [ ] I know when to use host, none, macvlan, and ipvlan network drivers
- [ ] I can inspect networks with `docker network inspect` and interpret the output
- [ ] I can debug network issues with ping, nslookup, curl, and netshoot
- [ ] I understand how to build a secure multi-tier network architecture
- [ ] I am ready to learn about persistent data storage
