# Solution 05: Multi-Datacenter Discovery Architecture

## Part A: Consul Multi-Datacenter Architecture

### Cluster Design

```
  dc-us-east (Virginia)                dc-eu-west (Ireland)
  +-----------------------+            +-----------------------+
  | Consul Servers (3)    |            | Consul Servers (3)    |
  |  server-1: 10.0.1.10  |            |  server-1: 10.1.1.10  |
  |  server-2: 10.0.1.11  |  <--WAN--> |  server-2: 10.1.1.11  |
  |  server-3: 10.0.1.12  |  gossip    |  server-3: 10.1.1.12  |
  +-----------------------+            +-----------------------+
       |                                    |
  +-----------------------+            +-----------------------+
  | Consul Agents (clients)|            | Consul Agents (clients)|
  |  on every host         |            | on every host          |
  +-----------------------+            +-----------------------+
       |                                    |
  [api, product, cart]                 [api, product, cart, db-replica]
```

**Why 3 servers per datacenter:**
- Raft consensus requires a quorum: `(n/2) + 1 = 2` for 3 nodes.
- 3 servers tolerate 1 failure. 5 servers tolerate 2 failures.
- 3 is the minimum for production. More than 5 adds latency without meaningful availability improvement.

### Server Configuration

**dc-us-east server (/etc/consul.d/server.hcl):**

```hcl
datacenter = "dc-us-east"
data_dir   = "/opt/consul"
server     = true
bootstrap_expect = 3

bind_addr  = "10.0.1.10"
client_addr = "0.0.0.0"

ui_config {
  enabled = true
}

connect {
  enabled = true
}

# Join local servers
retry_join = ["10.0.1.10", "10.0.1.11", "10.0.1.12"]

# WAN federation: join remote datacenter servers
retry_join_wan = ["10.1.1.10", "10.1.1.11", "10.1.1.12"]
```

**dc-eu-west server (/etc/consul.d/server.hcl):**

```hcl
datacenter = "dc-eu-west"
data_dir   = "/opt/consul"
server     = true
bootstrap_expect = 3

bind_addr  = "10.1.1.10"
client_addr = "0.0.0.0"

ui_config {
  enabled = true
}

connect {
  enabled = true
}

# Join local servers
retry_join = ["10.1.1.10", "10.1.1.11", "10.1.1.12"]

# WAN federation: join remote datacenter servers
retry_join_wan = ["10.0.1.10", "10.0.1.11", "10.0.1.12"]
```

### How WAN Federation Works

1. Each datacenter runs an independent Raft cluster (3 servers, local quorum).
2. WAN gossip connects server pools across datacenters using Serf.
3. WAN gossip propagates server membership, not service data.
4. Cross-datacenter queries are proxied: the local agent forwards the query to a remote datacenter's server via RPC.

### Service Registration

Services register with their local datacenter's agent. The agent automatically registers with the local server pool.

```bash
# In dc-us-east, register api
curl -X PUT http://localhost:8500/v1/agent/service/register \
  -d '{"ID": "api-1", "Name": "api", "Port": 8080, "Tags": ["production"]}'

# In dc-eu-west, register api (same service name, different datacenter)
curl -X PUT http://localhost:8500/v1/agent/service/register \
  -d '{"ID": "api-eu-1", "Name": "api", "Port": 8080, "Tags": ["production", "eu"]}'
```

### Common Mistakes to Avoid

- Running only 1 or 2 servers per datacenter. 1 has no fault tolerance. 2 cannot achieve quorum on a single failure (quorum of 2 requires both nodes).
- Confusing `retry_join` (local datacenter) with `retry_join_wan` (remote datacenter).
- Not opening the WAN gossip port (8302 TCP+UDP) between datacenters.

---

## Part B: Cross-Datacenter Service Queries

### 1. Query local datacenter only (default)

```bash
# From a node in dc-us-east, query local instances
curl "http://localhost:8500/v1/health/service/api?passing=true"

# DNS: standard query resolves local datacenter
dig @127.0.0.1 -p 8600 api.service.consul
```

### 2. Query all datacenters

```bash
# HTTP API: no "dc" parameter queries the local datacenter
# To query all, iterate over known datacenters
curl "http://localhost:8500/v1/health/service/api?passing=true&dc=dc-us-east"
curl "http://localhost:8500/v1/health/service/api?passing=true&dc=dc-eu-west"

# DNS: use the "any" datacenter query
dig @127.0.0.1 -p 8600 api.service.*.consul
```

### 3. Query dc-eu-west from dc-us-east

```bash
# HTTP API: specify the dc parameter
curl "http://localhost:8500/v1/health/service/api?passing=true&dc=dc-eu-west"

# DNS: use the datacenter-qualified name
dig @127.0.0.1 -p 8600 api.service.dc-eu-west.consul
```

### 4. Failover Strategy (Application Code)

```python
import requests
import random


class MultiDatacenterDiscovery:
    def __init__(self, consul_host="localhost", consul_port=8500,
                 local_dc="dc-us-east", fallback_dc="dc-eu-west"):
        self.base_url = f"http://{consul_host}:{consul_port}"
        self.local_dc = local_dc
        self.fallback_dc = fallback_dc

    def get_service(self, service_name):
        """Try local datacenter first, fall back to remote."""
        # Try local datacenter
        instances = self._query_dc(service_name, self.local_dc)
        if instances:
            return instances, self.local_dc

        # Fall back to remote datacenter
        instances = self._query_dc(service_name, self.fallback_dc)
        if instances:
            return instances, self.fallback_dc

        raise Exception(
            f"No healthy instances of {service_name} in any datacenter"
        )

    def _query_dc(self, service_name, datacenter):
        url = f"{self.base_url}/v1/health/service/{service_name}"
        params = {"passing": "true", "dc": datacenter}

        try:
            response = requests.get(url, params=params, timeout=5)
            return response.json()
        except requests.RequestException:
            return []

    def get_address(self, service_name):
        """Get a random healthy instance, preferring local."""
        instances, dc = self.get_service(service_name)
        instance = random.choice(instances)
        addr = instance["Service"]["Address"]
        port = instance["Service"]["Port"]
        return f"{addr}:{port}", dc


# Usage
discovery = MultiDatacenterDiscovery(local_dc="dc-us-east")
address, dc = discovery.get_address("api")
print(f"Resolved to {address} in {dc}")
```

### Common Mistakes to Avoid

- Not specifying `?passing=true`. Without it, you get instances with failed health checks.
- Querying the wrong datacenter's agent. Cross-datacenter queries must go through the local agent, which proxies to the remote datacenter.
- Not handling the case where both datacenters return no instances.

---

## Part C: Failure Scenarios

### Scenario 1: One Consul server in dc-us-east crashes

**What happens:** The remaining 2 servers maintain quorum (2 of 3). All service discovery continues without interruption. The crashed server is marked as failed by the Raft protocol.

**Recovery:** Restart the Consul server. It rejoins the cluster, catches up on missed log entries, and resumes participating in consensus. No manual intervention needed.

**Impact:** None. The cluster is designed to tolerate this.

### Scenario 2: All Consul servers in dc-us-east crash

**What happens:** Quorum is lost (0 of 3). The Raft cluster cannot elect a leader or process writes. New service registrations fail. Existing services continue to run but cannot register or deregister. Reads from the local agent's cache may still work for a short time.

**Recovery:** Restore at least 2 of the 3 servers. If all data is lost, you must rebuild the cluster from scratch and re-register all services.

**Impact:** Service discovery in dc-us-east is unavailable. dc-eu-west is unaffected (independent Raft cluster).

### Scenario 3: WAN link between datacenters is severed

**What happens:** Each datacenter continues to operate independently. Local service discovery works perfectly. Cross-datacenter queries fail (cannot proxy to the remote datacenter).

**Recovery:** Restore the WAN link. Gossip protocol automatically detects the restored connection and resynchronizes server membership.

**Impact:** Cross-datacenter discovery is unavailable. Applications using failover logic (Part B) will only find local instances. This is the correct behavior -- if the WAN link is down, you probably cannot reach remote services anyway.

### Scenario 4: A service in dc-eu-west registers but its health check fails

**What happens:** The service is registered in Consul but marked as "critical." Queries with `?passing=true` exclude it. Queries without `?passing=true` include it with a failing status.

**Recovery:** Fix the health check (the service is unhealthy). The health check will eventually pass, and the service will be included in passing queries. If the health check fails for `deregister_critical_service_after` (e.g., 30 minutes), the service is automatically deregistered.

**Impact:** The service is not discoverable by healthy-only queries. This is correct -- you do not want traffic routed to an unhealthy instance.

### Scenario 5: GDPR-sensitive service in dc-eu-west queried from dc-us-east

**What happens:** The query returns the service's metadata (name, address, port, tags). This is service discovery metadata, not user data. The query itself does not violate GDPR because it does not access, process, or transfer personal data.

**However:** If the query is used to route a request that contains European user data to dc-us-east, that would violate GDPR. The service discovery layer is fine; the data routing layer must enforce data residency.

**Recovery:** Ensure that GDPR-sensitive services in dc-eu-west are only called by services in dc-eu-west. Use network policies, service mesh intentions, or application-level routing rules to enforce this.

**Impact:** Service discovery metadata is not user data. But routing decisions based on discovery must respect data residency requirements. Document the boundary clearly.

### Common Mistakes to Avoid

- Confusing service discovery metadata with user data. Service names and IP addresses are infrastructure metadata.
- Not planning for WAN link failure. It will happen. Design for it.
- Assuming that a single server failure is catastrophic. Raft is designed for this.

---

## Key Takeaway

Multi-datacenter service discovery with Consul uses WAN federation to connect independent Raft clusters. Each datacenter operates autonomously, so local discovery works even when the WAN link fails. Cross-datacenter queries are proxied through the WAN gossip layer. Failure scenarios range from trivial (single server crash) to severe (total datacenter failure), but the architecture degrades gracefully. GDPR compliance is about data routing, not service discovery metadata -- but the two are connected because discovery determines where requests are routed.
