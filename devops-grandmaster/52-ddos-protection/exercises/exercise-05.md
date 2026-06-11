# Exercise 05: Multi-Layer DDoS Protection

**Type:** Integration
**Time:** 45 minutes
**Difficulty:** Hard

## Objective

Design and implement a complete multi-layer DDoS protection system that combines rate limiting, traffic analysis, circuit breakers, auto-scaling, and cost controls into a unified defense architecture. This exercise integrates concepts from all previous exercises into a production-ready system.

## Scenario

You are the platform engineer at StreamFlow, a video streaming platform with 500,000 active users. The architecture:

```
Internet
  |
CloudFront CDN (with Shield Advanced)
  |
Application Load Balancer
  |
+---+---+---+
|   |   |   |
API Auth Search Stream
|   |   |   |
+---+---+---+
  |
PostgreSQL + Redis + S3
```

Recent incidents have exposed gaps in the DDoS protection:
1. An HTTP flood overwhelmed the API service before rate limiting kicked in
2. A SYN flood exhausted the ALB connection table
3. Auto-scaling responded to attack traffic, causing a $15,000 cost spike
4. The search service was hammered, which cascaded to the database

You need to design a unified system that addresses all these gaps.

## Tasks

### Part A: Architecture Design Document

Create a file called `ddos-architecture.md` that describes the complete multi-layer defense architecture. Include:

1. **Layer diagram** showing all defense layers from the network edge to the application
2. **Data flow** showing how a request passes through each layer
3. **Decision matrix** mapping each attack type to the layer that handles it
4. **Failure modes** describing what happens when each layer fails
5. **Cost model** estimating the cost of each defense layer

The architecture must address all four incident scenarios from the scenario description.

<details>
<summary>Hint: Layer structure</summary>

```
Layer 1: Network Edge (CloudFront + Shield Advanced)
  - Absorbs volumetric attacks (UDP flood, ICMP flood)
  - Geo-blocking for regions you do not serve
  - Automatic scaling to absorb traffic spikes

Layer 2: CDN / WAF (CloudFront + AWS WAF)
  - Rate-based rules (2000 req/5min per IP)
  - Managed rule groups (bot detection, known bad inputs)
  - Challenge pages for suspicious traffic

Layer 3: Load Balancer (ALB)
  - Connection limits and idle timeout
  - Slow request protection
  - SYN cookie enforcement

Layer 4: Rate Limiting (Nginx / API Gateway)
  - Per-endpoint rate limits
  - Per-IP request quotas
  - Token bucket with burst handling

Layer 5: Application (Circuit Breakers + Graceful Degradation)
  - Database connection pool protection
  - Cache-first fallback
  - Feature flags for non-critical features
```

</details>

### Part B: Nginx Rate Limiting Configuration

Write a complete Nginx configuration (`nginx.conf`) that implements per-endpoint rate limiting with these requirements:

| Endpoint | Rate | Burst | Notes |
|----------|------|-------|-------|
| `/api/*` (general) | 30 r/s | 50 | Normal API traffic |
| `/api/auth/*` | 5 r/s | 10 | Auth endpoints |
| `/api/search` | 10 r/s | 20 | Search with DB queries |
| `/api/stream/*` | 100 r/s | 200 | High-throughput streaming |
| `/health` | Unlimited | - | Health checks |

Additionally:
- Implement IP-based whitelisting for internal monitoring systems
- Add geo-blocking for countries where the service is not available
- Configure custom JSON error responses for all limit-exceeded scenarios
- Add request size limits and header count limits
- Include proxy timeouts that protect against slow backends

<details>
<summary>Hint: GeoIP and whitelisting</summary>

Use the `geo` module to create an allow list for internal IPs:
```nginx
geo $is_internal {
    default 0;
    10.0.0.0/8 1;
    172.16.0.0/12 1;
    192.168.0.0/16 1;
}

map $is_internal $rate_limit_key {
    0 $binary_remote_addr;
    1 "";  # Empty key = no rate limit
}
```

For geo-blocking, use the `geoip2` module with MaxMind's GeoLite2 database, or use `map` with IP ranges if you do not have GeoIP.

</details>

### Part C: Traffic Analyzer with Auto-Scaling Awareness

Write a Python module called `smart_scaler.py` that integrates traffic analysis with auto-scaling decisions. The module should:

1. **Monitor traffic metrics** -- RPS, error rate, response time, connection count
2. **Distinguish attack traffic from legitimate growth** -- Use multiple signals:
   - If RPS increases but error rate stays low and response times are normal: legitimate growth, scale up
   - If RPS increases AND error rate spikes AND response times degrade: attack traffic, do NOT scale up
   - If RPS increases from a small number of IPs: attack, block IPs instead of scaling
3. **Implement cost controls** -- Set a maximum scale-up ceiling and a cost-per-hour budget
4. **Make scaling decisions** -- Output recommendations for ECS desired count

The module should expose a `make_decision(current_metrics, baseline, budget)` method that returns:

```python
{
    "action": "scale_up" | "scale_down" | "block_ips" | "enable_rate_limit" | "do_nothing",
    "target_count": 6,           # ECS task count (if scaling)
    "ips_to_block": ["1.2.3.4"], # IPs to block (if blocking)
    "rate_limit_rps": 5,         # New rate limit (if adjusting)
    "reason": "Traffic spike detected from 3 IPs, blocking instead of scaling",
    "estimated_cost_impact": "$0.00"  # Cost impact of the decision
}
```

<details>
<summary>Hint: Attack vs growth detection</summary>

The key signal is traffic diversity. Legitimate growth comes from many IPs with varied User-Agents and request patterns. Attack traffic often comes from few IPs (botnet with limited diversity) or many IPs with identical patterns (distributed botnet).

```python
def is_likely_attack(metrics):
    # Signal 1: Traffic concentration
    top_10_ip_share = sum(count for _, count in metrics['top_ips'][:10]) / metrics['total_requests']
    if top_10_ip_share > 0.5:  # Top 10 IPs account for >50% of traffic
        return True, "IP concentration"

    # Signal 2: Error rate correlation
    if metrics['error_rate'] > 0.2 and metrics['rps'] > metrics['baseline_rps'] * 3:
        return True, "High error rate with traffic spike"

    # Signal 3: User-Agent diversity
    if metrics['unique_user_agents'] / metrics['total_requests'] < 0.01:
        return True, "Low User-Agent diversity"

    return False, "Traffic appears legitimate"
```

</details>

### Part D: Circuit Breaker with Cache Fallback

Write a Python module called `resilient_service.py` that combines a circuit breaker with a cache-first strategy:

1. **Normal operation** (circuit CLOSED): Query database, cache the result in Redis
2. **Under attack** (circuit OPEN): Serve from Redis cache, mark response as stale
3. **Recovery** (circuit HALF_OPEN): Allow 1 request per 10 seconds to test the database
4. **Cache warming**: Pre-populate cache for critical endpoints on startup
5. **Cache stampede protection**: Use a distributed lock to prevent multiple processes from rebuilding the same cache entry simultaneously

Implement these endpoints:
- `GET /api/products` -- List products (cacheable, 60s TTL)
- `GET /api/products/{id}` -- Product detail (cacheable, 30s TTL)
- `GET /api/search?q=...` -- Search results (cacheable, 10s TTL)
- `GET /api/recommendations` -- Personalized recommendations (not cacheable, fallback to generic)

<details>
<summary>Hint: Cache stampede protection</summary>

Use Redis SET with NX (set if not exists) and EX (expiry) as a simple distributed lock:

```python
import redis
import time

def get_with_stampede_protection(redis_client, key, compute_fn, ttl=60):
    # Try cache first
    value = redis_client.get(key)
    if value:
        return json.loads(value)

    # Acquire lock
    lock_key = f"lock:{key}"
    acquired = redis_client.set(lock_key, "1", nx=True, ex=10)

    if acquired:
        try:
            # This process computes and caches
            value = compute_fn()
            redis_client.setex(key, ttl, json.dumps(value))
            return value
        finally:
            redis_client.delete(lock_key)
    else:
        # Another process is computing, wait and retry
        for _ in range(10):
            time.sleep(0.5)
            value = redis_client.get(key)
            if value:
                return json.loads(value)
        # Fallback after timeout
        return compute_fn()
```

</details>

### Part E: Integration Test Suite

Write a test script called `test_ddos_protection.py` that validates the entire multi-layer system:

1. **Test rate limiting** -- Verify that requests above the limit are rejected with 429
2. **Test circuit breaker** -- Simulate database failures and verify cached responses are served
3. **Test auto-scaling decisions** -- Feed attack traffic data and verify the scaler does NOT scale up
4. **Test IP blocking** -- Verify that identified attacker IPs are blocked
5. **Test graceful degradation** -- Verify that the system serves partial results when under load
6. **Test cost controls** -- Verify that scaling respects the budget ceiling
7. **Test recovery** -- Verify that the system returns to normal after the attack stops

Each test should:
- Set up the necessary mocks/fixtures
- Execute the test scenario
- Assert the expected behavior
- Clean up after itself

<details>
<summary>Hint: Mocking Redis for tests</summary>

Use `fakeredis` for unit tests instead of requiring a real Redis instance:
```python
import fakeredis

def test_circuit_breaker_serves_from_cache():
    redis_client = fakeredis.FakeRedis()

    # Pre-populate cache
    redis_client.setex("products:list", 60, json.dumps({"products": [...]}))

    # Simulate database failure
    circuit = CircuitBreaker(failure_threshold=1)
    circuit._on_failure()  # Force circuit open

    # Verify cached response is served
    response = get_products(circuit, redis_client)
    assert response["source"] == "cache"
    assert "products" in response
```

</details>

---

## Success Criteria

- [ ] The architecture document describes 5 defense layers with data flow and decision matrix.
- [ ] The Nginx configuration implements per-endpoint rate limiting with geo-blocking and whitelisting.
- [ ] The smart scaler correctly distinguishes attack traffic from legitimate growth in at least 3 scenarios.
- [ ] The circuit breaker serves from cache when open and recovers correctly when the database returns.
- [ ] The integration test suite covers at least 6 scenarios and all tests pass.
- [ ] The cost control mechanism prevents auto-scaling beyond the defined budget ceiling.
- [ ] The system demonstrates graceful degradation: partial service availability during an attack.

## What You Should Understand After This Exercise

Multi-layer DDoS protection is a system, not a single tool. Each layer has a specific responsibility and handles the attack types it is best suited for. The network edge absorbs volumetric attacks. The WAF filters malicious application traffic. Rate limiting caps per-client abuse. Circuit breakers protect downstream services. Auto-scaling handles legitimate traffic growth -- but only if you can distinguish it from attack traffic. Cost controls prevent attacks from becoming financial disasters. The key design principle is defense in depth: no single layer is sufficient, but together they create a resilient system that maintains service availability under attack while controlling costs.
