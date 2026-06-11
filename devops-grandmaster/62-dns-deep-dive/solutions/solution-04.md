# Solution 04: DNS Failover Setup

## Part A: Design the Failover Configuration

```yaml
# DNS Failover Configuration
# Provider: Route 53 / Cloudflare / similar

records:
  - name: shop.example-corp.com
    type: A
    ttl: 60  # Low TTL for fast failover

    # Primary record
    primary:
      value: 203.0.113.10
      health_check:
        endpoint: https://shop.example-corp.com/health
        protocol: HTTPS
        port: 443
        interval: 10          # Check every 10 seconds
        failure_threshold: 3  # 3 consecutive failures = unhealthy
        regions:              # Check from multiple locations
          - us-east-1
          - eu-west-1
          - ap-southeast-1
        latency_measurement: true
      routing_policy:
        type: failover
        role: primary

    # Secondary record (active only when primary is unhealthy)
    secondary:
      value: 198.51.100.10
      health_check:
        endpoint: https://shop.example-corp.com/health
        protocol: HTTPS
        port: 443
        interval: 10
        failure_threshold: 3
        regions:
          - us-east-1
          - eu-west-1
      routing_policy:
        type: failover
        role: secondary
```

### Why This Configuration Works

- **TTL of 60 seconds:** Ensures resolvers re-query within 1 minute of a
  failover event. Combined with health check intervals, worst-case failover
  time is approximately 60 + 30 = 90 seconds.
- **Health checks every 10 seconds:** Detects failures quickly without
  excessive load on the server.
- **3 consecutive failures:** Prevents false failovers caused by transient
  network glitches or single-request timeouts.
- **Multi-region health checks:** A single-region check might false-positive
  if the monitoring region has connectivity issues. Requiring agreement
  from multiple regions reduces false positives.

## Part B: Multi-Region Failover

```yaml
# Multi-Region Failover Configuration

records:
  - name: api.example-corp.com
    type: A
    ttl: 60

    # Geographic routing with failover priority
    routing:
      # US-East: Primary (lowest priority number = highest priority)
      - region: us-east-1
        value: 203.0.113.10
        priority: 1
        health_check:
          endpoint: https://203.0.113.10/health
          interval: 10
          failure_threshold: 3

      # EU-West: Secondary
      - region: eu-west-1
        value: 198.51.100.10
        priority: 2
        health_check:
          endpoint: https://198.51.100.10/health
          interval: 10
          failure_threshold: 3

      # AP-South: Tertiary
      - region: ap-south-1
        value: 192.0.2.10
        priority: 3
        health_check:
          endpoint: https://192.0.2.10/health
          interval: 10
          failure_threshold: 3
```

### Failover Behavior

```
Normal state:
  US-East healthy   → All traffic → 203.0.113.10 (US-East)
  EU-West healthy   → (standby, no traffic)
  AP-South healthy  → (standby, no traffic)

US-East fails:
  US-East unhealthy → Traffic shifts → 198.51.100.10 (EU-West)
  AP-South healthy  → (standby)

US-East + EU-West both fail:
  US-East unhealthy → Traffic shifts → 192.0.2.10 (AP-South)
  EU-West unhealthy → (skipped)

US-East recovers:
  US-East healthy   → Traffic returns → 203.0.113.10 (US-East)
  EU-West healthy   → (standby again)
```

### Alternative: Active-Active with Failover

For better resource utilization, you can use weighted or latency-based
routing with failover:

```yaml
# Active-Active with Failover
routing:
  - region: us-east-1
    value: 203.0.113.10
    weight: 100            # Proportional traffic share
    health_check: { ... }
    failover_to: eu-west-1  # If unhealthy, send to EU-West

  - region: eu-west-1
    value: 198.51.100.10
    weight: 50
    health_check: { ... }
    failover_to: ap-south-1

  - region: ap-south-1
    value: 192.0.2.10
    weight: 50
    health_check: { ... }
    failover_to: us-east-1
```

This keeps all regions active under normal conditions, improving latency
for global users, while still providing failover capability.

## Part C: Failover Testing Plan

### Test Case 1: Normal Operation

```bash
# Verify primary is serving traffic
dig +short shop.example-corp.com
# Expected: 203.0.113.10

# Verify health check is passing
curl -s https://shop.example-corp.com/health
# Expected: HTTP 200

# Verify from multiple locations
dig @8.8.8.8 +short shop.example-corp.com     # Google
dig @1.1.1.1 +short shop.example-corp.com     # Cloudflare
# Expected: All return 203.0.113.10
```

### Test Case 2: Failover Trigger

```bash
# Simulate primary failure
# Option A: Block traffic on primary server
iptables -A INPUT -p tcp --dport 443 -j DROP

# Option B: Stop the application on primary
systemctl stop nginx

# Monitor failover timing
while true; do
  result=$(dig +short shop.example-corp.com)
  echo "$(date): $result"
  sleep 5
done

# Expected: After ~90 seconds (health check interval * failures + TTL),
# resolution changes from 203.0.113.10 to 198.51.100.10
```

### Test Case 3: Failback

```bash
# Restore primary server
iptables -D INPUT -p tcp --dport 443 -j DROP

# Monitor failback timing
while true; do
  result=$(dig +short shop.example-corp.com)
  echo "$(date): $result"
  sleep 5
done

# Expected: After health check passes, traffic returns to 203.0.113.10
# Typical failback time: 30-60 seconds
```

### Test Case 4: Cascading Failure

```bash
# Take down primary AND secondary
# Primary: iptables -A INPUT -p tcp --dport 443 -j DROP
# Secondary: systemctl stop nginx

# Verify behavior
dig +short shop.example-corp.com
# Expected: Either the tertiary IP (if configured) or SERVFAIL/NXDOMAIN
# depending on provider behavior

# Document: What happens when ALL failover targets are unhealthy?
# Some providers return the primary IP (assuming monitoring error)
# Others return SERVFAIL
```

### Test Metrics to Record

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Failover detection time | < 60 seconds | Time from failure to health check failure |
| DNS propagation time | < 60 seconds | Time from health check failure to resolution change |
| Total failover time | < 120 seconds | Time from failure to client receiving new IP |
| Failback time | < 120 seconds | Time from recovery to traffic returning to primary |
| False positive rate | < 1 per month | Monitor failover events not caused by real failures |

## Part D: Failover Limitations

### Limitation 1: TTL-Based Propagation Delay

**Problem:** Even with a 60-second TTL, some resolvers cache records
longer than the TTL specifies. ISP resolvers, corporate DNS servers,
and browser DNS caches may ignore TTLs. The actual propagation can
take 5-15 minutes in worst cases.

**Mitigation:**
- Use the lowest practical TTL (30-60 seconds)
- Implement application-level retry logic that re-resolves DNS on failure
- Use `Cache-Control: no-cache` headers for critical endpoints
- Consider HTTP-level failover (load balancer) as a complement to DNS

### Limitation 2: No Session Persistence

**Problem:** DNS failover moves ALL clients to the new server. Clients
with active sessions (login tokens, shopping carts, WebSocket connections)
will lose their session state. The new server does not have the old
server's in-memory state.

**Mitigation:**
- Store session state in a shared database or Redis cluster (not in-memory)
- Use sticky sessions at the load balancer level (not DNS level)
- Design applications to be stateless (JWT tokens, not server sessions)
- Implement graceful connection draining during planned failovers

### Limitation 3: Split-Brain Problem

**Problem:** During failover, some resolvers may still point to the old
(primary) server while others point to the new (secondary) server. If
the primary server is partially available (e.g., network partition), both
servers may accept writes, causing data inconsistency.

**Mitigation:**
- Use a single writable database with read replicas
- Implement leader election so only one server accepts writes
- Use distributed consensus (Raft, Paxos) for critical state
- Monitor for split-brain conditions and alert immediately

### Common Mistakes to Avoid

- **Setting TTL too low globally.** A 10-second TTL on a high-traffic
  domain floods authoritative servers. Use low TTLs only for failover
  records, not for all records.
- **Health checks that are too aggressive.** Checking every second with a
  1-failure threshold causes failovers on brief network blips. Use
  10-second intervals with 3-failure thresholds for production.
- **Not testing failover.** An untested failover configuration is not a
  failover strategy. Run failover drills quarterly.
- **Forgetting about DNS client caching.** Browsers (Chrome, Firefox) have
  their own DNS caches that may ignore TTLs. Test with real browsers, not
  just `dig`.

## Key Takeaway

DNS failover provides high availability by routing traffic away from failed
servers. The key trade-off is TTL: lower TTL enables faster failover but
increases DNS query load. Health checks must balance detection speed with
false positive avoidance. DNS failover has inherent limitations (caching
delays, no session persistence, split-brain risk) that require complementary
solutions at the application and load balancer layers.
