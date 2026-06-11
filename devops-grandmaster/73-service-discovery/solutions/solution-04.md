# Solution 04: Client-Side vs. Server-Side Discovery

## Part A: Client-Side Discovery Implementation

```python
import time
import random
import requests
from enum import Enum


class CircuitState(Enum):
    CLOSED = "closed"
    OPEN = "open"
    HALF_OPEN = "half_open"


class ClientSideDiscovery:
    def __init__(self, registry):
        self.registry = registry
        self._counters = {}          # service_name -> round-robin counter
        self._circuit_state = {}     # service_name -> CircuitState
        self._failure_count = {}     # service_name -> int
        self._last_failure = {}      # service_name -> float (timestamp)
        self.failure_threshold = 3
        self.recovery_timeout = 30

    def call_service(self, service_name, path, method="GET", **kwargs):
        """Call a service with round-robin load balancing and circuit breaker."""
        # Check circuit breaker
        if self._is_circuit_open(service_name):
            raise CircuitOpenError(
                f"Circuit is open for {service_name}. "
                f"Retry after {self.recovery_timeout}s."
            )

        # Discover healthy instances
        instances = self.registry.discover(service_name)
        healthy = [i for i in instances if i.get("healthy", True)]

        if not healthy:
            raise NoHealthyInstancesError(
                f"No healthy instances of {service_name}"
            )

        # Round-robin selection
        counter = self._counters.get(service_name, 0)
        instance = healthy[counter % len(healthy)]
        self._counters[service_name] = counter + 1

        # Make the request
        url = f"http://{instance['address']}:{instance['port']}{path}"
        try:
            response = requests.request(method, url, timeout=5, **kwargs)
            response.raise_for_status()
            self._on_success(service_name)
            return response
        except Exception as e:
            self._on_failure(service_name)
            raise

    def _is_circuit_open(self, service_name):
        state = self._circuit_state.get(service_name, CircuitState.CLOSED)

        if state == CircuitState.OPEN:
            elapsed = time.time() - self._last_failure.get(service_name, 0)
            if elapsed > self.recovery_timeout:
                self._circuit_state[service_name] = CircuitState.HALF_OPEN
                return False
            return True

        return False

    def _on_success(self, service_name):
        self._failure_count[service_name] = 0
        if self._circuit_state.get(service_name) == CircuitState.HALF_OPEN:
            self._circuit_state[service_name] = CircuitState.CLOSED

    def _on_failure(self, service_name):
        count = self._failure_count.get(service_name, 0) + 1
        self._failure_count[service_name] = count
        self._last_failure[service_name] = time.time()

        if count >= self.failure_threshold:
            self._circuit_state[service_name] = CircuitState.OPEN


class CircuitOpenError(Exception):
    pass


class NoHealthyInstancesError(Exception):
    pass
```

### Why This Works

- **Round-robin:** A simple counter modulo the number of instances distributes requests evenly. More sophisticated algorithms (weighted, least-connections) would track additional state.
- **Circuit breaker:** After `failure_threshold` consecutive failures, the circuit opens and all calls fail fast. After `recovery_timeout`, the circuit transitions to half-open and allows a test request. If it succeeds, the circuit closes.
- **Separation of concerns:** Discovery (finding instances) is separate from routing (picking an instance) and resilience (circuit breaking).

### Common Mistakes to Avoid

- Not resetting the failure count on success. The circuit breaker would trip on a single failure per window.
- Using `random.choice` instead of round-robin. Random selection can be uneven, especially with few instances.
- Not handling the `NoHealthyInstancesError` in the calling code. The caller must have a fallback (error response, cached data, default value).

---

## Part B: Server-Side Discovery with Consul Template

### Nginx Configuration Template

```nginx
# /etc/consul-template/nginx.conf.ctmpl

# API upstream
{{ range service "api" }}
upstream api_backend {
  {{ range service "api" }}
  server {{ .Address }}:{{ .Port }} weight=10 max_fails=3 fail_timeout=30s;
  {{ end }}
}
{{ end }}

# Product upstream
{{ range service "product" }}
upstream product_backend {
  {{ range service "product" }}
  server {{ .Address }}:{{ .Port }} weight=10 max_fails=3 fail_timeout=30s;
  {{ end }}
}
{{ end }}

# Cart upstream
{{ range service "cart" }}
upstream cart_backend {
  {{ range service "cart" }}
  server {{ .Address }}:{{ .Port }} weight=10 max_fails=3 fail_timeout=30s;
  {{ end }}
}
{{ end }}

# Payment upstream
{{ range service "payment" }}
upstream payment_backend {
  {{ range service "payment" }}
  server {{ .Address }}:{{ .Port }} weight=10 max_fails=3 fail_timeout=30s;
  {{ end }}
}
{{ end }}

server {
  listen 80;

  location /api/ {
    proxy_pass http://api_backend;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_connect_timeout 5s;
    proxy_read_timeout 30s;
  }

  location /products/ {
    proxy_pass http://product_backend;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_connect_timeout 5s;
    proxy_read_timeout 30s;
  }

  location /cart/ {
    proxy_pass http://cart_backend;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_connect_timeout 5s;
    proxy_read_timeout 30s;
  }

  location /payment/ {
    proxy_pass http://payment_backend;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_connect_timeout 5s;
    proxy_read_timeout 30s;
  }
}
```

### consul-template Command

```bash
consul-template \
  -template "/etc/consul-template/nginx.conf.ctmpl:/etc/nginx/conf.d/services.conf:nginx -s reload" \
  -consul-addr "localhost:8500" \
  -log-level "info"
```

This command:
1. Watches Consul for changes to the registered services.
2. Regenerates `/etc/nginx/conf.d/services.conf` whenever instances change.
3. Reloads Nginx (`nginx -s reload`) to pick up the new configuration.

### Common Mistakes to Avoid

- Hardcoding the outer `{{ range service "api" }}` check. If no instances exist, the upstream block is omitted entirely, which causes Nginx to fail. Use a default server or handle the empty case.
- Not reloading Nginx after config changes. The template generates the file, but Nginx does not watch for file changes.
- Using `nginx -s reload` instead of `nginx -t && nginx -s reload`. Always test the config before reloading.

---

## Part C: Trade-off Analysis

### 1. Latency

**Winner: Client-side discovery.** Client-side discovery connects directly to the target service with no intermediary. Server-side discovery adds a proxy hop (0.5-2ms for Nginx, 0.1-0.5ms for Envoy). For latency-sensitive paths (e.g., database queries, real-time APIs), the extra hop matters.

### 2. Failure Isolation

**Winner: Server-side discovery.** If the registry is unavailable:
- Client-side: the calling service cannot discover instances and must fail or use stale data.
- Server-side: the proxy continues to serve from the last known configuration. Nginx does not need Consul to be running to serve existing upstreams.

### 3. Language Support

**Winner: Server-side discovery.** Client-side discovery requires a discovery client library in every language. If you have services in Python, Go, Java, and Node.js, you need 4 client libraries to maintain. Server-side discovery works with any language -- the client just makes an HTTP request to the proxy.

### 4. Load Balancing Sophistication

**Winner: Client-side discovery.** The client can implement any algorithm: round-robin, weighted, least-connections, latency-based, affinity-based. It can also implement application-aware routing (e.g., prefer instances with warm caches). Server-side discovery is limited to what the proxy supports (Nginx: round-robin, least-connections, IP hash).

### 5. Operational Complexity

**Winner: Client-side discovery.** There is no proxy cluster to operate, no Nginx config to manage, no reload logic to debug. However, this advantage disappears if you need a proxy for other reasons (TLS termination, rate limiting, authentication).

### Common Mistakes to Avoid

- Declaring a universal winner. Each approach wins on different dimensions.
- Not mentioning the hybrid approach (service mesh). Consul Connect and Istio provide server-side routing with client-side intelligence, combining the best of both.

---

## Key Takeaway

Client-side discovery offers maximum control and minimum latency but couples discovery logic to every service and requires client libraries for every language. Server-side discovery centralizes routing and works with any language but adds a proxy hop and operational overhead. In practice, many production systems use a service mesh that combines both: server-side proxies with client-side intelligence (circuit breaking, retries, load balancing).
