# Solution 03: Least-Connections vs IP Hash

## Part A: Least-Connections Deep Dive

### 1. Connection Distribution Diagram

```
Initial state:
  Server-A: 5 connections
  Server-B: 3 connections  ← fewest
  Server-C: 8 connections

7 new connections arrive:

  New Conn 1 → Server-B (3→4, now fewest is B with 4)
  New Conn 2 → Server-B (4→5, tied with A)
  New Conn 3 → Server-A (5→6, B=5 is now fewest)
  New Conn 4 → Server-B (5→6, tied with A)
  New Conn 5 → Server-A (6→7, B=6 is now fewest)
  New Conn 6 → Server-B (6→7, tied with A)
  New Conn 7 → Server-A (7→8, B=7 is now fewest)

Final state:
  Server-A: 5 + 3 = 8 connections
  Server-B: 3 + 4 = 7 connections
  Server-C: 8 + 0 = 8 connections
```

### Why Server-C Gets No New Connections

Server-C started with the most connections (8). The algorithm always
prefers servers with fewer connections. Until Server-A and Server-B
catch up to Server-C's count, Server-C receives nothing. This is the
algorithm working as designed: it prevents overloading already-loaded
servers.

### 2. Burst of Long-Lived Connections to Server-B

```
Scenario: Server-B receives 50 new WebSocket connections in 10 seconds

Time 0s:   A=8,  B=7,  C=8   → New connections go to B (fewest)
Time 2s:   A=8,  B=20, C=8   → New connections go to A or C
Time 5s:   A=15, B=40, C=15  → New connections go to A or C
Time 10s:  A=25, B=57, C=25  → B is now most loaded

Behavior:
1. Initially, B receives connections because it has the fewest
2. As B accumulates connections, the algorithm shifts to A and C
3. Eventually B stabilizes at a higher count (long-lived connections)
4. New connections distribute evenly between A and C
5. Over time (hours), as B's connections close, B becomes attractive again

The algorithm self-corrects but cannot prevent the initial burst.
```

### 3. Suboptimal Decision Scenario

```
Scenario: Server processing time varies

  Server-A: 10 connections, each processing a 1ms request
  Server-B: 5 connections, each processing a 30-second request

  Least-connections says: route to Server-B (fewer connections)
  Reality: Server-B is MUCH more loaded (5 × 30s = 150s of work)
           Server-A is lightly loaded (10 × 1ms = 10ms of work)

  Result: New requests queue behind Server-B's slow requests
          Response time increases dramatically
```

**This is the fundamental limitation of least-connections:** it counts
connections, not actual load. Connection count is a proxy for load, but
it breaks when request durations vary significantly.

## Part B: IP Hash Deep Dive

### 1. Client-to-Server Mapping

For IP hash, we need to compute `hash(IP) % 3`. Using a simple numeric
representation of the IP:

```
Client IP: 192.168.1.100
  Numeric: 192*256^3 + 168*256^2 + 1*256 + 100 = 3232235876
  hash % 3 = 3232235876 % 3 = 2 → Server-C

Client IP: 10.0.0.50
  Numeric: 10*256^3 + 0*256^2 + 0*256 + 50 = 167772210
  hash % 3 = 167772210 % 3 = 2 → Server-C

Client IP: 172.16.0.200
  Numeric: 172*256^3 + 16*256^2 + 0*256 + 200 = 2886730168
  hash % 3 = 2886730168 % 3 = 0 → Server-A

Client IP: 192.168.1.100 (reconnect)
  Same hash → Server-C  ← Session persistence working

Client IP: 192.168.1.101
  Numeric: 3232235877
  hash % 3 = 3232235877 % 3 = 0 → Server-A
```

```
Summary:
  192.168.1.100  → Server-C
  10.0.0.50      → Server-C
  172.16.0.200   → Server-A
  192.168.1.100  → Server-C  (same client, same server)
  192.168.1.101  → Server-A
```

### 2. Adding a 4th Server

```
Before (3 servers): hash % 3
After  (4 servers): hash % 4

  192.168.1.100: 3232235876 % 3 = 2 (Server-C)
                 3232235876 % 4 = 0 (Server-A) → REASSIGNED

  10.0.0.50:     167772210 % 3 = 2 (Server-C)
                 167772210 % 4 = 2 (Server-C) → same

  172.16.0.200:  2886730168 % 3 = 0 (Server-A)
                 2886730168 % 4 = 0 (Server-A) → same

Expected reassignment: ~75% of clients (N-1/N where N is new server count)
```

**This is a major problem.** Adding one server to a 3-server cluster
reassigns approximately 75% of clients. Their sessions are lost.

**Solution: Consistent Hashing**

```
Consistent hashing uses a hash ring (0 to 2^32-1).
Each server is placed at multiple points on the ring.
Each client is mapped to the nearest server clockwise on the ring.

Adding a server: only clients between the new server and the next
server clockwise are reassigned (~1/N of total clients).

Removing a server: only clients mapped to that server are reassigned.

Result: Adding a 4th server reassigns ~25% of clients instead of ~75%.
```

### 3. NAT/Masquerading Problem

```
Scenario: 500 employees at a corporate office all appear as 203.0.113.50

  hash(203.0.113.50) % 3 = X → Server-X

All 500 users go to Server-X.
Server-X receives 500 users while Server-Y and Server-Z get 0.

If the office has 10,000 users, Server-X is overwhelmed while
the other servers sit idle.
```

**Mitigation:**
- Use IP hash combined with a cookie or session ID (X-Forwarded-For header)
- Use least-connections instead (no persistence needed if state is external)
- Use consistent hashing with additional factors (User-Agent, path)

## Part C: Algorithm Comparison

| Criterion | Least-Connections | IP Hash |
|-----------|-------------------|---------|
| **Session persistence** | No -- requests from same client may go to different servers | Yes -- same client IP always goes to same server |
| **Load distribution accuracy** | High -- adapts to real-time connection count | Variable -- depends on IP distribution; NAT causes imbalance |
| **Behavior when adding a server** | New server receives connections as it has 0; gradual rebalancing | ~N-1/N of clients reassigned; consistent hashing reduces to ~1/N |
| **Behavior when removing a server** | Existing connections dropped; new connections redistribute | All clients on removed server reassigned; others unaffected |
| **Handling of varying request durations** | Good -- server with slow requests has fewer new connections | Poor -- does not consider request duration at all |
| **Handling of NAT/masquerading** | No impact -- connection count is independent of client IP | Major impact -- all NAT'd users hit same server |
| **State required on load balancer** | Active connection count per server | Client-to-server mapping (computed, not stored) |
| **Best for** | Variable-duration requests, stateless services | Stateful services requiring session persistence |

## Part D: Design Decision

### 1. Chat Service: Least-Connections

**Choice:** Least-connections with a maximum connection limit per server.

**Reasoning:**
- WebSocket connections are long-lived (avg 2 hours)
- Each server has a 5,000 connection limit
- Least-connections prevents any server from exceeding its limit
- New users are routed to the server with the fewest current connections
- Connection count is a good proxy for load in this case (each connection
  has similar resource cost)

```
Configuration:
  upstream chat_backend {
    least_conn;
    server 10.0.1.1:8080 max_conns=5000;
    server 10.0.1.2:8080 max_conns=5000;
    server 10.0.1.3:8080 max_conns=5000;
  }
```

### 2. E-Commerce API: IP Hash (with caveats)

**Choice:** IP hash for session persistence, BUT with a plan to externalize session state.

**Reasoning:**
- Shopping cart state is in server memory (requires session persistence)
- IP hash provides persistence without external storage
- However, IP hash has problems (NAT, server changes)

**Better long-term solution:**
- Move cart state to Redis
- Use round-robin or least-connections (no persistence needed)
- This decouples session state from server identity

```
Short-term (IP hash):
  upstream ecom_backend {
    ip_hash;
    server 10.0.1.1:8080;
    server 10.0.1.2:8080;
    server 10.0.1.3:8080;
  }

Long-term (externalized state):
  upstream ecom_backend {
    least_conn;
    server 10.0.1.1:8080;
    server 10.0.1.2:8080;
    server 10.0.1.3:8080;
  }
  # Cart state stored in Redis, not server memory
```

### 3. Combining Both Approaches

You can combine algorithms using path-based routing:

```nginx
upstream chat_backend {
    least_conn;
    server 10.0.1.1:8080;
    server 10.0.1.2:8080;
}

upstream ecom_backend {
    ip_hash;
    server 10.0.1.3:8080;
    server 10.0.1.4:8080;
}

server {
    location /ws/ {
        proxy_pass http://chat_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    location /api/ {
        proxy_pass http://ecom_backend;
    }
}
```

### Common Mistakes to Avoid

- **Using least-connections for stateful services.** If session state is in
  server memory, least-connections will break sessions by routing clients
  to different servers.
- **Using IP hash without considering NAT.** Corporate and mobile users
  often share IPs. IP hash can create severe imbalances.
- **Confusing connection count with load.** A server with 10 idle
  connections is less loaded than a server with 5 active connections.
  Consider using "least response time" if available.
- **Not planning for server topology changes.** IP hash without consistent
  hashing causes massive session loss when servers are added or removed.

## Key Takeaway

Least-connections and IP hash solve fundamentally different problems.
Least-connections optimizes for even load distribution by tracking active
connections -- ideal for stateless services with varying request durations.
IP hash provides session persistence by mapping clients to servers -- ideal
for stateful applications that cannot externalize session state. The best
choice depends on whether you need load accuracy or session affinity, and
the long-term answer is often to externalize state and use the simpler
algorithm.
