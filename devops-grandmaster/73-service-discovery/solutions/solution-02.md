# Solution 02: Consul Service Registration

## Part A: Service Definitions

### api.json

```json
{
  "service": {
    "id": "api-1",
    "name": "api",
    "port": 8080,
    "tags": ["v2", "production"],
    "meta": {
      "version": "2.0.0",
      "environment": "production"
    },
    "check": {
      "http": "http://localhost:8080/health",
      "interval": "10s",
      "timeout": "3s",
      "deregister_critical_service_after": "30m"
    }
  }
}
```

### payment.json

```json
{
  "service": {
    "id": "payment-1",
    "name": "payment",
    "port": 9090,
    "tags": ["v1", "production"],
    "meta": {
      "version": "1.0.0",
      "environment": "production"
    },
    "check": {
      "http": "http://localhost:9090/health",
      "interval": "10s",
      "timeout": "3s",
      "deregister_critical_service_after": "30m"
    }
  }
}
```

### notification.json

```json
{
  "service": {
    "id": "notification-1",
    "name": "notification",
    "port": 3000,
    "tags": ["v1", "staging"],
    "meta": {
      "version": "1.0.0",
      "environment": "staging"
    },
    "check": {
      "http": "http://localhost:3000/health",
      "interval": "10s",
      "timeout": "3s",
      "deregister_critical_service_after": "30m"
    }
  }
}
```

### Why This Works

- **`id`** must be unique per agent. Use `<service>-<instance>` format for multi-instance services.
- **`tags`** enable filtering. You can query `production.api.service.consul` to find only production instances.
- **`meta`** provides additional metadata that is returned in queries but not used for filtering.
- **`deregister_critical_service_after`** automatically removes the service if its health check fails for 30 minutes. This prevents stale registrations.
- **Health checks** run on the Consul agent (client), not the server. The agent reports health to the servers.

### Common Mistakes to Avoid

- Using the service name as the ID. If you run multiple instances, they will conflict. Always use a unique ID.
- Forgetting `deregister_critical_service_after`. Without it, a crashed service stays in the catalog forever with a "critical" status.
- Setting the health check interval too short (< 5s). This generates excessive load on the service and the Consul cluster.

---

## Part B: Service Registration via HTTP API

### 1. Register the api service

```bash
curl -X PUT http://localhost:8500/v1/agent/service/register \
  -d '{
    "ID": "api-1",
    "Name": "api",
    "Port": 8080,
    "Tags": ["v2", "production"],
    "Meta": {
      "version": "2.0.0",
      "environment": "production"
    },
    "Check": {
      "HTTP": "http://localhost:8080/health",
      "Interval": "10s",
      "Timeout": "3s",
      "DeregisterCriticalServiceAfter": "30m"
    }
  }'
```

### 2. Deregister the api service

```bash
curl -X PUT http://localhost:8500/v1/agent/service/deregister/api-1
```

### 3. List all healthy instances of payment

```bash
curl "http://localhost:8500/v1/health/service/payment?passing=true"
```

### 4. Blocking queries for notification changes

```bash
# First request (no index) returns current state with X-Consul-Index header
curl -v "http://localhost:8500/v1/health/service/notification?passing=true"
# Note the X-Consul-Index value, e.g., 1234

# Subsequent request blocks until the index changes
curl "http://localhost:8500/v1/health/service/notification?passing=true&index=1234&wait=5m"
# This returns immediately if the index changes, or after 5 minutes if it doesn't
```

### Common Mistakes to Avoid

- Using `GET` instead of `PUT` for registration. Consul's agent API uses `PUT` for mutations.
- Not URL-encoding the service ID in the deregister command if it contains special characters.
- Forgetting `?passing=true` on health queries. Without it, you get all instances including those with failed health checks.

---

## Part C: DNS-Based Discovery

### 1. Resolve all instances of api

```bash
dig @127.0.0.1 -p 8600 api.service.consul
```

This returns A records with the IP addresses of all healthy `api` instances.

### 2. Get SRV records for payment

```bash
dig @127.0.0.1 -p 8600 payment.service.consul SRV
```

SRV records include the port and node name in addition to the IP address:

```
;; ANSWER SECTION:
payment.service.consul. 0  IN  SRV  1 1 9090 node1.node.dc1.consul.
;; ADDITIONAL SECTION:
node1.node.dc1.consul.  0  IN  A    192.168.1.20
```

### 3. Filter by tag: production instances of notification

```bash
dig @127.0.0.1 -p 8600 production.notification.service.consul
```

Consul DNS supports tag-based filtering using the format `<tag>.<service>.service.consul`.

### Common Mistakes to Avoid

- Using the default DNS port (53) instead of Consul's port (8600). Consul runs its own DNS server.
- Not realizing that Consul DNS only returns healthy instances (by default). This is a key advantage over raw DNS.
- Forgetting that DNS TTL can cause stale results. Consul sets TTL to 0 by default, but intermediate DNS resolvers may cache.

---

## Key Takeaway

Consul provides both static (config file) and dynamic (HTTP API) service registration. Health checks ensure only healthy instances are returned. The dual interface (HTTP API for programmatic queries, DNS for standard resolution) makes Consul work with any language and any infrastructure. Blocking queries enable efficient long-polling for changes without constant polling.
