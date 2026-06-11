# Solution 01: Load Balancing Algorithm Selection

## Part A: Algorithm Characteristics

| Algorithm | How It Works | Strength | Weakness |
|-----------|-------------|----------|----------|
| **Round-Robin** | Cycles through backends sequentially: 1, 2, 3, 1, 2, 3... | Simple, no state required, perfectly even distribution | Ignores server capacity and current load |
| **Weighted Round-Robin** | Like round-robin but assigns more requests to higher-weight servers | Accounts for different server capacities | Still ignores real-time load; weights are static |
| **Least-Connections** | Routes to the server with the fewest active connections | Adapts to real-time load; good for varying request durations | Requires tracking connections; connection count may not equal actual load |
| **IP Hash** | Hashes client IP to select a server; same IP always goes to same server | Provides session persistence without external state | NAT causes imbalance; adding/removing servers reassigns many clients |
| **Random** | Selects a server randomly (optionally weighted) | Simple, no state, good with large request volumes | Can produce short-term imbalances; no persistence |

### Why These Characteristics Matter

Each algorithm uses different information to make routing decisions:
- Round-robin: just the request count (no server state)
- Least-connections: active connection count (server state)
- IP hash: client identity (request property)
- Random: nothing (pure chance)

The more information the algorithm uses, the better its decisions -- but the more state it must maintain.

## Part B: Match Algorithm to Service

| Service | Best Algorithm | Why |
|---------|---------------|-----|
| **A (Stateless REST API)** | Round-Robin | Requests are uniform (~50ms each) and servers have equal capacity. Round-robin provides perfectly even distribution with zero overhead. No need for intelligence. |
| **B (WebSocket server)** | Least-Connections | Long-lived connections accumulate unevenly. Least-connections prevents any single server from exceeding its 5,000 connection limit by routing new connections to the least-loaded server. |
| **C (E-commerce cart)** | IP Hash | Session persistence is required (shopping cart in server memory). IP hash ensures the same client always reaches the same server without external session storage. |
| **D (Image processing)** | Least-Connections | Requests vary wildly (100ms to 30s). A server processing a 30s request should not receive new requests. Least-connections routes to the server with the fewest active (possibly slow) connections. |
| **E (Mixed capacity)** | Weighted Round-Robin | Servers have different capacities. Assign weights proportional to capacity (e.g., 2:2:1 for two high-capacity and one low-capacity server). |

### Alternative Consideration for Service C

Instead of IP hash, you could store cart state in Redis and use round-robin or least-connections. This is often better because:
- IP hash breaks when clients change IPs (mobile networks, VPNs)
- Redis provides persistence across server restarts
- It decouples session state from server identity

## Part C: Failure Scenarios

### 1. Round-Robin with 4 servers, Server 2 goes down

```
Before failure:  1 → 2 → 3 → 4 → 1 → 2 → 3 → 4
After failure:   1 → [timeout] → 3 → 4 → 1 → [timeout] → 3 → 4

Behavior:
- Round-robin continues cycling through ALL servers (including the failed one)
- Every 4th request hits Server 2 and fails (until health check removes it)
- Failure rate: 25% of requests fail
- Detection: Passive health check (request failure) or active health check
- Recovery time: Depends on health check configuration (typically 10-30 seconds)
```

**Problem:** Basic round-robin does not detect failures. It continues sending traffic to the dead server until a health check removes it. This is why production load balancers always combine round-robin with health checks.

### 2. Least-Connections with 3 servers, Server 1 (100 connections) goes down

```
Before failure:
  Server 1: 100 connections ← fails
  Server 2: 50 connections
  Server 3: 60 connections

After failure detected:
  Server 1: removed from pool
  Server 2: 50 connections (receives all new connections)
  Server 3: 60 connections (receives all new connections)

Behavior:
- New connections route to Server 2 (fewer connections) first
- Existing 100 connections on Server 1 are dropped (TCP RST or timeout)
- Clients must reconnect; reconnection goes to Server 2 or 3
- Server 2 and 3 absorb the load from Server 1
```

**Problem:** Existing connections are lost. For stateless HTTP, clients retry. For WebSocket or database connections, the application must handle reconnection.

### 3. IP Hash with 3 servers, Server 3 goes down

```
Before failure:
  hash(IP) % 3 = 0 → Server 1
  hash(IP) % 3 = 1 → Server 2
  hash(IP) % 3 = 2 → Server 3 ← fails

After failure (server removed, now 2 servers):
  hash(IP) % 2 = 0 → Server 1
  hash(IP) % 2 = 1 → Server 2

Problem:
  Clients that were on Server 3 get reassigned (expected)
  BUT: clients on Server 1 and Server 2 may ALSO get reassigned
  because the modulo changed from %3 to %2

Example:
  Client A: hash=5, 5%3=2 (Server 3), 5%2=1 (Server 2) → reassigned
  Client B: hash=4, 4%3=1 (Server 2), 4%2=0 (Server 1) → reassigned!
  Client C: hash=3, 3%3=0 (Server 1), 3%2=1 (Server 2) → reassigned!
```

**Problem:** Changing the number of servers causes massive reassignment. This is the "hash ring disruption" problem. Consistent hashing solves this by ensuring only ~1/N of clients are reassigned when a server is added or removed.

### Common Mistakes to Avoid

- **Assuming round-robin handles failures.** It does not. Always combine with health checks.
- **Using IP hash behind NAT.** All users from one corporate network appear as the same IP, hitting one server.
- **Confusing connection count with load.** A server with 10 idle WebSocket connections is less loaded than a server with 5 active HTTP requests.
- **Ignoring the cost of state tracking.** Least-connections requires the load balancer to track every connection. At 100K connections, this consumes memory and CPU.

## Key Takeaway

Load balancing algorithms are not interchangeable. Round-robin is best for uniform, stateless workloads. Least-connections adapts to real-time load and handles variable request durations. IP hash provides session persistence but suffers from hash disruption. The right choice depends on request characteristics, session requirements, and whether backends have equal capacity. In production, always combine any algorithm with health checks.
