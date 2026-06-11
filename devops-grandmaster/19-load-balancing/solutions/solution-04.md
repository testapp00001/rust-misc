# Solution 04: Compare Algorithms Under Load

## Complete Answer

### Backend setup with latency variance (docker-compose.yml excerpt)

```yaml
services:
  backend-1:
    image: python:3.12-slim
    command: >
      python -c "
      from http.server import HTTPServer, BaseHTTPRequestHandler
      import time, os
      class H(BaseHTTPRequestHandler):
          def do_GET(self):
              time.sleep(float(os.environ.get('DELAY', '0')))
              self.send_response(200)
              self.end_headers()
              self.wfile.write(b'backend-1')
          def log_message(self, *a): pass
      HTTPServer(('0.0.0.0', 5678), H).serve_forever()
      "
    environment:
      DELAY: "0"

  backend-2:
    image: python:3.12-slim
    command: >
      python -c "
      from http.server import HTTPServer, BaseHTTPRequestHandler
      import time, os
      class H(BaseHTTPRequestHandler):
          def do_GET(self):
              time.sleep(float(os.environ.get('DELAY', '0')))
              self.send_response(200)
              self.end_headers()
              self.wfile.write(b'backend-2')
          def log_message(self, *a): pass
      HTTPServer(('0.0.0.0', 5678), H).serve_forever()
      "
    environment:
      DELAY: "0.05"

  backend-3:
    image: python:3.12-slim
    command: >
      python -c "
      from http.server import HTTPServer, BaseHTTPRequestHandler
      import time, os
      class H(BaseHTTPRequestHandler):
          def do_GET(self):
              time.sleep(float(os.environ.get('DELAY', '0')))
              self.send_response(200)
              self.end_headers()
              self.wfile.write(b'backend-3')
          def log_message(self, *a): pass
      HTTPServer(('0.0.0.0', 5678), H).serve_forever()
      "
    environment:
      DELAY: "0.2"

  load-balancer:
    image: nginx:1.25-alpine
    ports:
      - "8080:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - backend-1
      - backend-2
      - backend-3
```

### nginx.conf variants

**Round Robin** (default -- no directive):
```nginx
events { worker_connections 1024; }
http {
    upstream backend_pool {
        server backend-1:5678;
        server backend-2:5678;
        server backend-3:5678;
    }
    server {
        listen 80;
        location / { proxy_pass http://backend_pool; }
    }
}
```

**Least Connections**:
```nginx
upstream backend_pool {
    least_conn;
    server backend-1:5678;
    server backend-2:5678;
    server backend-3:5678;
}
```

**IP Hash**:
```nginx
upstream backend_pool {
    ip_hash;
    server backend-1:5678;
    server backend-2:5678;
    server backend-3:5678;
}
```

### Load test commands

```bash
# Round Robin (no delay)
hey -n 1000 -c 50 http://localhost:8080 > results-round-robin.txt 2>&1

# Least Connections (no delay)
hey -n 1000 -c 50 http://localhost:8080 > results-least-connections.txt 2>&1

# IP Hash (no delay)
hey -n 1000 -c 50 http://localhost:8080 > results-ip-hash.txt 2>&1

# Round Robin (with delay)
hey -n 1000 -c 50 http://localhost:8080 > results-round-robin-delayed.txt 2>&1

# Least Connections (with delay)
hey -n 1000 -c 50 http://localhost:8080 > results-least-connections-delayed.txt 2>&1

# IP Hash (with delay)
hey -n 1000 -c 50 http://localhost:8080 > results-ip-hash-delayed.txt 2>&1
```

### Typical results (representative -- your numbers will vary)

#### Without latency variance

| Metric | Round Robin | Least Conn | IP Hash |
|--------|------------|------------|---------|
| Avg latency | 12ms | 12ms | 12ms |
| p99 latency | 28ms | 27ms | 30ms |
| Throughput | 4,100 req/s | 4,050 req/s | 4,000 req/s |
| Distribution | 334/333/333 | 335/332/333 | ~333/333/334 |

All three algorithms perform nearly identically because all backends respond
at the same speed. The slight variations are noise.

#### With latency variance (backend-1: 0ms, backend-2: 50ms, backend-3: 200ms)

| Metric | Round Robin | Least Conn | IP Hash |
|--------|------------|------------|---------|
| Avg latency | ~83ms | ~25ms | ~83ms |
| p99 latency | ~210ms | ~55ms | ~215ms |
| Throughput | ~600 req/s | ~1,800 req/s | ~580 req/s |
| Distribution | 334/333/333 | ~600/250/150 | ~333/333/334 |

### Analysis

#### 1. Comparison table

See tables above. The key observation is that without latency variance, all
algorithms are equivalent. With latency variance, Least Connections dominates.

#### 2. Best without latency variance: Round Robin (by a hair)

Without latency variance, all algorithms perform similarly. Round Robin is
marginally best because it has the least overhead -- no connection counting, no
hash computation. The difference is negligible in practice.

#### 3. Best with latency variance: Least Connections

Least Connections delivers dramatically better performance when backends vary
in speed. Here is why:

- **Round Robin** sends one-third of requests to each backend. The fast
  backend (0ms) finishes immediately but has to wait for its next turn. The
  slow backend (200ms) builds up a queue. The load balancer's concurrency is
  effectively limited by the slowest backend: with 50 concurrent connections
  and 3 backends, ~17 connections are always stuck waiting on the 200ms
  backend.

- **Least Connections** dynamically adapts. The fast backend (0ms) finishes
  requests instantly, its connection count drops to 0, and it immediately
  gets the next request. The slow backend (200ms) holds connections longer,
  so its connection count stays high, and it receives fewer new requests.
  Over time, the fast backend handles the majority of traffic.

- **IP Hash** has the same problem as Round Robin: it is deterministic and
  ignores backend speed. Worse, if the client population is small, one
  backend may get all the traffic by hash luck.

#### 4. Recommendation for production with heterogeneous backends

**Least Connections** is the clear winner for production services with
heterogeneous response times. It:

- Automatically adapts to backend speed differences without manual tuning.
- Prevents slow backends from accumulating a backlog.
- Produces the lowest average and tail latency.
- Requires no client-side information (unlike IP Hash).

If backends also have different capacities (CPU/RAM), use **Weighted Least
Connections** (available in HAProxy, not directly in open-source Nginx) to
combine capacity awareness with dynamic load adaptation.

## Common Mistakes

### 1. Running only one load test per algorithm

Load tests have inherent variance. Run each test 3 times and take the median.
A single run can be misleading due to system noise, GC pauses, or Docker
startup effects.

### 2. Using too few requests

With only 100 requests and 3 backends, the distribution will not stabilise.
Use at least 1,000 requests (preferably 10,000) for meaningful results.

### 3. Not warming up

The first few requests are slow because of connection establishment and DNS
resolution. Run a short warm-up burst (e.g., 100 requests) before the
measured run, or use `hey`'s `-disable-keepalive` flag carefully.

### 4. Forgetting to reload Nginx between tests

After changing `nginx.conf`, you must reload Nginx (`nginx -s reload`). If you
forget, you are still testing the previous algorithm. Verify with a manual
`curl` that the expected backend pattern appears.

### 5. Measuring only throughput, not latency

An algorithm can have high throughput but terrible tail latency. Always report
both average and p99 latency. A system that serves 10,000 req/s but has a p99
of 5 seconds is worse than one that serves 8,000 req/s with a p99 of 50ms.

### 6. Ignoring connection pooling effects

`hey` with `-c 50` opens 50 concurrent connections. With keep-alive (default),
these connections are reused. This means the first request on each connection
goes through the load balancer, but subsequent requests on the same connection
go to the same backend (because the TCP connection is already established).
This skews results. Use `-disable-keepalive` for a true per-request
distribution test, but be aware this increases latency due to TCP handshake
overhead.
