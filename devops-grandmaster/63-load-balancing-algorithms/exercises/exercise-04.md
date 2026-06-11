# Exercise 04: Production Load Balancer Design

**Type:** Challenge
**Time:** 45 min
**Difficulty:** Medium-Hard

## Objective

Design a production-grade load balancing configuration that handles health checks, SSL termination, connection draining, and multiple algorithm selection for different traffic types.

## Scenario

You are configuring an Nginx load balancer for a production environment with these requirements:

```
Backend Pool:
├── api-server-1: 10.0.1.1:8080 (healthy, 50 active connections)
├── api-server-2: 10.0.1.2:8080 (healthy, 120 active connections)
├── api-server-3: 10.0.1.3:8080 (healthy, 30 active connections)
└── api-server-4: 10.0.1.4:8080 (unhealthy, 200 active connections)

Requirements:
- SSL termination at the load balancer
- Health checks every 5 seconds
- Automatic removal of unhealthy backends
- Connection draining on backend removal
- Different algorithms for /api/* vs /ws/* endpoints
- Rate limiting: 100 req/s per client IP
```

## Tasks

### Part A: Nginx Configuration

Write the Nginx configuration for this load balancer. Include:

1. Upstream block with health checks
2. SSL termination
3. Location blocks for /api/* and /ws/*
4. Rate limiting

```nginx
# Your configuration here
```

<details>
<summary>Hint</summary>

Use `upstream` blocks to define backend pools. For health checks, use `max_fails` and `fail_timeout`. For WebSocket support, use `proxy_set_header Upgrade` and `proxy_http_version 1.1`. Rate limiting uses `limit_req_zone` and `limit_req`.

</details>

### Part B: Health Check Design

Design a health check configuration that:
1. Checks backends every 5 seconds
2. Requires 3 consecutive failures to mark a backend as unhealthy
3. Requires 2 consecutive successes to mark a backend as healthy again
4. Distinguishes between "ready to serve" and "alive but not ready"

Write the health check endpoint that each backend should implement.

<details>
<summary>Hint</summary>

Nginx open-source uses passive health checks (`max_fails`, `fail_timeout`). Nginx Plus adds active health checks. The health endpoint should distinguish between liveness (process is running) and readiness (process can handle requests). Kubernetes uses this same pattern with liveness and readiness probes.

</details>

### Part C: Connection Draining

Explain the connection draining process:

1. What happens to in-flight requests when a backend is marked unhealthy?
2. How do you configure Nginx to drain connections before removing a backend?
3. What is the difference between `down` and removing a backend from the upstream block?

<details>
<summary>Hint</summary>

Connection draining allows in-flight requests to complete while preventing new requests from being sent to the backend. In Nginx, `fail_timeout` controls how long a failed server is removed. The `down` directive permanently removes a server, while failure-based removal is temporary.

</details>

### Part D: Rate Limiting

Configure rate limiting that:
1. Allows 100 requests per second per client IP
2. Allows bursts of up to 200 requests
3. Returns HTTP 429 when the limit is exceeded
4. Exempts health check endpoints from rate limiting

<details>
<summary>Hint</summary>

Use `limit_req_zone $binary_remote_addr zone=api:10m rate=100r/s` to define the zone, and `limit_req zone=api burst=200 nodelay` to apply it. Use a separate location for health checks without the rate limit directive.

</details>

## Success Criteria

- [ ] You can write a complete Nginx load balancer configuration
- [ ] You can design health check endpoints and thresholds
- [ ] You understand connection draining and how to configure it
- [ ] You can implement rate limiting with exemptions
- [ ] Your configuration handles the unhealthy backend (api-server-4)

## What You Should Understand After This Exercise

A production load balancer is far more than a traffic distributor. It handles SSL termination (offloading crypto from backends), health monitoring (detecting and removing failed backends), connection draining (graceful removal), and rate limiting (protecting backends from overload). The load balancer is a critical control point in the request path.
