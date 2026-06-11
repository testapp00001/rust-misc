# Module 63: Load Balancing Algorithms — Round-Robin, Least-Connections, IP Hash

> **Previous Module**: [62 - DNS Deep Dive](../62-dns-deep-dive/README.md)
> **Next Module**: [64 - CDN & Edge](../64-cdn-and-edge/README.md)
> **Phase**: 9 — Networking

---

## The Problem

You have multiple servers behind a load balancer, but traffic isn't distributed evenly. One server is overwhelmed with 500 active connections while others have 50. Users report slow responses because they're hitting an overloaded server. Or worse — users lose their shopping carts because requests bounce between servers that don't share session state.

**Real-World Scenarios:**
- Round-robin sends equal traffic to a server with 2GB RAM and one with 16GB RAM
- Long-polling connections accumulate on one server while others sit idle
- Users get logged out randomly because requests hit different backend servers
- A slow database query on one server blocks all requests routed to it
- Cache hit rates drop because requests are scattered across all servers

---

## The Naive Way

```nginx
# Nginx default — simple round-robin
upstream backend {
    server 10.0.0.1:8080;
    server 10.0.0.2:8080;
    server 10.0.0.3:8080;
}

# Problems:
# 1. No consideration of server capacity
# 2. No health checks
# 3. No session persistence
# 4. No connection limits
# 5. Slow server gets same traffic as fast server
```

**Why This Fails:**
- Doesn't account for server capacity differences
- Doesn't consider current load (connections, CPU, memory)
- Can't handle long-running requests (WebSocket, streaming)
- No sticky sessions for stateful applications
- One slow server degrades overall performance

---

## The Right Way

### Load Balancing Algorithms Compared

```
┌─────────────────────────────────────────────────────────────────┐
│                    LOAD BALANCING ALGORITHMS                     │
├─────────────────────┬───────────────────────────────────────────┤
│ Algorithm           │ Best For                                  │
├─────────────────────┼───────────────────────────────────────────┤
│ Round Robin         │ Uniform servers, stateless apps           │
│ Weighted Round Robin│ Heterogeneous servers                     │
│ Least Connections   │ Varying request durations                 │
│ IP Hash             │ Session persistence needed                │
│ Random              │ Simple, good with large server pools      │
│ Consistent Hashing  │ Caching, minimal redistribution           │
│ Resource Based      │ CPU/memory constrained workloads          │
└─────────────────────┴───────────────────────────────────────────┘
```

### 1. Round Robin

```nginx
# Simple round-robin — rotates through servers sequentially
upstream backend_rr {
    server 10.0.0.1:8080;
    server 10.0.0.2:8080;
    server 10.0.0.3:8080;
    # Request 1 → server 1
    # Request 2 → server 2
    # Request 3 → server 3
    # Request 4 → server 1 (cycle repeats)
}

# Pros: Simple, fair distribution
# Cons: Ignores server capacity and current load
# Use when: All servers are identical and requests are uniform
```

### 2. Weighted Round Robin

```nginx
# Weighted — more powerful servers get more traffic
upstream backend_wrr {
    server 10.0.0.1:8080 weight=5;  # Gets 5/9 of traffic
    server 10.0.0.2:8080 weight=3;  # Gets 3/9 of traffic
    server 10.0.0.3:8080 weight=1;  # Gets 1/9 of traffic
    # Useful when servers have different capacities
}

# Real-world example:
# server 1: 16GB RAM, 8 CPU → weight=5
# server 2: 8GB RAM, 4 CPU → weight=3
# server 3: 4GB RAM, 2 CPU → weight=1
```

### 3. Least Connections

```nginx
# Routes to server with fewest active connections
upstream backend_lc {
    least_conn;
    server 10.0.0.1:8080;
    server 10.0.0.2:8080;
    server 10.0.0.3:8080;
    # If server 1 has 10 connections, server 2 has 5, server 3 has 8
    # Next request → server 2 (fewest connections)
}

# Pros: Adapts to varying request durations
# Cons: Doesn't consider request complexity
# Use when: Requests have varying processing times
# Example: API with both fast reads and slow writes

# Weighted least connections
upstream backend_wlc {
    least_conn;
    server 10.0.0.1:8080 weight=5;
    server 10.0.0.2:8080 weight=3;
    server 10.0.0.3:8080 weight=1;
}
```

### 4. IP Hash (Sticky Sessions)

```nginx
# Routes based on client IP — same client always hits same server
upstream backend_iphash {
    ip_hash;
    server 10.0.0.1:8080;
    server 10.0.0.2:8080;
    server 10.0.0.3:8080;
    # Client 192.168.1.100 → always server 1
    # Client 192.168.1.101 → always server 2
}

# Pros: Session persistence without shared state
# Cons: Can cause uneven distribution
# Use when: Session state is stored locally on servers
# Example: Shopping cart stored in server memory

# Caution: NAT can cause many users to appear as one IP
# Corporate networks may route all users through one IP
# Mobile users change IPs frequently
```

### 5. Consistent Hashing

```nginx
# Nginx Plus supports consistent hashing
upstream backend_ch {
    hash $request_uri consistent;
    server 10.0.0.1:8080;
    server 10.0.0.2:8080;
    server 10.0.0.3:8080;
    # Same URI always goes to same server
    # Adding/removing server only redistributes minimal keys
}

# Use case: Caching proxy
# Cache hit rate improves because same content always goes to same server
# When server is added, only 1/N keys are redistributed
```

### Health Checks and Failover

```nginx
# Nginx Plus health checks
upstream backend {
    zone backend_zone 64k;
    least_conn;

    server 10.0.0.1:8080 max_fails=3 fail_timeout=30s;
    server 10.0.0.2:8080 max_fails=3 fail_timeout=30s;
    server 10.0.0.3:8080 max_fails=3 fail_timeout=30s;

    # Active health checks (Nginx Plus)
    health_check interval=5s fails=3 passes=2 uri=/health;
}

# Open source Nginx — passive health checks only
upstream backend_passive {
    server 10.0.0.1:8080 max_fails=3 fail_timeout=30s;
    # If 3 consecutive failures within 30s, mark server down
    # After 30s, try again
}
```

### Connection Draining

```nginx
# Gracefully remove server from rotation
upstream backend {
    server 10.0.0.1:8080;
    server 10.0.0.2:8080;
    server 10.0.0.3:8080 drain;  # Stop new connections, allow existing
}

# HAProxy equivalent
backend servers
    server web1 10.0.0.1:8080 check
    server web2 10.0.0.2:8080 check
    server web3 10.0.0.3:8080 check state drain

# Use case: Deploying new version
# 1. Mark old server as "drain"
# 2. Wait for existing connections to complete
# 3. Deploy new version
# 4. Add new server back to rotation
# 5. Repeat for next server
```

---

## The Production Way

### L4 vs L7 Load Balancing

```
┌─────────────────────────────────────────────────────────────────┐
│                      L4 vs L7 LOAD BALANCING                     │
├─────────────────────┬───────────────────────────────────────────┤
│ Feature             │ L4 (Transport)    │ L7 (Application)      │
├─────────────────────┼───────────────────┼───────────────────────┤
│ Layer               │ TCP/UDP           │ HTTP/HTTPS            │
│ Routing based on    │ IP + Port         │ URL, Headers, Cookies │
│ SSL termination     │ No (passthrough)  │ Yes                   │
│ Content inspection  │ No                │ Yes                   │
│ Speed               │ Faster            │ Slower (more parsing) │
│ Use case            │ Database, gaming  │ Web apps, APIs        │
│ Examples            │ NLB, HAProxy TCP  │ ALB, Nginx, Envoy     │
└─────────────────────┴───────────────────┴───────────────────────┘
```

```bash
# L4 Load Balancer (HAProxy) — TCP mode
frontend tcp_frontend
    bind *:3306
    mode tcp
    default_backend mysql_servers

backend mysql_servers
    mode tcp
    balance leastconn
    server db1 10.0.0.1:3306 check
    server db2 10.0.0.2:3306 check

# L7 Load Balancer (Nginx) — HTTP mode
upstream web_servers {
    least_conn;
    server 10.0.0.1:8080;
    server 10.0.0.2:8080;
}

server {
    listen 80;
    location /api/ {
        proxy_pass http://api_servers;
    }
    location /static/ {
        proxy_pass http://cdn_servers;
    }
}
```

### HAProxy Configuration

```bash
# /etc/haproxy/haproxy.cfg

global
    maxconn 50000
    log /dev/log local0
    stats socket /var/run/haproxy.sock mode 660 level admin

defaults
    mode http
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    option httplog
    option dontlognull
    option http-server-close
    option forwardfor

# Statistics page
listen stats
    bind *:8404
    stats enable
    stats uri /stats
    stats refresh 10s
    stats admin if TRUE

# Frontend — accepts connections
frontend http_front
    bind *:80
    bind *:443 ssl crt /etc/ssl/certs/site.pem

    # Route based on URL path
    acl is_api path_beg /api
    acl is_static path_beg /static

    use_backend api_servers if is_api
    use_backend static_servers if is_static
    default_backend web_servers

# Backend — web servers (least connections)
backend web_servers
    balance leastconn
    option httpchk GET /health
    http-check expect status 200

    server web1 10.0.0.1:8080 check inter 5s fall 3 rise 2 weight 100
    server web2 10.0.0.2:8080 check inter 5s fall 3 rise 2 weight 100
    server web3 10.0.0.3:8080 check inter 5s fall 3 rise 2 weight 50

# Backend — API servers (IP hash for session persistence)
backend api_servers
    balance source
    hash-type consistent
    option httpchk GET /api/health

    server api1 10.0.0.11:8080 check
    server api2 10.0.0.12:8080 check

# Backend — static servers (round robin)
backend static_servers
    balance roundrobin
    server static1 10.0.0.21:80 check
    server static2 10.0.0.22:80 check
```

### Envoy Proxy Configuration

```yaml
# Envoy — Modern, cloud-native L4/L7 proxy
static_resources:
  listeners:
    - name: listener_0
      address:
        socket_address:
          address: 0.0.0.0
          port_value: 80
      filter_chains:
        - filters:
            - name: envoy.filters.network.http_connection_manager
              typed_config:
                "@type": type.googleapis.com/envoy.extensions.filters.network.http_connection_manager.v3.HttpConnectionManager
                stat_prefix: ingress_http
                route_config:
                  name: local_route
                  virtual_hosts:
                    - name: backend
                      domains: ["*"]
                      routes:
                        - match:
                            prefix: "/api"
                          route:
                            cluster: api_service
                        - match:
                            prefix: "/"
                          route:
                            cluster: web_service
                http_filters:
                  - name: envoy.filters.http.router

  clusters:
    - name: web_service
      type: EDS
      lb_policy: LEAST_REQUEST
      eds_cluster_config:
        service_name: web_service
        eds_config:
          resource_api_version: V3
          api_config_source:
            api_type: GRPC
            grpc_services:
              envoy_grpc:
                cluster_name: xds_cluster
      health_checks:
        - timeout: 1s
          interval: 5s
          unhealthy_threshold: 3
          healthy_threshold: 2
          http_health_check:
            path: "/health"

    - name: api_service
      type: EDS
      lb_policy: RING_HASH
      eds_cluster_config:
        service_name: api_service
        eds_config:
          resource_api_version: V3
          api_config_source:
            api_type: GRPC
            grpc_services:
              envoy_grpc:
                cluster_name: xds_cluster
```

### Circuit Breaker Pattern

```yaml
# Envoy circuit breaker
clusters:
  - name: api_service
    circuit_breakers:
      thresholds:
        - priority: DEFAULT
          max_connections: 1000
          max_pending_requests: 100
          max_requests: 1000
          max_retries: 3
          retry_budget:
            budget_percent:
              value: 20.0
            min_retry_concurrency: 3

# How circuit breaker works:
# 1. Normal operation: requests flow through
# 2. Error threshold exceeded: circuit OPENS
#    - All requests immediately fail (fast fail)
#    - No load on struggling service
# 3. After timeout: circuit goes to HALF-OPEN
#    - Allow limited requests through
# 4. If requests succeed: circuit CLOSES
#    - Normal operation resumes
# 5. If requests fail: circuit OPENS again
```

### Retry and Timeout Configuration

```yaml
# Envoy retry policy
routes:
  - match:
      prefix: "/api"
    route:
      cluster: api_service
      timeout: 10s
      retry_policy:
        retry_on: "5xx,reset,connect-failure,refused-stream"
        num_retries: 3
        per_try_timeout: 3s
        retry_back_off:
          base_interval: 0.25s
          max_interval: 2s

# HAProxy retry configuration
backend api_servers
    option redispatch
    retries 3
    timeout connect 5s
    timeout server 10s
    timeout check 3s
```

---

## Hands-On Lab

### Lab: Compare Load Balancing Algorithms with HAProxy

**Objective:** Set up HAProxy with multiple algorithms and observe traffic distribution under different load patterns.

**Prerequisites:**
- Docker and Docker Compose
- 4 terminal windows
- `curl` and `jq` installed

#### Step 1: Create Test Environment

```yaml
# docker-compose.yml
version: '3.8'

services:
  haproxy:
    image: haproxy:2.8
    ports:
      - "8080:80"
      - "8404:8404"
    volumes:
      - ./haproxy.cfg:/usr/local/etc/haproxy/haproxy.cfg:ro
    depends_on:
      - web1
      - web2
      - web3

  web1:
    image: nginx:alpine
    volumes:
      - ./server.sh:/docker-entrypoint.d/server.sh
    environment:
      - SERVER_ID=1
      - PROCESSING_TIME=0.1

  web2:
    image: nginx:alpine
    volumes:
      - ./server.sh:/docker-entrypoint.d/server.sh
    environment:
      - SERVER_ID=2
      - PROCESSING_TIME=0.5

  web3:
    image: nginx:alpine
    volumes:
      - ./server.sh:/docker-entrypoint.d/server.sh
    environment:
      - SERVER_ID=3
      - PROCESSING_TIME=1.0
```

```bash
# server.sh — Simple echo server
#!/bin/sh
cat > /etc/nginx/conf.d/default.conf << EOF
server {
    listen 80;
    location / {
        default_type application/json;
        return 200 '{"server": "$SERVER_ID", "processing_time": $PROCESSING_TIME}';
    }
    location /health {
        return 200 'OK';
    }
}
EOF
nginx -s reload
```

#### Step 2: Test Round Robin

```bash
# haproxy.cfg — Round Robin
global
    log stdout format raw local0

defaults
    mode http
    log global
    timeout connect 5s
    timeout client 30s
    timeout server 30s

listen stats
    bind *:8404
    stats enable
    stats uri /stats

frontend http_front
    bind *:80
    default_backend servers_rr

backend servers_rr
    balance roundrobin
    server web1 web1:80 check
    server web2 web2:80 check
    server web3 web3:80 check
```

```bash
# Start environment
docker-compose up -d

# Send 100 requests and count distribution
for i in {1..100}; do
  curl -s http://localhost:8080 | jq -r '.server'
done | sort | uniq -c

# Expected output (roughly equal):
# 34 1
# 33 2
# 33 3
```

#### Step 3: Test Least Connections

```bash
# Update haproxy.cfg
cat > haproxy.cfg << 'EOF'
global
    log stdout format raw local0

defaults
    mode http
    log global
    timeout connect 5s
    timeout client 30s
    timeout server 30s

listen stats
    bind *:8404
    stats enable
    stats uri /stats

frontend http_front
    bind *:80
    default_backend servers_lc

backend servers_lc
    balance leastconn
    server web1 web1:80 check
    server web2 web2:80 check
    server web3 web3:80 check
EOF

# Restart HAProxy
docker-compose restart haproxy

# Test with long-running requests
# Terminal 1: Start long-running requests
for i in {1..10}; do
  curl -s http://localhost:8080 &
done

# Terminal 2: Send quick requests
for i in {1..50}; do
  curl -s http://localhost:8080 | jq -r '.server' &
done | sort | uniq -c

# With least connections, quick requests should go to
# servers with fewer active long-running connections
```

#### Step 4: Test IP Hash (Sticky Sessions)

```bash
# Update haproxy.cfg
cat > haproxy.cfg << 'EOF'
global
    log stdout format raw local0

defaults
    mode http
    log global
    timeout connect 5s
    timeout client 30s
    timeout server 30s

listen stats
    bind *:8404
    stats enable
    stats uri /stats

frontend http_front
    bind *:80
    default_backend servers_hash

backend servers_hash
    balance source
    hash-type consistent
    server web1 web1:80 check
    server web2 web2:80 check
    server web3 web3:80 check
EOF

docker-compose restart haproxy

# Test sticky sessions
echo "Client 1 (192.168.1.100):"
for i in {1..5}; do
  curl -s -H "X-Forwarded-For: 192.168.1.100" http://localhost:8080 | jq -r '.server'
done

echo "Client 2 (192.168.1.101):"
for i in {1..5}; do
  curl -s -H "X-Forwarded-For: 192.168.1.101" http://localhost:8080 | jq -r '.server'
done

# Same client should always hit same server
```

#### Step 5: Monitor with HAProxy Stats

```bash
# Open in browser: http://localhost:8404/stats

# Key metrics to observe:
# - Session rate (current, max)
# - Session total (current, max)
# - Bytes in/out
# - Denied/Errors
# - Server status (UP/DOWN)

# Or use CLI
echo "show stat" | socat /var/run/haproxy.sock stdio | \
  cut -d ',' -f 1,2,5,8,18 | column -s, -t

# Watch in real-time
watch -n 1 'echo "show stat" | socat /var/run/haproxy.sock stdio | \
  cut -d "," -f 1,2,5,8,18 | column -s, -t'
```

#### Step 6: Test Health Checks and Failover

```bash
# Simulate server failure
docker-compose stop web2

# Watch HAProxy detect failure
watch -n 1 'echo "show stat" | socat /var/run/haproxy.sock stdio | \
  grep -E "^servers" | cut -d "," -f 1,2,18 | column -s, -t'

# Send requests — should only go to web1 and web3
for i in {1..20}; do
  curl -s http://localhost:8080 | jq -r '.server'
done | sort | uniq -c

# Restore server
docker-compose start web2

# Watch HAProxy detect recovery
watch -n 1 'echo "show stat" | socat /var/run/haproxy.sock stdio | \
  grep -E "^servers" | cut -d "," -f 1,2,18 | column -s, -t'
```

#### Step 7: Benchmark Different Algorithms

```bash
# Install Apache Bench
sudo apt-get install apache2-utils

# Benchmark round-robin
cat > haproxy-rr.cfg << 'EOF'
# ... round-robin config ...
EOF
docker-compose restart haproxy
ab -n 10000 -c 100 http://localhost:8080/ | grep "Requests per second"

# Benchmark least-connections
cat > haproxy-lc.cfg << 'EOF'
# ... least-connections config ...
EOF
docker-compose restart haproxy
ab -n 10000 -c 100 http://localhost:8080/ | grep "Requests per second"

# Compare results
# Round-robin: X requests/sec
# Least-connections: Y requests/sec
# The difference becomes significant with varying request durations
```

---

## Limitation → Next Topic

**What You Learned:**
- Load balancing algorithms and when to use each
- L4 vs L7 load balancing trade-offs
- HAProxy and Envoy configuration
- Health checks and failover
- Connection draining for zero-downtime deployments
- Circuit breaker pattern for resilience
- Retry and timeout strategies

**What's Still Hard:**
You can distribute traffic across servers, but all servers are in the same region. Users on the other side of the world still experience high latency. Your static assets (images, CSS, JS) are served from the same origin servers, adding load and increasing response times. You need to bring content closer to users.

**Next Module:** [64 - CDN & Edge](../64-cdn-and-edge/README.md) — Learn how CDNs cache content at edge locations worldwide, reducing latency and offloading your origin servers. Explore edge computing for dynamic content at the edge.

---

## Quick Reference

### Algorithm Selection Guide

| Scenario | Algorithm | Why |
|----------|-----------|-----|
| Uniform servers, stateless | Round Robin | Simple, fair |
| Different server capacities | Weighted Round Robin | Proportional load |
| Varying request times | Least Connections | Adapts to load |
| Session persistence needed | IP Hash | Same client → same server |
| Caching layer | Consistent Hashing | Minimal redistribution |
| Mixed workloads | L7 routing | Path-based routing |

### HAProxy Commands

```bash
# Show all servers
echo "show servers state" | socat /var/run/haproxy.sock stdio

# Disable server
echo "set server backend/web1 state drain" | socat /var/run/haproxy.sock stdio

# Enable server
echo "set server backend/web1 state ready" | socat /var/run/haproxy.sock stdio

# Show stats
echo "show stat" | socat /var/run/haproxy.sock stdio

# Reload config
haproxy -f /etc/haproxy/haproxy.cfg -p /var/run/haproxy.pid -sf $(cat /var/run/haproxy.pid)
```

### Circuit Breaker States

```
CLOSED → (errors exceed threshold) → OPEN
OPEN → (timeout expires) → HALF-OPEN
HALF-OPEN → (requests succeed) → CLOSED
HALF-OPEN → (requests fail) → OPEN
```

---

**Remember:** The best load balancing algorithm depends on your specific workload. Test with realistic traffic patterns before choosing. What works for a simple web app may fail for a real-time API with long-polling connections.
