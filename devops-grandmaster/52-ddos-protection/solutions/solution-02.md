# Solution 02: Implement Rate Limiting with Nginx

## Part A: Rate Limiting Zones

The complete `http` block with all rate limiting zones defined:

```nginx
worker_processes auto;

events {
    worker_connections 1024;
}

http {
    upstream backend {
        server app:5000;
    }

    # --- Rate Limiting Zones ---

    # Zone 1: General API -- 50 requests/second per IP
    limit_req_zone $binary_remote_addr zone=general_api:10m rate=50r/s;

    # Zone 2: Authentication -- 2 requests/second per IP
    limit_req_zone $binary_remote_addr zone=auth:10m rate=2r/s;

    # Zone 3: Report generation -- 6 requests/minute per IP (1 every 10 seconds)
    limit_req_zone $binary_remote_addr zone=reports:10m rate=6r/m;

    # Zone 4: Search -- 5 requests/second per IP
    limit_req_zone $binary_remote_addr zone=search:10m rate=5r/s;

    # Zone 5: Connection limiting -- concurrent connections per IP
    limit_conn_zone $binary_remote_addr zone=conn_limit:10m;

    # --- Error Status Codes ---
    limit_req_status 429;
    limit_conn_status 429;

    # ... (server block continues in Part B)
}
```

### Why `$binary_remote_addr` and not `$remote_addr`

`$binary_remote_addr` stores an IPv4 address as a 4-byte binary value (16 bytes for IPv6), whereas `$remote_addr` stores it as a variable-length ASCII string. In a shared memory zone of 10 MB, the binary form stores roughly 160,000 IPv4 addresses versus far fewer with the string form. This is a *direct cost savings* -- the same zone size tracks more IPs.

### Why separate zones instead of one zone with the strictest rate

Each zone maintains its own sliding window counter per IP. If you used one zone at `2r/s` for everything, then `/api/data` (which legitimately needs 50 req/s) would be throttled to 2 req/s. Separate zones let you apply different rates to different endpoint categories while sharing the same per-IP key.

### Zone sizing rationale

A `10m` zone allocates approximately 10 MB of shared memory. For IPv4 addresses stored in binary form, this holds about 160,000 unique IP addresses. For a typical application behind a CDN or load balancer, this is sufficient. If your user base is larger (millions of IPs), increase to `20m` or `50m`. The memory is allocated once per worker process if using shared memory, so it is efficient.

---

## Part B: Location Blocks with Rate Limits

```nginx
    server {
        listen 80;

        # --- Health check: NO rate limit ---
        location /health {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # --- Static assets: NO rate limit ---
        location /static/ {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # --- General data API: 50r/s, burst 30, nodelay ---
        location /api/data {
            limit_req zone=general_api burst=30 nodelay;
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # --- Login: 2r/s, burst 3, nodelay ---
        location /api/auth/login {
            limit_req zone=auth burst=3 nodelay;
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # --- Registration: 2r/s, burst 2, NO nodelay (stricter) ---
        location /api/auth/register {
            limit_req zone=auth burst=2;
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # --- Report generation: 6r/m, burst 1, NO nodelay ---
        location /api/reports/generate {
            limit_req zone=reports burst=1;
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # --- Search: 5r/s, burst 10, nodelay ---
        location /api/search {
            limit_req zone=search burst=10 nodelay;
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # --- Catch-all for any other API endpoints ---
        location /api/ {
            limit_req zone=general_api burst=20 nodelay;
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }

        # --- Custom 429 error page ---
        error_page 429 /429.json;
        location = /429.json {
            internal;
            default_type application/json;
            return 429 '{"error": "Rate limit exceeded", "retry_after": 1}';
        }
    }
}
```

### Why these burst and nodelay choices

The `burst` parameter defines how many requests can exceed the rate limit before Nginx starts rejecting. The `nodelay` parameter controls what happens to burst requests:

| Endpoint | Rate | Burst | nodelay | Rationale |
|----------|------|-------|---------|-----------|
| `/api/data` | 50r/s | 30 | yes | General API users may click rapidly or page through results. A burst of 30 handles legitimate spikes (a page load triggers multiple API calls). `nodelay` processes them immediately rather than queuing. |
| `/api/auth/login` | 2r/s | 3 | yes | Users may mistype and retry quickly. A burst of 3 allows 2 rapid retries. `nodelay` is acceptable because login should feel responsive. |
| `/api/auth/register` | 2r/s | 2 | **no** | Registration is sensitive -- attackers use it to create fake accounts. A smaller burst (2) with *no* nodelay means excess requests are *delayed* rather than processed immediately, slowing down automated registration attempts. |
| `/api/reports/generate` | 6r/m | 1 | **no** | Report generation is extremely expensive. A burst of 1 means at most 2 requests can be in flight (1 at rate + 1 burst). No nodelay ensures the rate is strictly enforced, preventing resource exhaustion. |
| `/api/search` | 5r/s | 10 | yes | Search users type, search, refine, search again. The burst of 10 handles the rapid interaction pattern. `nodelay` keeps search responsive. |

The critical distinction: **`nodelay` processes burst requests immediately** (they consume burst slots but are served without waiting). **Without `nodelay`**, burst requests are *queued and delayed* to maintain the configured rate. For sensitive endpoints like registration and report generation, the delay acts as a natural throttle against abuse.

---

## Part C: Connection Limits and Timeouts

Add these directives inside the `server` block (or in the `http` block for server-wide defaults):

```nginx
    server {
        listen 80;

        # --- Connection limits (Slowloris protection) ---
        # Maximum 20 concurrent connections per IP
        limit_conn conn_limit 20;

        # --- Timeout protections ---
        # Close connections that take too long to send headers (Slowloris defense)
        client_header_timeout 10s;

        # Close connections that take too long to send body
        client_body_timeout 10s;

        # Limit request body size to 1 MB
        client_max_body_size 1m;

        # Close idle keep-alive connections promptly
        keepalive_timeout 15s;

        # --- Header limits ---
        # Maximum 50 headers per request
        limit_req_fields 50;

        # Maximum size of each header: 4 KB
        limit_req_field_size 4k;

        # ... (location blocks from Part B)
    }
```

### Why these specific values matter for Slowloris

Slowloris works by opening many connections and sending HTTP headers extremely slowly (1 byte per second). Each incomplete connection holds a server slot. The defenses are layered:

| Directive | Value | What it prevents |
|-----------|-------|------------------|
| `limit_conn conn_limit 20` | 20 concurrent connections per IP | Caps the number of slow connections a single attacker can hold open. Even if they send 1 byte/sec, they can only hold 20 slots. |
| `client_header_timeout 10s` | 10 seconds | If a client does not send complete headers within 10 seconds, Nginx closes the connection. This is the *primary* Slowloris defense -- slow headers are terminated. |
| `client_body_timeout 10s` | 10 seconds | If a client stops sending body data for 10 seconds, the connection is closed. Prevents slow POST body attacks. |
| `client_max_body_size 1m` | 1 MB | Prevents large body attacks that consume memory and upload bandwidth. |
| `keepalive_timeout 15s` | 15 seconds | Idle keep-alive connections are closed after 15 seconds. Prevents attackers from holding connections open without sending requests. |
| `limit_req_fields 50` | 50 headers | Prevents header bloat attacks that send thousands of headers to consume parsing resources. |
| `limit_req_field_size 4k` | 4 KB per header | Prevents oversized headers that consume memory. |

Without these timeouts, an attacker can open 10,000 connections and hold them indefinitely with 1 byte/sec traffic. With `client_header_timeout 10s`, each connection is closed after 10 seconds of incomplete headers, capping the attacker to roughly 2,000 concurrent slow connections (10,000 opened / 5 per timeout cycle) -- and `limit_conn 20` caps that further to 20 per IP.

---

## Part D: Custom Error Responses

The 429 error page is already included in the Part B configuration. Here is the complete configuration with all error handling additions:

```nginx
http {
    # ... zones from Part A ...

    limit_req_status 429;
    limit_conn_status 429;

    server {
        listen 80;

        # ... connection limits from Part C ...
        # ... location blocks from Part B ...

        # --- Custom 429 JSON error page ---
        error_page 429 /429.json;

        location = /429.json {
            internal;
            default_type application/json;
            return 429 '{"error": "Rate limit exceeded", "retry_after": 1}';
        }
    }
}
```

### Why `internal` on the error location

The `internal` directive prevents clients from directly requesting `/429.json`. Without it, a client could `GET /429.json` and receive the error page as a normal response. The `internal` directive ensures Nginx only serves this location when it is internally redirected via `error_page`.

### A note on rate limit response headers

Nginx's built-in `limit_req` module does not expose the remaining quota as a variable, so you cannot natively add `X-RateLimit-Limit` and `X-RateLimit-Remaining` headers. To achieve this, you need one of:

- **OpenResty with Lua**: Use `lua-resty-limit-req` which exposes the remaining count as a variable you can inject into response headers.
- **API Gateway (Kong, AWS API Gateway)**: These natively add rate limit headers to every response.
- **Application-level rate limiting**: The application tracks rate limit state and sets its own headers.

For a pure Nginx solution, the 429 JSON response body is the most practical feedback mechanism. Clients receive clear error information when they are rate-limited, and monitoring systems can track 429 rates via access logs.

---

## Part E: Test Script

```bash
#!/bin/bash
# test-rate-limits.sh
# Validates Nginx rate limiting configuration

set -euo pipefail

BASE_URL="${1:-http://localhost}"
PASS=0
FAIL=0

check() {
    local description="$1"
    local expected="$2"
    local actual="$3"

    if [ "$actual" = "$expected" ]; then
        echo "  PASS: $description (got $actual)"
        PASS=$((PASS + 1))
    else
        echo "  FAIL: $description (expected $expected, got $actual)"
        FAIL=$((FAIL + 1))
    fi
}

echo "=== Test 1: Normal request to /api/data succeeds ==="
STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/data")
check "Single request returns 200" "200" "$STATUS"

echo ""
echo "=== Test 2: Rapid requests to /api/data trigger 429 ==="
RATE_LIMITED=0
# Sequential test for reliable counting
for i in $(seq 1 200); do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/data")
    if [ "$STATUS" = "429" ]; then
        RATE_LIMITED=$((RATE_LIMITED + 1))
    fi
done
if [ "$RATE_LIMITED" -gt 0 ]; then
    echo "  PASS: Got $RATE_LIMITED rate-limited responses out of 200"
    PASS=$((PASS + 1))
else
    echo "  FAIL: No 429 responses received"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "=== Test 3: /health is never rate limited ==="
HEALTH_FAILURES=0
for i in $(seq 1 200); do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/health")
    if [ "$STATUS" != "200" ]; then
        HEALTH_FAILURES=$((HEALTH_FAILURES + 1))
    fi
done
if [ "$HEALTH_FAILURES" -eq 0 ]; then
    echo "  PASS: All 200 health check requests returned 200"
    PASS=$((PASS + 1))
else
    echo "  FAIL: $HEALTH_FAILURES health checks failed"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "=== Test 4: /api/auth/login has stricter limits than /api/data ==="
# Send rapid requests to each and compare 429 rates
DATA_429=0
for i in $(seq 1 50); do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/data")
    if [ "$STATUS" = "429" ]; then
        DATA_429=$((DATA_429 + 1))
    fi
done

# Small delay to partially reset rate windows
sleep 1

LOGIN_429=0
for i in $(seq 1 50); do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" "$BASE_URL/api/auth/login")
    if [ "$STATUS" = "429" ]; then
        LOGIN_429=$((LOGIN_429 + 1))
    fi
done

if [ "$LOGIN_429" -gt "$DATA_429" ]; then
    echo "  PASS: Login endpoint is stricter ($LOGIN_429 vs $DATA_429 rate-limited)"
    PASS=$((PASS + 1))
else
    echo "  FAIL: Login not stricter (login: $LOGIN_429, data: $DATA_429)"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "=== Test 5: 429 response has JSON body ==="
# Rapid-fire to trigger a 429, capture the body
BODY=""
for i in $(seq 1 100); do
    RESPONSE=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/auth/login")
    STATUS=$(echo "$RESPONSE" | tail -1)
    BODY=$(echo "$RESPONSE" | head -1)
    if [ "$STATUS" = "429" ]; then
        break
    fi
done

if echo "$BODY" | grep -q "Rate limit exceeded"; then
    echo "  PASS: 429 response contains JSON error message"
    PASS=$((PASS + 1))
else
    echo "  FAIL: 429 response body: $BODY"
    FAIL=$((FAIL + 1))
fi

echo ""
echo "=== Test 6: Concurrent connections are limited ==="
# Open many concurrent connections and check for failures
CONN_FAILURES=0
for i in $(seq 1 30); do
    curl -s -o /dev/null "$BASE_URL/api/data" &
done
# While those are running, try more
sleep 0.1
for i in $(seq 1 10); do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 5 "$BASE_URL/api/data")
    if [ "$STATUS" = "503" ] || [ "$STATUS" = "429" ] || [ "$STATUS" = "000" ]; then
        CONN_FAILURES=$((CONN_FAILURES + 1))
    fi
done
wait

if [ "$CONN_FAILURES" -gt 0 ]; then
    echo "  PASS: Connection limiting active ($CONN_FAILURES connection failures)"
    PASS=$((PASS + 1))
else
    echo "  WARN: Could not verify connection limiting (may need more concurrent connections)"
fi

echo ""
echo "=============================="
echo "Results: $PASS passed, $FAIL failed"
echo "=============================="

if [ "$FAIL" -gt 0 ]; then
    exit 1
fi
```

---

## Common Mistakes

**Mistake 1: Using one rate limit zone for all endpoints.** If you define a single zone at the strictest rate (1 req/s for registration), every endpoint gets that rate. A user browsing `/api/data` would be throttled to 1 request per second, making the application unusable. Each endpoint category needs its own zone with an appropriate rate.

**Mistake 2: Forgetting `nodelay` and wondering why requests hang.** Without `nodelay`, excess burst requests are *delayed* (held in a queue by Nginx) to maintain the configured rate. The client experiences this as a slow response, not a 429 error. For most endpoints, `nodelay` provides a better user experience -- burst requests are served immediately, and only truly excessive traffic gets 429.

**Mistake 3: Applying rate limits to `/health`.** Health check endpoints must be unlimited. If a load balancer's health checks are rate-limited, the health check fails, the load balancer marks the server as unhealthy, and traffic is routed away -- causing an outage *caused by your own rate limiting*.

**Mistake 4: Setting `client_header_timeout` too low.** A value of 1-2 seconds will reject legitimate clients on slow mobile connections or high-latency networks. 10 seconds is a reasonable balance between Slowloris protection and usability.

**Mistake 5: Using `$remote_addr` behind a load balancer.** If your Nginx is behind a load balancer or CDN, `$remote_addr` will always be the load balancer's IP. You need to use `set_real_ip_from` and `real_ip_header` directives to extract the real client IP from `X-Forwarded-For` before applying rate limits. Otherwise, all clients share one rate limit counter (the load balancer's IP), and a single abusive client triggers rate limiting for everyone.

**Mistake 6: Not testing with concurrent connections.** Sequential `curl` tests may not trigger connection limits because connections close between requests. Use background processes (`&`) or tools like `ab` (Apache Bench) or `wrk` to create genuine concurrent load.

**Mistake 7: Placing location blocks in the wrong order.** Nginx uses the *first matching* location for prefix matches. If you put `location /api/` before `location /api/auth/login`, the catch-all `/api/` block matches first and the stricter auth limit is never applied. Place more specific locations before more general ones.

---

## Key Takeaway

Rate limiting is not a single setting -- it is a *per-endpoint configuration* that matches each endpoint's sensitivity and traffic pattern. Authentication endpoints need strict limits (1-2 req/s) to prevent brute force. Expensive operations need very low rates (1 per 10 seconds) to prevent resource exhaustion. General API endpoints need moderate limits with burst tolerance for normal usage spikes. The `burst` and `nodelay` parameters control whether excess traffic is delayed (gentle throttle) or rejected (hard limit). Connection limits and timeouts are the final layer, protecting against slow connection attacks that bypass rate limiting by holding connections open without sending requests at high rates.
