# Solution 01: Classify the Attack

## Part A: Attack Classification

| Time | Attack Type | OSI Layer | Attack Name | Goal |
|------|------------|-----------|-------------|------|
| 09:15 | Volumetric | L3/L4 | UDP Flood | Saturate inbound bandwidth (45 Gbps vs 500 Mbps normal) |
| 09:32 | Protocol | L4 | SYN Flood | Exhaust server connection table with half-open TCP connections |
| 09:47 | Application | L7 | HTTP Flood (Resource Exhaustion) | Exhaust database resources via expensive search queries |
| 10:05 | Application | L7 | Credential Stuffing / Brute Force | Gain unauthorized access or exhaust auth service |
| 10:20 | Application | L7 | Slowloris | Exhaust server thread pool with slow connections |

**Detailed analysis:**

- **09:15 (UDP Flood):** 45 Gbps from 12,000 IPs on random ports is a classic volumetric attack. The goal is to saturate the network link so legitimate traffic cannot reach the servers. The use of random ports makes simple port-based filtering ineffective. This is a Layer 3/4 attack because it targets the network infrastructure, not the application.

- **09:32 (SYN Flood):** 800,000 SYN packets/sec with no completing ACKs is a SYN flood. The attacker sends TCP SYN packets to initiate connections but never completes the three-way handshake. Each half-open connection consumes a slot in the server's connection table. When the table fills, the server cannot accept new connections from anyone. This is a Layer 4 attack because it exploits the TCP protocol.

- **09:47 (HTTP Flood):** 50,000 GET requests to `/api/search` with valid User-Agents is an application-layer attack. Each request triggers a full-text database search, which is CPU and I/O intensive. The attacker is not trying to saturate bandwidth -- they are trying to exhaust application resources (database connections, CPU). This is the hardest attack to mitigate because the traffic looks legitimate. The residential IPs suggest a botnet.

- **10:05 (Credential Stuffing):** 5,000 POST requests to `/api/login` with different username/password pairs from only 3 IPs is credential stuffing. The attacker is testing stolen credentials. This is an application-layer attack that targets the authentication service. The small number of source IPs makes IP-based blocking effective.

- **10:20 (Slowloris):** 10,000 connections sending headers at 1 byte/sec is Slowloris. The attacker opens many connections and keeps them alive by sending data as slowly as possible. Each connection consumes a server thread. When all threads are consumed, the server cannot handle new requests. This is a Layer 7 attack because it targets the application's connection handling.

---

## Part B: Mitigation Layer Mapping

| Time | Attack | Primary Layer | Secondary Layer | Rationale |
|------|--------|--------------|-----------------|-----------|
| 09:15 | UDP Flood | Layer 1: Cloud DDoS Shield | Layer 2: CDN | 45 Gbps must be scrubbed upstream. Your 500 Mbps link is already saturated. No on-premise device can handle this volume. |
| 09:32 | SYN Flood | Layer 3: Load Balancer | Layer 1: Cloud DDoS Shield | SYN cookies at the load balancer prevent connection table exhaustion. Upstream scrubbing can also filter SYN floods before they reach you. |
| 09:47 | HTTP Flood | Layer 4: Rate Limiting | Layer 5: Application Code | Rate limiting caps requests per IP. Application-level caching of search results and query optimization reduce the cost per request. |
| 10:05 | Credential Stuffing | Layer 4: Rate Limiting | Layer 5: Application Code | Strict rate limiting on `/api/auth/login` (e.g., 1 req/s per IP). Application code should implement account lockout and CAPTCHA. |
| 10:20 | Slowloris | Layer 3: Load Balancer | Layer 4: Rate Limiting | Connection timeouts and connection limits at the load balancer close slow connections. Rate limiting caps the number of new connections per IP. |

**Why this mapping matters:**

The most common mistake in DDoS defense is trying to stop volumetric attacks at the application layer. If 45 Gbps of UDP traffic is hitting your server, no amount of Nginx rate limiting will help -- the bandwidth is already saturated before traffic reaches Nginx. Volumetric attacks must be stopped at Layer 1 (upstream scrubbing by your cloud provider or CDN). Conversely, using upstream scrubbing for application-layer attacks is expensive and imprecise -- the upstream provider cannot distinguish between a legitimate search request and an attack search request. Each layer handles the attack type it is designed for.

---

## Part C: Blast Radius Analysis

### 09:15 -- UDP Flood (if Layer 1 mitigation fails)

- **Affected services:** ALL services. Bandwidth saturation means no traffic reaches any server.
- **Impact on legitimate users:** Complete outage. Users cannot reach the website, API, or any service.
- **Escalation speed:** Immediate. As soon as the link is saturated, all services are down.
- **Cascading effects:** Health checks fail, load balancers remove all targets, auto-scaling triggers (wasting money), DNS may cache the failure.

### 09:32 -- SYN Flood (if Layer 3 mitigation fails)

- **Affected services:** ALL TCP-based services. The connection table is shared across all ports.
- **Impact on legitimate users:** New connections fail. Existing connections continue until they timeout.
- **Escalation speed:** Seconds. The connection table fills within seconds at 800,000 SYN/sec.
- **Cascading effects:** Load balancer cannot establish connections to backends, health checks fail, auto-scaling adds more servers (which also fill their connection tables).

### 09:47 -- HTTP Flood (if Layer 4 mitigation fails)

- **Affected services:** Initially only `/api/search`. But if the database connection pool is exhausted, ALL database-dependent endpoints fail.
- **Impact on legitimate users:** Search is slow or unavailable. If cascading, all data-dependent pages fail.
- **Escalation speed:** Minutes. The database connection pool fills gradually as queries queue up.
- **Cascading effects:** Database CPU maxes out, query response times increase, connection pool exhaustion spreads to other services, application timeouts trigger, error rates spike across all endpoints.

### 10:05 -- Credential Stuffing (if Layer 4 mitigation fails)

- **Affected services:** Authentication service, login page.
- **Impact on legitimate users:** Login is slow or unavailable. Account lockouts may affect real users.
- **Escalation speed:** Minutes. Depends on the authentication service's capacity.
- **Cascading effects:** If the auth service shares a database with other services, database load increases. Account lockouts create support tickets. If credentials are valid, unauthorized access occurs.

### 10:20 -- Slowloris (if Layer 3 mitigation fails)

- **Affected services:** All services behind the same web server. Thread pool exhaustion prevents handling any new request.
- **Impact on legitimate users:** All requests timeout. The server appears completely unresponsive.
- **Escalation speed:** Seconds. 10,000 slow connections exhaust a typical thread pool (default 200-500 threads) almost immediately.
- **Cascading effects:** Load balancer sees the server as unhealthy, removes it from the pool, distributes load to remaining servers (which may also be under Slowloris attack), cascading failure.

---

## Part D: Alert Thresholds

| Attack Type | Metric | Baseline | Alert Threshold | Severity |
|-------------|--------|----------|-----------------|----------|
| Volumetric (UDP Flood) | Inbound bandwidth (Mbps) | 500 Mbps | 5,000 Mbps (10x) | Critical |
| Volumetric (UDP Flood) | Inbound packet rate (pps) | 50,000 pps | 500,000 pps (10x) | Critical |
| SYN Flood | SYN packets/sec | 1,000/s | 10,000/s (10x) | Critical |
| SYN Flood | Connection table utilization | 20% | 70% | Warning |
| SYN Flood | Half-open connections | 100 | 5,000 | Critical |
| HTTP Flood | RPS per endpoint | 500/s | 5,000/s (10x) | Warning |
| HTTP Flood | RPS per endpoint | 500/s | 15,000/s (30x) | Critical |
| HTTP Flood | 5xx error rate | 0.5% | 10% | Warning |
| HTTP Flood | 5xx error rate | 0.5% | 30% | Critical |
| HTTP Flood | Database connection utilization | 30% | 80% | Warning |
| Credential Stuffing | Login attempts/sec per IP | 0.1/s | 1/s (10x) | Warning |
| Credential Stuffing | Failed login rate | 5% | 50% | Critical |
| Slowloris | Average connection duration | 2s | 30s | Warning |
| Slowloris | Active connections per IP | 2 | 20 | Warning |
| Slowloris | Server thread pool utilization | 30% | 80% | Critical |

**Detection strategy:**

- Use a 60-second sliding window for all metrics.
- Alert on the first threshold breach (do not wait for sustained breach).
- Use anomaly detection (standard deviation from rolling baseline) as a secondary signal.
- Correlated alerts (e.g., high RPS + high error rate + high response time) should escalate severity.
- Implement a 5-minute cooldown between repeated alerts of the same type to prevent alert fatigue.

---

## Common Mistakes

- **Trying to stop volumetric attacks at the application layer.** If 45 Gbps is hitting your server, the network link is saturated. Nginx rate limiting cannot help because the traffic never reaches Nginx. Volumetric attacks must be stopped upstream by your cloud provider or CDN.
- **Treating all HTTP floods the same.** A flood of GET requests to `/health` is very different from a flood of GET requests to `/api/search`. The former is cheap to serve; the latter triggers expensive database queries. Rate limits must be endpoint-specific.
- **Ignoring cascading failures.** A SYN flood that fills the connection table affects ALL services, not just the targeted port. An HTTP flood that exhausts the database connection pool affects all endpoints that use the database. Always consider the blast radius beyond the immediate target.
- **Alerting on absolute thresholds only.** A threshold of 10,000 RPS might be normal during peak hours but an attack at 3 AM. Use both absolute thresholds and deviation-from-baseline detection.
