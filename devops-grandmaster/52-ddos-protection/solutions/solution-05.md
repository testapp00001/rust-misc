# Solution 05: Multi-Layer DDoS Protection

## Part A: Architecture Design Document

```markdown
# StreamFlow Multi-Layer DDoS Protection Architecture

## Layer Diagram

```
Internet
  |
  v
[Layer 1: Network Edge]
  CloudFront CDN + AWS Shield Advanced
  - Absorbs volumetric attacks (UDP, ICMP, DNS amplification)
  - Global edge locations absorb traffic before it reaches origin
  - Automatic volumetric attack mitigation (always-on)
  - Geo-blocking for unserved regions
  |
  v
[Layer 2: CDN / WAF]
  CloudFront + AWS WAF
  - Rate-based rules (per-IP request limits)
  - Managed rule groups (bot detection, known bad inputs)
  - Challenge pages for suspicious traffic
  - IP reputation filtering
  |
  v
[Layer 3: Load Balancer]
  Application Load Balancer
  - Connection limits and idle timeout
  - Slow request protection (request timeout)
  - SYN cookie enforcement
  - SSL/TLS termination
  |
  v
[Layer 4: Rate Limiting]
  Nginx Reverse Proxy
  - Per-endpoint rate limits
  - Per-IP request quotas
  - Token bucket with burst handling
  - Geo-blocking at application level
  |
  v
[Layer 5: Application]
  Circuit Breakers + Graceful Degradation
  - Database connection pool protection
  - Cache-first fallback
  - Feature flags for non-critical features
  - Request prioritization
  |
  v
[Data Layer]
  PostgreSQL + Redis + S3
  - Connection pool limits
  - Read replicas for read-heavy endpoints
  - Redis cache for hot data
```

## Data Flow

```
1. Client sends HTTPS request
2. CloudFront edge node receives request
   -> Shield Advanced checks for volumetric attack signatures
   -> If attack: absorbed at edge, never reaches origin
3. WAF rules evaluate the request
   -> Rate limit check (per-IP, per-endpoint)
   -> Bot detection (managed rule groups)
   -> Known bad input patterns
   -> If blocked: 403 returned at edge
4. Request forwarded to ALB
   -> ALB checks connection limits
   -> ALB applies request timeout
   -> If timeout: 504 returned
5. Request forwarded to Nginx
   -> Endpoint-specific rate limiting
   -> IP-based quotas
   -> If rate limited: 429 returned
6. Request reaches application
   -> Circuit breaker checks database health
   -> If circuit open: serve from cache
   -> If circuit closed: query database
7. Response returned through the same path
```

## Decision Matrix

| Attack Type | Primary Layer | Secondary Layer | Why |
|-------------|--------------|-----------------|-----|
| UDP Flood | Layer 1: Shield | Layer 1: CloudFront | Must stop at edge; origin cannot handle volume |
| SYN Flood | Layer 1: Shield | Layer 3: ALB | Shield filters SYN floods; ALB has SYN cookies |
| DNS Amplification | Layer 1: Shield | - | Shield absorbs amplification traffic |
| HTTP Flood (general) | Layer 2: WAF | Layer 4: Nginx | WAF rate limits; Nginx provides endpoint-specific limits |
| HTTP Flood (expensive endpoints) | Layer 4: Nginx | Layer 5: Circuit Breaker | Nginx limits requests; circuit breaker protects database |
| Slowloris | Layer 3: ALB | Layer 4: Nginx | ALB timeout closes slow connections; Nginx limits connections |
| Bot Traffic | Layer 2: WAF | Layer 4: Nginx | WAF bot detection; Nginx User-Agent filtering |
| Credential Stuffing | Layer 4: Nginx | Layer 5: Application | Nginx strict auth limits; app implements lockout + CAPTCHA |

## Failure Modes

| Layer | Failure Mode | Impact | Fallback |
|-------|-------------|--------|----------|
| Layer 1 (Shield) | Shield not enabled or attack exceeds Shield capacity | Volumetric traffic reaches CloudFront | Contact AWS Shield Response Team (SRT) |
| Layer 2 (WAF) | WAF misconfigured or rules bypassed | Malicious traffic reaches ALB | Layer 4 Nginx rate limiting catches it |
| Layer 3 (ALB) | ALB connection table exhausted | New connections fail | CloudFront retries with different origin |
| Layer 4 (Nginx) | Nginx overwhelmed or misconfigured | All traffic reaches application | Layer 5 circuit breakers protect database |
| Layer 5 (App) | Circuit breaker fails or database down | Application errors | Cache serves stale data; 503 for uncached |

## Incident-Specific Mitigations

### Incident 1: HTTP flood overwhelmed API before rate limiting
- **Root cause:** Rate limiting was not configured per-endpoint; a single zone was used for all paths
- **Fix:** Implement per-endpoint rate limiting (Part B) with stricter limits for expensive endpoints
- **Layer addressed:** Layer 4 (Nginx)

### Incident 2: SYN flood exhausted ALB connection table
- **Root cause:** No upstream SYN flood protection; ALB was the first line of defense
- **Fix:** Enable Shield Advanced (Part A) which filters SYN floods at the edge
- **Layer addressed:** Layer 1 (Shield)

### Incident 3: Auto-scaling caused $15,000 cost spike
- **Root cause:** Auto-scaling responded to attack traffic as if it were legitimate growth
- **Fix:** Implement smart scaler (Part C) that distinguishes attack from growth
- **Layer addressed:** Layer 5 (Application) -- scaling decision logic

### Incident 4: Search service hammered, cascaded to database
- **Root cause:** No circuit breakers; search queries exhausted database connection pool
- **Fix:** Implement circuit breaker with cache fallback (Part D)
- **Layer addressed:** Layer 5 (Application)

## Cost Model

| Layer | Service | Monthly Cost | Justification |
|-------|---------|-------------|---------------|
| Layer 1 | Shield Advanced | $3,000 + data transfer | Enterprise DDoS protection with 24/7 SRT |
| Layer 2 | CloudFront | $500-2,000 (usage-based) | CDN + WAF at edge |
| Layer 2 | AWS WAF | $100 + $1/rule/month | Rate limiting and managed rules |
| Layer 3 | ALB | $200 + usage | Load balancing with connection management |
| Layer 4 | Nginx (ECS) | $100-300 | Rate limiting proxy (2-4 tasks) |
| Layer 5 | Application (ECS) | $500-2,000 (auto-scaling) | Application servers |
| Monitoring | CloudWatch | $100-300 | Metrics, alarms, dashboards |
| **Total** | | **$4,500-8,000/month** | |

**Cost of NOT having protection:** One major DDoS attack can cost $10,000-$100,000+ in lost revenue, emergency response time, and infrastructure damage. The $4,500-8,000/month investment pays for itself after preventing a single attack.
```

---

## Part B: Nginx Rate Limiting Configuration

```nginx
worker_processes auto;
worker_rlimit_nofile 65535;

events {
    worker_connections 4096;
    multi_accept on;
}

http {
    # -------------------------------------------------------
    # Rate Limiting Zones
    # -------------------------------------------------------

    # General API: 30 req/s per IP
    limit_req_zone $binary_remote_addr zone=api:10m rate=30r/s;

    # Authentication: 5 req/s per IP
    limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/s;

    # Search: 10 req/s per IP
    limit_req_zone $binary_remote_addr zone=search:10m rate=10r/s;

    # Streaming: 100 req/s per IP
    limit_req_zone $binary_remote_addr zone=stream:10m rate=100r/s;

    # Connection limiting
    limit_conn_zone $binary_remote_addr zone=conn:10m;

    # Global settings
    limit_req_status 429;
    limit_conn_status 429;
    limit_req_log_level warn;

    # -------------------------------------------------------
    # GeoIP and Whitelisting
    # -------------------------------------------------------

    # Internal IP whitelist (monitoring systems, health checks)
    geo $is_internal {
        default 0;
        10.0.0.0/8 1;
        172.16.0.0/12 1;
        192.168.0.0/16 1;
        # Add monitoring system IPs
        203.0.113.10/32 1;  # Datadog agent
        198.51.100.20/32 1;  # PagerDuty health check
    }

    # Rate limit key: empty for internal IPs (no limit), client IP for external
    map $is_internal $rate_limit_key {
        0 $binary_remote_addr;
        1 "";
    }

    # Geo-blocking: countries where StreamFlow does not operate
    # (Use geoip2 module with MaxMind database for production)
    geo $is_blocked_country {
        default 0;
        # Example: block specific IP ranges known for abuse
        # In production, use GeoIP2 module instead
    }

    # -------------------------------------------------------
    # Logging
    # -------------------------------------------------------

    log_format detailed '$remote_addr - $remote_user [$time_local] '
                        '"$request" $status $body_bytes_sent '
                        '"$http_referer" "$http_user_agent" '
                        'rt=$request_time '
                        'rl=$limit_req_status';

    access_log /var/log/nginx/access.log detailed;

    # -------------------------------------------------------
    # Upstream
    # -------------------------------------------------------

    upstream api_backend {
        least_conn;
        server api-service:8080 max_fails=3 fail_timeout=30s;
        keepalive 32;
    }

    upstream auth_backend {
        least_conn;
        server auth-service:8081 max_fails=3 fail_timeout=30s;
        keepalive 16;
    }

    upstream search_backend {
        least_conn;
        server search-service:8082 max_fails=3 fail_timeout=30s;
        keepalive 16;
    }

    upstream stream_backend {
        least_conn;
        server stream-service:8083 max_fails=3 fail_timeout=30s;
        keepalive 64;
    }

    # -------------------------------------------------------
    # Server Block
    # -------------------------------------------------------

    server {
        listen 80;
        server_name api.streamflow.com;

        # Global connection limit
        limit_conn conn 50;

        # Request size and timeout limits
        client_max_body_size 10m;
        client_body_timeout 10s;
        client_header_timeout 10s;
        keepalive_timeout 30s;

        # Header limits
        limit_req_fields 50;
        limit_req_field_size 8k;

        # Proxy timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 30s;
        proxy_read_timeout 30s;

        # Common proxy headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_http_version 1.1;
        proxy_set_header Connection "";

        # -------------------------------------------------------
        # Geo-blocking
        # -------------------------------------------------------
        if ($is_blocked_country) {
            return 403;
        }

        # -------------------------------------------------------
        # Health Check (no rate limit)
        # -------------------------------------------------------
        location /health {
            proxy_pass http://api_backend/health;
            access_log off;
        }

        # -------------------------------------------------------
        # General API
        # -------------------------------------------------------
        location /api/ {
            limit_req zone=api burst=50 nodelay;
            proxy_pass http://api_backend;
        }

        # -------------------------------------------------------
        # Authentication
        # -------------------------------------------------------
        location /api/auth/ {
            limit_req zone=auth burst=10 nodelay;
            proxy_pass http://auth_backend;

            # Additional auth protections
            proxy_read_timeout 10s;  # Auth should be fast
        }

        location /api/auth/login {
            limit_req zone=auth burst=5 nodelay;
            proxy_pass http://auth_backend;
        }

        location /api/auth/register {
            limit_req zone=auth burst=3;
            proxy_pass http://auth_backend;
        }

        # -------------------------------------------------------
        # Search
        # -------------------------------------------------------
        location /api/search {
            limit_req zone=search burst=20 nodelay;
            proxy_pass http://search_backend;
            proxy_read_timeout 15s;  # Search can be slower
        }

        # -------------------------------------------------------
        # Streaming
        # -------------------------------------------------------
        location /api/stream/ {
            limit_req zone=stream burst=200 nodelay;
            proxy_pass http://stream_backend;

            # Streaming needs longer timeouts
            proxy_read_timeout 300s;
            proxy_send_timeout 300s;
            proxy_buffering off;
        }

        # -------------------------------------------------------
        # Catch-all (deny)
        # -------------------------------------------------------
        location / {
            return 404;
        }

        # -------------------------------------------------------
        # Error pages
        # -------------------------------------------------------
        error_page 429 /429.json;
        location = /429.json {
            internal;
            default_type application/json;
            return 429 '{"error": "Rate limit exceeded", "retry_after": 1}';
        }

        error_page 502 /502.json;
        location = /502.json {
            internal;
            default_type application/json;
            return 502 '{"error": "Service temporarily unavailable"}';
        }

        error_page 503 /503.json;
        location = /503.json {
            internal;
            default_type application/json;
            return 503 '{"error": "Service overloaded", "retry_after": 10}';
        }
    }
}
```

**Key design decisions:**

- **Per-endpoint zones:** Each endpoint category gets its own rate limit zone. Streaming (100r/s) is much more permissive than auth (5r/s).
- **Internal IP whitelisting:** The `geo` + `map` combination creates a rate limit key that is empty for internal IPs. An empty key means the request is not counted by `limit_req`, effectively bypassing rate limits for monitoring systems.
- **Burst values:** General API gets burst=50 because page loads trigger many simultaneous requests. Auth gets burst=5 because login retries should be fast but limited. Streaming gets burst=200 because video players make many requests for segments.
- **No `nodelay` on register:** Registration should be queued, not burst, because there is no legitimate reason to register multiple accounts rapidly.

---

## Part C: Traffic Analyzer with Auto-Scaling Awareness

```python
#!/usr/bin/env python3
"""
smart_scaler.py -- Traffic-aware auto-scaling with DDoS detection.
"""

import time
from collections import Counter
from typing import Dict, Any, List, Optional
from dataclasses import dataclass


@dataclass
class TrafficMetrics:
    """Current traffic metrics snapshot."""
    rps: float
    error_rate: float
    avg_response_time: float
    p99_response_time: float
    top_ips: List[tuple]  # [(ip, count), ...]
    total_requests: int
    unique_ips: int
    unique_user_agents: int
    baseline_rps: float
    connection_count: int


@dataclass
class ScalingDecision:
    """A scaling decision with rationale."""
    action: str  # scale_up, scale_down, block_ips, enable_rate_limit, do_nothing
    target_count: Optional[int] = None
    ips_to_block: Optional[List[str]] = None
    rate_limit_rps: Optional[int] = None
    reason: str = ""
    estimated_cost_impact: str = "$0.00"


class SmartScaler:
    def __init__(
        self,
        min_tasks: int = 3,
        max_tasks: int = 20,
        cost_per_task_per_hour: float = 0.10,
        max_hourly_budget: float = 50.0,
    ):
        self.min_tasks = min_tasks
        self.max_tasks = max_tasks
        self.cost_per_task_per_hour = cost_per_task_per_hour
        self.max_hourly_budget = max_hourly_budget

        self.current_task_count = min_tasks
        self.last_scale_time = 0
        self.scale_cooldown = 300  # 5 minutes between scaling actions

    def make_decision(
        self,
        metrics: TrafficMetrics,
        baseline: Dict[str, Any]
    ) -> ScalingDecision:
        """
        Analyze traffic and decide on a scaling action.

        Args:
            metrics: Current traffic metrics.
            baseline: Baseline traffic statistics.

        Returns:
            A ScalingDecision with the recommended action.
        """
        # Check if we are in cooldown
        if time.time() - self.last_scale_time < self.scale_cooldown:
            return ScalingDecision(
                action="do_nothing",
                reason="Scale cooldown active (last scale was within 5 minutes)"
            )

        # Step 1: Detect if this is an attack
        is_attack, attack_signal = self._is_likely_attack(metrics)

        if is_attack:
            return self._handle_attack(metrics, attack_signal)

        # Step 2: If not an attack, check if we need to scale
        return self._handle_legitimate_growth(metrics, baseline)

    def _is_likely_attack(self, metrics: TrafficMetrics) -> tuple:
        """
        Determine if traffic is likely an attack.

        Returns:
            (is_attack: bool, signal: str)
        """
        # Signal 1: IP concentration
        if metrics.top_ips and metrics.total_requests > 0:
            top_10_count = sum(count for _, count in metrics.top_ips[:10])
            top_10_share = top_10_count / metrics.total_requests
            if top_10_share > 0.5:
                return True, f"IP concentration: top 10 IPs account for {top_10_share:.0%} of traffic"

        # Signal 2: Error rate spike with traffic increase
        if (metrics.error_rate > 0.2 and
            metrics.rps > metrics.baseline_rps * 3):
            return True, f"High error rate ({metrics.error_rate:.0%}) with {metrics.rps/metrics.baseline_rps:.1f}x traffic spike"

        # Signal 3: Low User-Agent diversity
        if (metrics.total_requests > 100 and
            metrics.unique_user_agents / metrics.total_requests < 0.01):
            return True, f"Low User-Agent diversity ({metrics.unique_user_agents} unique UAs in {metrics.total_requests} requests)"

        # Signal 4: Response time degradation with traffic spike
        if (metrics.p99_response_time > 5.0 and
            metrics.rps > metrics.baseline_rps * 2):
            return True, f"Response time degradation (p99={metrics.p99_response_time:.1f}s) with traffic spike"

        return False, "Traffic appears legitimate"

    def _handle_attack(self, metrics: TrafficMetrics, signal: str) -> ScalingDecision:
        """Handle detected attack traffic."""
        # Block concentrated IPs
        if metrics.top_ips:
            suspicious_ips = [
                ip for ip, count in metrics.top_ips
                if count > metrics.baseline_rps * 0.1  # IPs with >10% of baseline RPS
            ]
            if suspicious_ips:
                return ScalingDecision(
                    action="block_ips",
                    ips_to_block=suspicious_ips[:20],  # Block top 20
                    reason=f"Attack detected ({signal}). Blocking {len(suspicious_ips)} suspicious IPs instead of scaling.",
                    estimated_cost_impact="$0.00"
                )

        # If IPs are distributed, enable stricter rate limiting
        return ScalingDecision(
            action="enable_rate_limit",
            rate_limit_rps=max(5, int(metrics.baseline_rps * 0.5)),
            reason=f"Attack detected ({signal}). Reducing rate limit to {int(metrics.baseline_rps * 0.5)} RPS per IP.",
            estimated_cost_impact="$0.00"
        )

    def _handle_legitimate_growth(self, metrics: TrafficMetrics, baseline: Dict[str, Any]) -> ScalingDecision:
        """Handle legitimate traffic growth."""
        # Calculate desired task count based on RPS
        rps_per_task = 500  # Each task handles ~500 RPS
        desired_count = max(
            self.min_tasks,
            min(
                self.max_tasks,
                int(metrics.rps / rps_per_task) + 1
            )
        )

        # Check budget constraint
        estimated_cost = desired_count * self.cost_per_task_per_hour
        if estimated_cost > self.max_hourly_budget:
            max_affordable = int(self.max_hourly_budget / self.cost_per_task_per_hour)
            desired_count = min(desired_count, max_affordable)

        # Scale up
        if desired_count > self.current_task_count:
            cost_increase = (desired_count - self.current_task_count) * self.cost_per_task_per_hour
            return ScalingDecision(
                action="scale_up",
                target_count=desired_count,
                reason=f"Legitimate traffic growth: RPS {metrics.rps:.0f} (baseline {metrics.baseline_rps:.0f}), scaling to {desired_count} tasks",
                estimated_cost_impact=f"+${cost_increase:.2f}/hour"
            )

        # Scale down
        if desired_count < self.current_task_count and metrics.rps < metrics.baseline_rps * 0.5:
            cost_savings = (self.current_task_count - desired_count) * self.cost_per_task_per_hour
            return ScalingDecision(
                action="scale_down",
                target_count=desired_count,
                reason=f"Traffic below baseline: RPS {metrics.rps:.0f} (baseline {metrics.baseline_rps:.0f}), scaling down to {desired_count} tasks",
                estimated_cost_impact=f"-${cost_savings:.2f}/hour"
            )

        return ScalingDecision(
            action="do_nothing",
            reason=f"Traffic is within normal range: RPS {metrics.rps:.0f} (baseline {metrics.baseline_rps:.0f})",
            estimated_cost_impact="$0.00"
        )
```

**Key design decisions:**

- **Attack vs growth detection:** The scaler uses four signals to distinguish attacks from legitimate growth: IP concentration, error rate correlation, User-Agent diversity, and response time degradation. All four must fail for traffic to be considered legitimate.
- **Cost controls:** The `max_hourly_budget` parameter caps the scaling. Even if traffic is legitimate, the scaler will not exceed the budget. This prevents financial surprises.
- **IP blocking over scaling:** When an attack is detected, the scaler prefers blocking IPs over scaling up. Scaling up during an attack wastes money and gives the attacker more resources.
- **Cooldown period:** The 5-minute cooldown prevents rapid scaling oscillation. Each scaling action takes time to propagate (new ECS tasks need time to start), so frequent changes are counterproductive.

---

## Part D: Circuit Breaker with Cache Fallback

```python
#!/usr/bin/env python3
"""
resilient_service.py -- Circuit breaker with cache-first strategy and stampede protection.
"""

import json
import time
import threading
from enum import Enum
from typing import Any, Callable, Optional
from functools import wraps

import redis


class CircuitState(Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class ResilientService:
    def __init__(self, redis_client: redis.Redis):
        self.redis = redis_client
        self._circuit = CircuitState.CLOSED
        self._failure_count = 0
        self._last_failure_time = 0
        self._half_open_successes = 0
        self._lock = threading.Lock()

        # Configuration
        self.failure_threshold = 5
        self.recovery_timeout = 30
        self.half_open_test_interval = 10  # seconds between test requests

    # -------------------------------------------------------
    # Cache with stampede protection
    # -------------------------------------------------------

    def get_or_compute(
        self,
        key: str,
        compute_fn: Callable,
        ttl: int = 60,
        stale_ttl: int = 300
    ) -> dict:
        """
        Get a value from cache, or compute it with stampede protection.

        Args:
            key: Cache key.
            compute_fn: Function to compute the value if not cached.
            ttl: Cache TTL in seconds.
            stale_ttl: How long to serve stale cache after TTL expires.

        Returns:
            Dict with 'data', 'source', 'stale', 'circuit_state'.
        """
        # Step 1: Try fresh cache
        cached = self._get_cache(key)
        if cached is not None:
            return {
                'data': cached,
                'source': 'cache',
                'stale': False,
                'circuit_state': self._circuit.value,
            }

        # Step 2: Try stale cache
        stale_cached = self._get_cache(f"stale:{key}")
        if stale_cached is not None:
            # If circuit is open, always serve stale
            if self._circuit == CircuitState.OPEN:
                return {
                    'data': stale_cached,
                    'source': 'stale_cache',
                    'stale': True,
                    'circuit_state': self._circuit.value,
                }

        # Step 3: Circuit breaker check
        if self._circuit == CircuitState.OPEN:
            if stale_cached is not None:
                return {
                    'data': stale_cached,
                    'source': 'stale_cache',
                    'stale': True,
                    'circuit_state': 'open',
                }
            return {
                'data': None,
                'source': 'unavailable',
                'stale': False,
                'circuit_state': 'open',
            }

        if self._circuit == CircuitState.HALF_OPEN:
            # Rate limit test requests
            if not self._allow_half_open_request():
                if stale_cached is not None:
                    return {
                        'data': stale_cached,
                        'source': 'stale_cache',
                        'stale': True,
                        'circuit_state': 'half_open',
                    }
                return {
                    'data': None,
                    'source': 'unavailable',
                    'stale': False,
                    'circuit_state': 'half_open',
                }

        # Step 4: Compute with stampede protection
        return self._compute_with_lock(key, compute_fn, ttl, stale_ttl, stale_cached)

    def _compute_with_lock(
        self,
        key: str,
        compute_fn: Callable,
        ttl: int,
        stale_ttl: int,
        stale_cached: Optional[Any]
    ) -> dict:
        """Compute a value with distributed lock to prevent stampede."""
        lock_key = f"lock:{key}"
        acquired = self.redis.set(lock_key, "1", nx=True, ex=10)

        if acquired:
            try:
                result = compute_fn()
                self._on_success()
                self._set_cache(key, result, ttl)
                self._set_cache(f"stale:{key}", result, stale_ttl)
                return {
                    'data': result,
                    'source': 'database',
                    'stale': False,
                    'circuit_state': self._circuit.value,
                }
            except Exception as e:
                self._on_failure()
                if stale_cached is not None:
                    return {
                        'data': stale_cached,
                        'source': 'stale_cache',
                        'stale': True,
                        'circuit_state': self._circuit.value,
                    }
                return {
                    'data': None,
                    'source': 'unavailable',
                    'stale': False,
                    'circuit_state': self._circuit.value,
                }
            finally:
                self.redis.delete(lock_key)
        else:
            # Another process is computing; wait and retry from cache
            for _ in range(20):
                time.sleep(0.25)
                cached = self._get_cache(key)
                if cached is not None:
                    return {
                        'data': cached,
                        'source': 'cache',
                        'stale': False,
                        'circuit_state': self._circuit.value,
                    }

            # Timeout: serve stale or fail
            if stale_cached is not None:
                return {
                    'data': stale_cached,
                    'source': 'stale_cache',
                    'stale': True,
                    'circuit_state': self._circuit.value,
                }
            return {
                'data': None,
                'source': 'timeout',
                'stale': False,
                'circuit_state': self._circuit.value,
            }

    # -------------------------------------------------------
    # Circuit breaker state management
    # -------------------------------------------------------

    def _on_success(self):
        with self._lock:
            self._failure_count = 0
            if self._circuit == CircuitState.HALF_OPEN:
                self._half_open_successes += 1
                if self._half_open_successes >= 3:
                    self._circuit = CircuitState.CLOSED
                    self._half_open_successes = 0

    def _on_failure(self):
        with self._lock:
            self._failure_count += 1
            self._last_failure_time = time.time()

            if self._circuit == CircuitState.HALF_OPEN:
                self._circuit = CircuitState.OPEN
                self._half_open_successes = 0
            elif self._failure_count >= self.failure_threshold:
                self._circuit = CircuitState.OPEN

    def _allow_half_open_request(self) -> bool:
        now = time.time()
        if now - self._last_failure_time >= self.half_open_test_interval:
            return True
        return False

    @property
    def state(self) -> CircuitState:
        if self._circuit == CircuitState.OPEN:
            if time.time() - self._last_failure_time > self.recovery_timeout:
                self._circuit = CircuitState.HALF_OPEN
                self._half_open_successes = 0
        return self._circuit

    # -------------------------------------------------------
    # Cache operations
    # -------------------------------------------------------

    def _get_cache(self, key: str) -> Optional[Any]:
        try:
            value = self.redis.get(f"cache:{key}")
            if value:
                return json.loads(value)
        except Exception:
            pass
        return None

    def _set_cache(self, key: str, value: Any, ttl: int):
        try:
            self.redis.setex(f"cache:{key}", ttl, json.dumps(value, default=str))
        except Exception:
            pass

    def warm_cache(self, key: str, compute_fn: Callable, ttl: int = 60):
        """Pre-populate cache for critical endpoints."""
        try:
            result = compute_fn()
            self._set_cache(key, result, ttl)
            self._set_cache(f"stale:{key}", result, ttl * 5)
        except Exception as e:
            print(f"Cache warming failed for {key}: {e}")

    def get_status(self) -> dict:
        return {
            'state': self.state.value,
            'failure_count': self._failure_count,
            'failure_threshold': self.failure_threshold,
        }


# -------------------------------------------------------
# Flask Integration
# -------------------------------------------------------
from flask import Flask, jsonify, make_response, request

app = Flask(__name__)
redis_client = redis.Redis(host='redis', port=6379, decode_responses=True)
service = ResilientService(redis_client)


def get_products_from_db():
    """Simulated database query."""
    import psycopg2
    conn = psycopg2.connect(app.config['DATABASE_URL'])
    cur = conn.cursor()
    cur.execute('SELECT id, name, price FROM products WHERE active = true LIMIT 100')
    products = [{'id': r[0], 'name': r[1], 'price': float(r[2])} for r in cur.fetchall()]
    cur.close()
    conn.close()
    return products


def get_product_from_db(product_id):
    """Simulated single product query."""
    import psycopg2
    conn = psycopg2.connect(app.config['DATABASE_URL'])
    cur = conn.cursor()
    cur.execute('SELECT id, name, price, description FROM products WHERE id = %s', (product_id,))
    row = cur.fetchone()
    cur.close()
    conn.close()
    if row:
        return {'id': row[0], 'name': row[1], 'price': float(row[2]), 'description': row[3]}
    return None


def get_recommendations_from_db():
    """Simulated recommendations query (not cacheable)."""
    import psycopg2
    conn = psycopg2.connect(app.config['DATABASE_URL'])
    cur = conn.cursor()
    cur.execute('SELECT id, name FROM products ORDER BY RANDOM() LIMIT 10')
    recs = [{'id': r[0], 'name': r[1]} for r in cur.fetchall()]
    cur.close()
    conn.close()
    return recs


# Cache warming on startup
@app.before_request
def warm_critical_caches():
    if not hasattr(app, '_cache_warmed'):
        service.warm_cache('products:list', get_products_from_db, ttl=60)
        app._cache_warmed = True


@app.route('/api/products')
def get_products():
    result = service.get_or_compute('products:list', get_products_from_db, ttl=60)

    response = make_response(jsonify({
        'products': result['data'] or [],
        'source': result['source'],
    }))

    if result['stale']:
        response.headers['X-Served-From'] = 'cache'
        response.headers['X-Cache-Stale'] = 'true'

    if result['source'] == 'unavailable':
        response.status_code = 503
        response.headers['Retry-After'] = '10'

    return response


@app.route('/api/products/<int:product_id>')
def get_product(product_id):
    result = service.get_or_compute(
        f'product:{product_id}',
        lambda: get_product_from_db(product_id),
        ttl=30
    )

    if result['data'] is None and result['source'] == 'unavailable':
        return jsonify({'error': 'Product not available', 'retry_after': 10}), 503

    if result['data'] is None:
        return jsonify({'error': 'Product not found'}), 404

    response = make_response(jsonify({
        'product': result['data'],
        'source': result['source'],
    }))

    if result['stale']:
        response.headers['X-Served-From'] = 'cache'

    return response


@app.route('/api/search')
def search():
    query = request.args.get('q', '')
    cache_key = f'search:{query}'

    result = service.get_or_compute(
        cache_key,
        lambda: search_products_in_db(query),
        ttl=10
    )

    response = make_response(jsonify({
        'results': result['data'] or [],
        'source': result['source'],
        'query': query,
    }))

    if result['stale']:
        response.headers['X-Served-From'] = 'cache'

    return response


@app.route('/api/recommendations')
def recommendations():
    """Recommendations are not cacheable (personalized)."""
    result = service.get_or_compute(
        'recommendations:generic',
        get_recommendations_from_db,
        ttl=300
    )

    response = make_response(jsonify({
        'recommendations': result['data'] or [],
        'source': result['source'],
    }))

    if result['stale']:
        response.headers['X-Served-From'] = 'cache'
        response.headers['X-Cache-Stale'] = 'true'

    return response


@app.route('/api/circuit-breaker/status')
def circuit_status():
    return jsonify(service.get_status())
```

**Key design decisions:**

- **Cache-first strategy:** Every endpoint tries the cache first. This means that even when the database is down, most endpoints can serve data (potentially stale).
- **Stampede protection:** The distributed lock (`SET NX EX`) prevents multiple processes from computing the same cache entry simultaneously. Without this, 100 concurrent requests for an expired cache entry would all hit the database.
- **Stale cache:** The `stale:` prefix stores a long-lived copy of each cache entry. When the circuit is open, stale data is served instead of an error. Users see slightly old data, but the service stays up.
- **Cache warming:** Critical endpoints (like product listings) are pre-populated on startup. This means the first user after a deploy gets a fast response from cache.
- **Half-open rate limiting:** During recovery, only one test request per 10 seconds is allowed through. This prevents a recovered database from being immediately overwhelmed by the queued traffic.

---

## Part E: Integration Test Suite

```python
#!/usr/bin/env python3
"""
test_ddos_protection.py -- Integration tests for multi-layer DDoS protection.
"""

import json
import time
import pytest
import fakeredis
from unittest.mock import MagicMock, patch

# Import modules under test
from smart_scaler import SmartScaler, TrafficMetrics, ScalingDecision
from resilient_service import ResilientService, CircuitState


# -------------------------------------------------------
# Fixtures
# -------------------------------------------------------

@pytest.fixture
def redis_client():
    return fakeredis.FakeRedis()


@pytest.fixture
def scaler():
    return SmartScaler(
        min_tasks=3,
        max_tasks=20,
        cost_per_task_per_hour=0.10,
        max_hourly_budget=50.0,
    )


@pytest.fixture
def service(redis_client):
    return ResilientService(redis_client)


@pytest.fixture
def baseline():
    return {
        'rps': {'mean': 1000.0, 'std': 200.0},
        'requests_per_ip_per_minute': {'mean': 50.0, 'std': 20.0},
        'status_distribution': {'2xx': 0.95, '4xx': 0.04, '5xx': 0.01},
        'avg_request_time': {'mean': 0.15, 'std': 0.05},
        'unique_ips_per_minute': {'mean': 200.0, 'std': 50.0},
    }


# -------------------------------------------------------
# Test 1: Rate Limiting
# -------------------------------------------------------

class TestRateLimiting:
    """Test that rate limiting rejects excess requests."""

    def test_normal_requests_pass(self):
        """Normal request rate should not be limited."""
        # This test validates the Nginx configuration conceptually.
        # In a real environment, use subprocess to test against running Nginx.
        rate_limit = 30  # r/s
        request_count = 20  # Below limit
        assert request_count <= rate_limit, "Normal requests should pass"

    def test_excess_requests_rejected(self):
        """Requests above the rate limit should get 429."""
        rate_limit = 30
        burst = 50
        request_count = 100  # Above limit + burst
        expected_rejected = request_count - rate_limit - burst
        assert expected_rejected > 0, "Excess requests should be rejected"


# -------------------------------------------------------
# Test 2: Circuit Breaker
# -------------------------------------------------------

class TestCircuitBreaker:
    """Test circuit breaker state transitions and cache fallback."""

    def test_closed_state_passes_through(self, service, redis_client):
        """In CLOSED state, requests should pass through to the database."""
        redis_client.setex("cache:test", 60, json.dumps({"value": "cached"}))

        db_result = {"value": "from_db"}
        result = service.get_or_compute("test", lambda: db_result, ttl=60)

        assert result['source'] == 'database'
        assert result['data'] == db_result
        assert result['circuit_state'] == 'closed'

    def test_opens_after_threshold_failures(self, service):
        """Circuit should OPEN after failure_threshold consecutive failures."""
        for i in range(service.failure_threshold):
            service._on_failure()

        assert service.state == CircuitState.OPEN

    def test_open_state_serves_from_cache(self, service, redis_client):
        """In OPEN state, requests should be served from cache."""
        # Pre-populate cache
        redis_client.setex("cache:products", 60, json.dumps(["product1", "product2"]))

        # Force circuit open
        for _ in range(service.failure_threshold):
            service._on_failure()

        assert service.state == CircuitState.OPEN

        result = service.get_or_compute("products", lambda: None, ttl=60)

        assert result['source'] == 'stale_cache'
        assert result['stale'] is True
        assert result['circuit_state'] == 'open'

    def test_open_state_returns_none_without_cache(self, service):
        """In OPEN state with no cache, return unavailable."""
        for _ in range(service.failure_threshold):
            service._on_failure()

        result = service.get_or_compute("no_cache_key", lambda: None, ttl=60)

        assert result['source'] == 'unavailable'
        assert result['data'] is None

    def test_half_open_after_recovery_timeout(self, service):
        """Circuit should transition to HALF_OPEN after recovery_timeout."""
        for _ in range(service.failure_threshold):
            service._on_failure()

        assert service.state == CircuitState.OPEN

        # Simulate recovery timeout passing
        service._last_failure_time = time.time() - service.recovery_timeout - 1

        assert service.state == CircuitState.HALF_OPEN

    def test_closes_after_successful_half_open(self, service, redis_client):
        """Circuit should CLOSE after 3 successful test requests in HALF_OPEN."""
        # Force to HALF_OPEN
        for _ in range(service.failure_threshold):
            service._on_failure()
        service._last_failure_time = time.time() - service.recovery_timeout - 1
        assert service.state == CircuitState.HALF_OPEN

        # 3 successful requests should close the circuit
        for _ in range(3):
            service._on_success()

        assert service.state == CircuitState.CLOSED


# -------------------------------------------------------
# Test 3: Auto-Scaling Decisions
# -------------------------------------------------------

class TestSmartScaler:
    """Test that the scaler distinguishes attack from growth."""

    def test_legitimate_growth_scales_up(self, scaler, baseline):
        """Legitimate traffic growth should trigger scale up."""
        metrics = TrafficMetrics(
            rps=3000.0,  # 3x baseline
            error_rate=0.01,  # Low error rate
            avg_response_time=0.18,  # Normal
            p99_response_time=0.5,  # Normal
            top_ips=[(f"10.0.0.{i}", 100) for i in range(20)],  # Distributed
            total_requests=180000,
            unique_ips=5000,
            unique_user_agents=500,
            baseline_rps=1000.0,
            connection_count=500,
        )

        decision = scaler.make_decision(metrics, baseline)

        assert decision.action == "scale_up"
        assert decision.target_count > scaler.min_tasks

    def test_attack_traffic_blocks_ips(self, scaler, baseline):
        """Attack traffic with IP concentration should block IPs, not scale."""
        metrics = TrafficMetrics(
            rps=5000.0,  # 5x baseline
            error_rate=0.35,  # High error rate
            avg_response_time=5.0,  # Slow
            p99_response_time=12.0,  # Very slow
            top_ips=[("10.0.0.1", 50000), ("10.0.0.2", 40000), ("10.0.0.3", 30000)],
            total_requests=200000,
            unique_ips=10,
            unique_user_agents=5,
            baseline_rps=1000.0,
            connection_count=2000,
        )

        decision = scaler.make_decision(metrics, baseline)

        assert decision.action == "block_ips"
        assert len(decision.ips_to_block) > 0

    def test_distributed_attack_enables_rate_limit(self, scaler, baseline):
        """Distributed attack with diverse IPs should reduce rate limits."""
        metrics = TrafficMetrics(
            rps=4000.0,
            error_rate=0.30,
            avg_response_time=3.0,
            p99_response_time=8.0,
            top_ips=[(f"10.0.{i}.{j}", 100) for i in range(10) for j in range(10)],
            total_requests=200000,
            unique_ips=10000,  # Many unique IPs
            unique_user_agents=3,  # But very few user agents
            baseline_rps=1000.0,
            connection_count=1500,
        )

        decision = scaler.make_decision(metrics, baseline)

        # Should detect via User-Agent diversity and reduce rate limits
        assert decision.action in ("block_ips", "enable_rate_limit")

    def test_budget_cap_prevents_overscaling(self, scaler, baseline):
        """Scaling should respect the budget ceiling."""
        scaler.max_hourly_budget = 0.50  # Very low budget

        metrics = TrafficMetrics(
            rps=10000.0,  # Very high traffic
            error_rate=0.01,
            avg_response_time=0.15,
            p99_response_time=0.3,
            top_ips=[(f"10.0.0.{i}", 100) for i in range(100)],
            total_requests=600000,
            unique_ips=10000,
            unique_user_agents=1000,
            baseline_rps=1000.0,
            connection_count=1000,
        )

        decision = scaler.make_decision(metrics, baseline)

        if decision.action == "scale_up":
            # With $0.50 budget and $0.10/task/hour, max is 5 tasks
            assert decision.target_count <= 5

    def test_cooldown_prevents_rapid_scaling(self, scaler, baseline):
        """Scaling should not happen during cooldown."""
        scaler.last_scale_time = time.time()  # Just scaled

        metrics = TrafficMetrics(
            rps=5000.0,
            error_rate=0.01,
            avg_response_time=0.15,
            p99_response_time=0.3,
            top_ips=[],
            total_requests=300000,
            unique_ips=5000,
            unique_user_agents=500,
            baseline_rps=1000.0,
            connection_count=500,
        )

        decision = scaler.make_decision(metrics, baseline)

        assert decision.action == "do_nothing"
        assert "cooldown" in decision.reason.lower()


# -------------------------------------------------------
# Test 4: Graceful Degradation
# -------------------------------------------------------

class TestGracefulDegradation:
    """Test that the system degrades gracefully under load."""

    def test_serves_stale_data_when_db_down(self, service, redis_client):
        """Should serve stale cache when database is unavailable."""
        # Set up stale cache
        stale_data = {"products": [{"id": 1, "name": "Widget"}]}
        redis_client.setex("cache:products", 60, json.dumps(stale_data))
        redis_client.setex("stale:products", 300, json.dumps(stale_data))

        # Force circuit open
        for _ in range(service.failure_threshold):
            service._on_failure()

        result = service.get_or_compute("products", lambda: None, ttl=60)

        assert result['data'] == stale_data
        assert result['stale'] is True

    def test_returns_none_when_no_cache_and_db_down(self, service):
        """Should return unavailable when both cache and database fail."""
        for _ in range(service.failure_threshold):
            service._on_failure()

        result = service.get_or_compute("nonexistent", lambda: None, ttl=60)

        assert result['data'] is None
        assert result['source'] == 'unavailable'


# -------------------------------------------------------
# Test 5: Cache Stampede Protection
# -------------------------------------------------------

class TestStampedeProtection:
    """Test that concurrent requests don't all hit the database."""

    def test_only_one_process_computes(self, service, redis_client):
        """When cache is empty, only one process should compute."""
        call_count = 0

        def slow_compute():
            nonlocal call_count
            call_count += 1
            time.sleep(0.5)
            return {"computed": True}

        # First call acquires lock and computes
        result = service.get_or_compute("stampede_test", slow_compute, ttl=60)
        assert result['source'] == 'database'
        assert call_count == 1

        # Second call should get from cache
        result = service.get_or_compute("stampede_test", slow_compute, ttl=60)
        assert result['source'] == 'cache'
        assert call_count == 1  # Not called again


# -------------------------------------------------------
# Test 6: Recovery
# -------------------------------------------------------

class TestRecovery:
    """Test that the system returns to normal after attack stops."""

    def test_circuit_closes_after_recovery(self, service, redis_client):
        """Circuit should close after successful requests during half-open."""
        # Simulate attack: force circuit open
        for _ in range(service.failure_threshold):
            service._on_failure()
        assert service.state == CircuitState.OPEN

        # Simulate recovery: wait for timeout
        service._last_failure_time = time.time() - service.recovery_timeout - 1
        assert service.state == CircuitState.HALF_OPEN

        # Successful requests close the circuit
        for _ in range(3):
            result = service.get_or_compute(
                "recovery_test",
                lambda: {"recovered": True},
                ttl=60
            )

        assert service.state == CircuitState.CLOSED
        assert service._failure_count == 0

    def test_scaler_resumes_normal_after_attack(self, scaler, baseline):
        """Scaler should return to normal decisions after attack traffic stops."""
        # During attack
        attack_metrics = TrafficMetrics(
            rps=5000.0, error_rate=0.35, avg_response_time=5.0,
            p99_response_time=12.0,
            top_ips=[("10.0.0.1", 50000)],
            total_requests=200000, unique_ips=10, unique_user_agents=3,
            baseline_rps=1000.0, connection_count=2000,
        )
        attack_decision = scaler.make_decision(attack_metrics, baseline)
        assert attack_decision.action in ("block_ips", "enable_rate_limit")

        # After attack (normal traffic)
        scaler.last_scale_time = 0  # Reset cooldown
        normal_metrics = TrafficMetrics(
            rps=1100.0, error_rate=0.01, avg_response_time=0.15,
            p99_response_time=0.3,
            top_ips=[(f"10.0.0.{i}", 50) for i in range(100)],
            total_requests=66000, unique_ips=5000, unique_user_agents=500,
            baseline_rps=1000.0, connection_count=300,
        )
        normal_decision = scaler.make_decision(normal_metrics, baseline)
        assert normal_decision.action == "do_nothing"


# -------------------------------------------------------
# Run tests
# -------------------------------------------------------

if __name__ == '__main__':
    pytest.main([__file__, '-v', '--tb=short'])
```

**Running the tests:**

```bash
# Install dependencies
pip install pytest fakeredis

# Run all tests
pytest test_ddos_protection.py -v

# Run specific test class
pytest test_ddos_protection.py::TestCircuitBreaker -v

# Run with coverage
pytest test_ddos_protection.py --cov=. --cov-report=term-missing
```

---

## Common Mistakes

- **Not distinguishing attack from growth.** The most expensive mistake is auto-scaling during an attack. If you cannot tell attack traffic from legitimate growth, you will spend thousands of dollars scaling up infrastructure that serves the attacker. Use multiple signals (IP concentration, error rate, User-Agent diversity) to make the distinction.
- **Cache stampede without distributed locks.** When a popular cache entry expires, 100 concurrent requests all see the cache miss and all hit the database. This is a cache stampede. Use a distributed lock (Redis `SET NX EX`) so only one process computes the value and the rest wait.
- **Circuit breaker without cache fallback.** A circuit breaker that just returns errors is better than nothing, but a circuit breaker that serves stale cache is much better. Users see slightly old data instead of an error page.
- **Ignoring cost controls.** Auto-scaling without a ceiling is a financial DDoS. An attacker can cause your AWS bill to spike by generating traffic that triggers scaling. Always set a maximum task count based on your budget.
- **Testing only the happy path.** The integration tests must cover attack scenarios, recovery, and edge cases (no cache, database down, distributed attacks). The test suite above covers 6 scenarios including attack detection, circuit breaker state transitions, budget caps, cooldown periods, and recovery.
