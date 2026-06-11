# Exercise 02: Implement Rate Limiting with Nginx

**Type:** Guided
**Time:** 30 minutes
**Difficulty:** Easy-Medium

## Objective

Configure a complete Nginx rate limiting setup that protects different API endpoints with appropriate limits per endpoint, handles burst traffic gracefully, and returns proper error responses. You will work with `limit_req_zone`, `limit_conn_zone`, burst handling, and custom error pages.

## Scenario

Your team has a web application behind Nginx with the following endpoints:

| Endpoint | Purpose | Expected Traffic | Sensitivity |
|----------|---------|-----------------|-------------|
| `/api/data` | General data API | 50 req/s per client | Medium |
| `/api/auth/login` | User login | 2 req/s per client | High |
| `/api/auth/register` | User registration | 1 req/s per client | High |
| `/api/reports/generate` | Report generation | 1 req/10s per client | Very High |
| `/api/search` | Search with DB queries | 5 req/s per client | Medium |
| `/health` | Health check | Unlimited | None |
| `/static/*` | Static assets | Unlimited | None |

Currently, the Nginx configuration has no rate limiting at all.

**The unprotected Nginx configuration:**

```nginx
worker_processes auto;

events {
    worker_connections 1024;
}

http {
    upstream backend {
        server app:5000;
    }

    server {
        listen 80;

        location / {
            proxy_pass http://backend;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
        }
    }
}
```

## Tasks

### Part A: Define Rate Limiting Zones

Add `limit_req_zone` directives in the `http` block for each endpoint category. Each zone needs a key (use `$binary_remote_addr` for per-IP tracking), a zone name with shared memory size, and a rate.

You need zones for:
1. **General API** -- 50 requests/second per IP
2. **Authentication** -- 2 requests/second per IP
3. **Report generation** -- 6 requests/minute per IP (1 every 10 seconds)
4. **Search** -- 5 requests/second per IP
5. **Connection limiting** -- limit concurrent connections per IP

Also add a `limit_conn_zone` for tracking concurrent connections.

<details>
<summary>Hint 1: Zone syntax</summary>

The syntax for a rate limit zone is:
```nginx
limit_req_zone $binary_remote_addr zone=NAME:SIZE rate=RATE;
```
Use `$binary_remote_addr` as the key (it is more memory-efficient than `$remote_addr`). Zone size of `10m` stores approximately 160,000 IP addresses. Rate can be specified as `r/s` (per second) or `r/m` (per minute).

</details>

<details>
<summary>Hint 2: Connection zone syntax</summary>

The connection zone syntax is:
```nginx
limit_conn_zone $binary_remote_addr zone=conn_zone:10m;
```
This tracks the number of concurrent connections per IP address.

</details>

### Part B: Apply Rate Limits to Locations

Create separate `location` blocks for each endpoint and apply the appropriate rate limit zone using `limit_req`. Use the `burst` and `nodelay` parameters to handle traffic bursts gracefully.

For each location, configure:
- The appropriate `limit_req zone=...` directive
- A `burst` value that allows reasonable traffic spikes
- The `nodelay` parameter where appropriate
- The `proxy_pass` to the backend

<details>
<summary>Hint 1: Burst and nodelay</summary>

The `burst` parameter defines how many requests can exceed the rate before being rejected. For example, `burst=20 nodelay` means 20 extra requests can be queued and processed immediately (without waiting). Without `nodelay`, excess requests are delayed to maintain the rate. Use `nodelay` for most endpoints; use a small burst without `nodelay` for sensitive endpoints like login.

</details>

<details>
<summary>Hint 2: Endpoint-specific limits</summary>

- `/api/auth/login`: Use the authentication zone, burst=3, nodelay
- `/api/auth/register`: Use the authentication zone, burst=2, no nodelay (stricter)
- `/api/reports/generate`: Use the report zone, burst=1, no nodelay
- `/api/search`: Use the search zone, burst=10, nodelay
- `/api/data`: Use the general API zone, burst=30, nodelay
- `/health` and `/static/`: No rate limit

</details>

### Part C: Add Connection Limits and Timeouts

Add connection limiting and timeout protections:

1. Apply `limit_conn` to limit concurrent connections per IP (suggest: 20).
2. Set `client_body_timeout` and `client_header_timeout` to protect against Slowloris attacks (suggest: 10s).
3. Set `client_max_body_size` to limit request body size (suggest: 1m).
4. Set `keepalive_timeout` to close idle connections promptly (suggest: 15s).
5. Limit the number and size of request headers.

<details>
<summary>Hint: Slowloris protection</summary>

Slowloris works by opening many connections and sending headers very slowly. The key defenses are:
- `client_header_timeout 10s;` -- close connections that take too long to send headers
- `client_body_timeout 10s;` -- close connections that take too long to send body
- `keepalive_timeout 15s;` -- close idle keep-alive connections
- `limit_conn` -- cap the number of concurrent connections per IP

</details>

### Part D: Configure Custom Error Responses

Configure Nginx to return proper JSON error responses when rate limits are triggered:

1. Set `limit_req_status` and `limit_conn_status` to return HTTP 429 (Too Many Requests).
2. Create a custom error page that returns a JSON response with the error message and a `Retry-After` hint.
3. Add rate limit headers to responses (`X-RateLimit-Limit`, `X-RateLimit-Remaining`) using `add_header` in the error handling.

<details>
<summary>Hint: Custom 429 page</summary>

Use the `error_page` directive and an internal location block:
```nginx
error_page 429 /429.json;

location = /429.json {
    internal;
    default_type application/json;
    return 429 '{"error": "Rate limit exceeded", "retry_after": 1}';
}
```

</details>

### Part E: Test Your Configuration

Write a shell script called `test-rate-limits.sh` that validates your Nginx configuration works correctly:

1. Test that normal requests to `/api/data` succeed (HTTP 200).
2. Send rapid requests and verify that some return HTTP 429.
3. Test that `/health` is never rate limited.
4. Test that `/api/auth/login` has stricter limits than `/api/data`.
5. Test that concurrent connections are limited.

<details>
<summary>Hint: Testing with curl</summary>

Use curl with the `-o` and `-w` flags to capture the HTTP status code:
```bash
# Test single request
STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost/api/data)
echo "Status: $STATUS"

# Test rapid requests
for i in $(seq 1 100); do
    curl -s -o /dev/null -w "%{http_code}\n" http://localhost/api/data &
done
wait

# Test health endpoint (should never be rate limited)
for i in $(seq 1 100); do
    STATUS=$(curl -s -o /dev/null -w "%{http_code}" http://localhost/health)
    if [ "$STATUS" != "200" ]; then
        echo "FAIL: Health endpoint returned $STATUS"
    fi
done
```

</details>

---

## Success Criteria

- [ ] The Nginx configuration defines separate rate limit zones for each endpoint category.
- [ ] `/api/auth/login` has a stricter rate limit than `/api/data`.
- [ ] `/health` and `/static/*` are not subject to rate limiting.
- [ ] `burst` values are configured appropriately for each endpoint.
- [ ] Connection limits protect against Slowloris-style attacks.
- [ ] Rate-limited requests receive HTTP 429 with a JSON error body.
- [ ] The test script validates all rate limiting behaviors.

## What You Should Understand After This Exercise

Rate limiting is not one-size-fits-all. Different endpoints have different sensitivity levels and traffic patterns. Authentication endpoints need strict limits (1-2 req/s) to prevent brute force. Expensive operations like report generation need very low rates to prevent resource exhaustion. General API endpoints need moderate limits with burst tolerance for normal usage spikes. The `burst` and `nodelay` parameters control how Nginx handles traffic that exceeds the rate -- `nodelay` processes burst requests immediately rather than queuing them, which provides a better user experience while still enforcing the overall rate.
