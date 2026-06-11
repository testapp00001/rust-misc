# Solution 05: Multi-Tier Load Balancing Architecture

## Part A: Architecture Design

```
Multi-Tier Load Balancing Architecture
═══════════════════════════════════════

                        Internet Users
                       (US, EU, APAC)
                             │
                             ▼
┌─────────────────────────────────────────────────────┐
│  Tier 1: DNS/Global Load Balancer                   │
│  Technology: Route 53 / Cloudflare                  │
│  Algorithm: Latency-based / Geographic routing      │
│  Distributes: DNS queries (routes users to nearest  │
│               regional endpoint)                    │
│  Health check: HTTP endpoint every 30s              │
└────────────┬──────────────┬──────────────┬──────────┘
             │              │              │
     ┌───────┴──────┐ ┌────┴─────┐ ┌─────┴──────┐
     │  US-East     │ │ EU-West  │ │ AP-South   │
     │  Region      │ │ Region   │ │ Region     │
     └───────┬──────┘ └────┬─────┘ └─────┬──────┘
             │              │              │
             ▼              ▼              ▼
┌─────────────────────────────────────────────────────┐
│  Tier 2: Regional L4 Load Balancer                  │
│  Technology: HAProxy / AWS NLB / MetalLB            │
│  Algorithm: Least-connections                        │
│  Distributes: TCP connections to L7 load balancers  │
│  Health check: TCP check every 5s                   │
│  Purpose: High throughput, low overhead             │
└────────────┬──────────────┬──────────────┬──────────┘
             │              │              │
     ┌───────┴──────┐ ┌────┴─────┐ ┌─────┴──────┐
     │  LB-1        │ │ LB-2     │ │ LB-3       │
     │  (Nginx)     │ │ (Nginx)  │ │ (Nginx)    │
     └───────┬──────┘ └────┬─────┘ └─────┬──────┘
             │              │              │
             ▼              ▼              ▼
┌─────────────────────────────────────────────────────┐
│  Tier 3: Application L7 Load Balancer               │
│  Technology: Nginx / Envoy / Traefik                │
│  Algorithm: Per-path (round-robin, least-conn, etc) │
│  Distributes: HTTP requests to backend services     │
│  Health check: HTTP /health every 5s                │
│  Features: SSL termination, rate limiting, routing  │
└────────────┬──────────────┬──────────────┬──────────┘
             │              │              │
    ┌────────┼────────┐     │              │
    │        │        │     │              │
    ▼        ▼        ▼     ▼              ▼
┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐
│ API  │ │ WS   │ │Upload│ │ API  │ │  ...     │
│Servers│ │Servers│ │Servers│ │Servers│ │          │
│(3x)  │ │(3x)  │ │(3x)  │ │(3x)  │ │          │
└──────┘ └──────┘ └──────┘ └──────┘ └──────────┘
```

### Tier Descriptions

**Tier 1 -- DNS/Global (Route 53)**
- Distributes users to the nearest region based on latency
- If US-East is down, automatically routes to EU-West
- TTL: 60 seconds for fast failover
- Health checks from 3+ global locations

**Tier 2 -- Regional L4 (HAProxy / NLB)**
- Distributes TCP connections within a region
- Handles 100K+ connections with minimal overhead
- No SSL termination (pass-through)
- No HTTP awareness (pure TCP forwarding)
- Least-connections algorithm for even distribution

**Tier 3 -- Application L7 (Nginx / Envoy)**
- SSL termination (decrypt HTTPS)
- Path-based routing (/api/*, /ws/*, /upload/*)
- Different algorithms per service
- Rate limiting, caching, compression
- Health checks per backend pool

## Part B: Per-Service Algorithm Selection

| Service | Tier 1 (DNS) | Tier 2 (L4) | Tier 3 (L7) | Justification |
|---------|-------------|-------------|-------------|---------------|
| **api.company.com** | Latency-based | Least-conn | Round-robin | Stateless API: route to nearest region, distribute evenly within region. No persistence needed. |
| **ws.company.com** | Latency-based | Least-conn | Least-conn | Long-lived connections: track connections at L7 to prevent overloading any single backend. |
| **upload.company.com** | Latency-based | Least-conn | Least-conn | Variable duration (100ms-5min): least-connections prevents slow uploads from blocking fast ones. |
| **admin.company.com** | Static (single region) | Round-robin | IP hash | Low traffic, IP-restricted. Simple distribution with persistence for admin sessions. |

### Why These Choices

**API (round-robin at L7):**
- Requests are uniform (~50ms)
- Stateless: no session persistence needed
- Round-robin is simplest and most efficient
- Geographic routing at Tier 1 handles latency optimization

**WebSocket (least-conn at L7):**
- Connections are long-lived (2 hours avg)
- Connection count is the primary load metric
- Least-connections prevents any backend from exceeding capacity
- Must also configure WebSocket-specific headers (Upgrade, Connection)

**Upload (least-conn at L7):**
- Request duration varies wildly (100ms to 5min)
- A server handling a 5-min upload should not receive new requests
- Least-connections naturally routes around slow requests
- Also need: `client_max_body_size 1g;` and extended timeouts

**Admin (IP hash at L7):**
- Low traffic volume
- IP-restricted (only office IPs)
- Admin sessions benefit from persistence
- Simple round-robin at L4 is sufficient for the low connection count

## Part C: Failover Design

### 1. Regional L4 Load Balancer Failure

```
Normal state:
  LB-1, LB-2, LB-3 all healthy
  Traffic distributed via Tier 2 (least-connections)

LB-2 fails:
  Tier 2 health check detects failure (TCP check, 5s interval)
  After 3 failures (15s): LB-2 marked unhealthy
  Traffic redistributed: LB-1 and LB-2 share all connections
  LB-2's existing connections: dropped (TCP RST)
  Clients reconnect: new connections go to LB-1 or LB-3

Impact:
  - Existing connections through LB-2 are dropped
  - New connections redistributed to LB-1 and LB-3
  - Recovery time: ~15 seconds (health check detection)
  - No data loss (HTTP requests are retried by clients)
```

### 2. Backend Server Failure During WebSocket Connection

```
WebSocket connection lifecycle during backend failure:

  Client ←→ LB ←→ Backend (fails)

  Step 1: Backend process crashes
  Step 2: LB detects TCP RST or timeout
  Step 3: LB closes connection to client (TCP RST)
  Step 4: Client receives connection close
  Step 5: Client reconnects (automatic with exponential backoff)
  Step 6: New connection routed to healthy backend
  Step 7: Client re-establishes WebSocket state

  Total disruption: 5-30 seconds (depends on client retry logic)

  Mitigation:
  - Client-side reconnection with exponential backoff
  - WebSocket message acknowledgment (re-send unacked messages)
  - Application-level heartbeat (detect connection loss faster)
  - Connection draining on planned restarts (send "reconnect" message)
```

### 3. Entire Region Goes Offline

```
US-East region fails completely:

  Tier 1 (DNS) behavior:
  1. Health checks from 3+ locations detect US-East is down
  2. After 3 consecutive failures (90s): US-East marked unhealthy
  3. DNS responses change: EU-West becomes primary
  4. Propagation: up to 60s (TTL) + 90s (detection) = ~2.5 minutes

  Client behavior:
  - Clients in US: DNS changes to EU-West endpoint
  - Latency increases: US→EU = ~100ms additional
  - Existing connections: dropped (regional failure)
  - New connections: routed to EU-West

  Capacity concern:
  - EU-West must absorb US traffic
  - Ensure EU-West has sufficient capacity (auto-scaling)
  - Monitor latency and error rates during failover
```

### 4. Split-Brain Scenario

```
Split-brain: Network partition between US-East and EU-West
  - US-East thinks it is primary
  - EU-West thinks it is primary
  - Both accept writes

Problem:
  - User A writes to US-East (creates order #1234)
  - User B writes to EU-West (creates order #1235)
  - When partition heals, databases must reconcile

Solutions:
  1. Single-writer: Only one region accepts writes
     - Use distributed lock (etcd, Consul)
     - Other regions are read-only replicas
     - Simple but limits write throughput

  2. Multi-writer with conflict resolution:
     - Use CRDTs (Conflict-free Replicated Data Types)
     - Use last-write-wins with vector clocks
     - Complex but higher availability

  3. Accept eventual consistency:
     - Design application to handle conflicts
     - Use saga pattern for distributed transactions
     - Reconcile during partition healing
```

## Part D: Deployment Strategy

### Zero-Downtime Deployment Procedure

```
Step 1: Prepare new version
════════════════════════════
  # Build and push new container image
  docker build -t api:v2.1 .
  docker push registry/api:v2.1

Step 2: Canary deployment (5% traffic)
══════════════════════════════════════
  # Add new backend to L7 load balancer with low weight
  upstream api_backend {
      server 10.0.1.1:8080 weight=95;  # Old version (95%)
      server 10.0.1.5:8080 weight=5;   # New version (5%)
  }
  nginx -s reload

  # Monitor for 5 minutes
  # Check: error rate, latency, 5xx responses
  # If metrics are bad → Step 6 (rollback)

Step 3: Gradual rollout
════════════════════════
  # Increase new version traffic: 5% → 25% → 50% → 100%
  upstream api_backend {
      server 10.0.1.1:8080 weight=75;
      server 10.0.1.5:8080 weight=25;
  }
  nginx -s reload
  # Monitor 2 minutes

  upstream api_backend {
      server 10.0.1.1:8080 weight=50;
      server 10.0.1.5:8080 weight=50;
  }
  nginx -s reload
  # Monitor 2 minutes

  upstream api_backend {
      server 10.0.1.1:8080;  # Remove weight (or mark down)
      server 10.0.1.5:8080;
  }
  nginx -s reload

Step 4: Drain old version
══════════════════════════
  # Mark old backend as draining
  upstream api_backend {
      server 10.0.1.1:8080 down;  # No new connections
      server 10.0.1.5:8080;
  }
  nginx -s reload

  # Wait for in-flight requests to complete (30s)
  sleep 30

  # Stop old container
  docker stop api-v2.0

Step 5: WebSocket deployment (special handling)
═══════════════════════════════════════════════
  # For WebSocket services, send "reconnect" message before draining

  # 1. Mark backend as "draining" (no new connections)
  # 2. Send WebSocket message to all connected clients:
  #    {"type": "reconnect", "reason": "deployment", "delay": 5000}
  # 3. Wait 5 seconds for clients to process
  # 4. Close remaining connections
  # 5. Stop old container

  # Client code handles reconnect:
  # ws.onmessage = (event) => {
  #   const msg = JSON.parse(event.data);
  #   if (msg.type === 'reconnect') {
  #     ws.close();
  #     setTimeout(() => connect(), msg.delay);
  #   }
  # };

Step 6: Rollback (if needed)
════════════════════════════
  # If new version has errors, immediately rollback:

  upstream api_backend {
      server 10.0.1.1:8080;  # Old version (100%)
      server 10.0.1.5:8080 down;  # New version removed
  }
  nginx -s reload

  # Total rollback time: < 5 seconds (nginx reload)
```

### Deployment Summary

| Phase | Traffic Split | Duration | Rollback? |
|-------|--------------|----------|-----------|
| Canary | 95% old / 5% new | 5 min | Monitor metrics |
| 25% | 75% old / 25% new | 2 min | Monitor metrics |
| 50% | 50% old / 50% new | 2 min | Monitor metrics |
| 100% | 0% old / 100% new | - | - |
| Drain | Old drained | 30s | - |

### Common Mistakes to Avoid

- **Deploying to all servers simultaneously.** If the new version has a
  bug, all servers fail at once. Always use canary or rolling deployment.
- **Not monitoring during canary.** The canary phase is useless if you do
  not actually compare error rates and latency between old and new versions.
- **Forgetting about WebSocket connections during deployment.** HTTP
  connections are short and retry-friendly. WebSocket connections are
  long-lived and stateful. Send a reconnect message before closing.
- **DNS failover that is too slow.** If DNS TTL is 1 hour, regional
  failover takes 1 hour. Use 60-second TTL for critical services.
- **No capacity planning for failover.** If US-East fails and EU-West
  must absorb all traffic, EU-West needs 2x capacity. Plan for this.

## Key Takeaway

Multi-tier load balancing distributes traffic at different granularities:
DNS for global routing, L4 for regional TCP distribution, L7 for
application-aware HTTP routing. Each tier has different failover characteristics
(seconds at L4, minutes at DNS). The combination must handle backend failures,
regional outages, and zero-downtime deployments. The key insight is that each
tier optimizes for different properties: DNS optimizes for latency, L4 for
throughput, and L7 for intelligence.
