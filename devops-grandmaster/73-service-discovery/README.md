# 73 - Service Discovery

> **Previous:** [72 - Multi-Host Orchestration](../72-multi-host-orchestration/README.md)
> **Next:** [74 - Distributed Systems Patterns](../74-distributed-systems-patterns/README.md)

## The Problem

In a static environment, you know where every service lives. You hardcode IP addresses in configuration files: the database is at `10.0.1.50`, the payment API is at `10.0.1.60`. But in a dynamic environment — containers scheduled across hosts, auto-scaling groups, rolling deployments, failover — service instances come and go. IP addresses change. Ports change. Hardcoded addresses break.

You need a system where services register themselves when they start, deregister when they stop, and other services can query the registry to find healthy instances. This is service discovery: the phone book of your infrastructure.

---

## The Naive Way

Hardcode service addresses in configuration files or environment variables.

```yaml
# application.yml — the "phone book in stone" approach
database:
  host: 10.0.1.50
  port: 5432

payment_service:
  host: 10.0.1.60
  port: 8080

notification_service:
  host: 10.0.1.70
  port: 9090

email_service:
  host: 10.0.1.80
  port: 3000
```

**Why this fails:**
- If `10.0.1.60` goes down, the payment service is unreachable until you manually update the config.
- Auto-scaling adds new instances, but nobody knows their addresses.
- Rolling deployments create new containers with new IPs — the old config points to the old container.
- Every service needs to know the address of every other service it calls.
- Configuration changes require redeployments.
- No health checking — you don't know if the address is still alive until a request fails.

---

## The Right Way

Use a service discovery system that provides dynamic registration, health checking, and resolution.

### DNS-Based Discovery

The simplest approach: use DNS as the discovery mechanism. Docker Swarm and Kubernetes both use this pattern internally.

**How it works:**
1. Services register with a DNS server (or the orchestrator does it automatically).
2. Clients look up the service name via DNS.
3. DNS returns one or more IP addresses of healthy instances.
4. Client connects to one of the returned addresses.

```bash
# Docker Swarm — automatic DNS discovery
docker service create --name api --replicas 3 myapp/api:latest

# Any container in the same network can resolve "api"
docker exec some_container nslookup api
# Returns IPs of all 3 replicas

# Kubernetes — CoreDNS
kubectl get svc api
# api.default.svc.cluster.local -> 10.96.0.100 (ClusterIP)
```

```python
# Application code using DNS discovery
import socket

def get_service_address(service_name, port):
    """Resolve service name to IP addresses via DNS."""
    try:
        results = socket.getaddrinfo(
            service_name,
            port,
            socket.AF_INET,
            socket.SOCK_STREAM
        )
        addresses = [(r[4][0], r[4][1]) for r in results]
        return addresses
    except socket.gaierror:
        return []

# Usage
addresses = get_service_address("api.internal", 8080)
# Could return: [('10.0.1.10', 8080), ('10.0.1.11', 8080), ('10.0.1.12', 8080)]
```

**Limitations of DNS-based discovery:**
- DNS TTL causes propagation delays — clients cache DNS results.
- No health checking at the DNS level (unless the orchestrator handles it).
- Limited to IP addresses — no metadata (version, region, weight).
- Client-side load balancing relies on DNS round-robin, which is crude.

### Consul

HashiCorp Consul is the industry standard for service discovery. It provides registration, health checking, DNS and HTTP interfaces, key-value storage, and multi-datacenter support.

**Architecture:**

```
  Datacenter 1                    Datacenter 2
  +-----------+                   +-----------+
  |  Consul   |  <-- WAN gossip -->|  Consul  |
  |  Server 1 |                   |  Server 1 |
  |  Server 2 |                   |  Server 2 |
  |  Server 3 |                   |  Server 3 |
  +-----------+                   +-----------+
       |                               |
  +-----------+                   +-----------+
  | Consul    |                   | Consul    |
  | Agents    |                   | Agents    |
  | (clients) |                   | (clients) |
  +-----------+                   +-----------+
  |  |  |  |                      |  |  |  |
  svc svc svc svc                svc svc svc svc
```

- **Server nodes** maintain the cluster state, participate in Raft consensus (3 or 5 per datacenter).
- **Client agents** run on every host, forward requests to servers, perform local health checks.
- **Gossip protocol** propagates membership and failure detection.

**Setup:**

```bash
# Install Consul
wget https://releases.hashicorp.com/consul/1.16.0/consul_1.16.0_linux_amd64.zip
unzip consul_1.16.0_linux_amd64.zip
sudo mv consul /usr/local/bin/

# Server configuration (/etc/consul.d/server.hcl)
datacenter = "dc1"
data_dir   = "/opt/consul"
server     = true
bootstrap_expect = 3

bind_addr = "192.168.1.10"
client_addr = "0.0.0.0"

ui_config {
  enabled = true
}

connect {
  enabled = true
}

retry_join = ["192.168.1.10", "192.168.1.11", "192.168.1.12"]
```

**Service Registration:**

```json
// /etc/consul.d/api.json — service definition
{
  "service": {
    "name": "api",
    "port": 8080,
    "tags": ["v2", "production", "primary"],
    "meta": {
      "version": "2.1.0",
      "region": "us-east"
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

```bash
# Register via HTTP API (dynamic registration)
curl -X PUT http://localhost:8500/v1/agent/service/register \
  -d '{
    "ID": "api-1",
    "Name": "api",
    "Port": 8080,
    "Tags": ["v2", "production"],
    "Check": {
      "HTTP": "http://localhost:8080/health",
      "Interval": "10s",
      "Timeout": "3s"
    }
  }'

# Deregister
curl -X PUT http://localhost:8500/v1/agent/service/deregister/api-1
```

**Querying Services:**

```bash
# HTTP API — discover all healthy instances of a service
curl http://localhost:8500/v1/health/service/api?passing=true

# DNS interface (Consul runs a DNS server on port 8600)
dig @127.0.0.1 -p 8600 api.service.consul SRV

# Blocking queries (long-poll for changes)
curl "http://localhost:8500/v1/health/service/api?passing=true&index=1234&wait=5m"
```

**Application Integration:**

```python
import requests
import random

class ConsulDiscovery:
    def __init__(self, consul_host="localhost", consul_port=8500):
        self.base_url = f"http://{consul_host}:{consul_port}"

    def get_service(self, service_name, tag=None):
        """Get all healthy instances of a service."""
        url = f"{self.base_url}/v1/health/service/{service_name}"
        params = {"passing": "true"}
        if tag:
            params["tag"] = tag

        response = requests.get(url, params=params)
        instances = []

        for entry in response.json():
            instances.append({
                "id": entry["Service"]["ID"],
                "address": entry["Service"]["Address"],
                "port": entry["Service"]["Port"],
                "tags": entry["Service"]["Tags"],
                "meta": entry["Service"].get("Meta", {})
            })

        return instances

    def get_service_address(self, service_name):
        """Get a random healthy instance address."""
        instances = self.get_service(service_name)
        if not instances:
            raise Exception(f"No healthy instances of {service_name}")

        instance = random.choice(instances)
        return f"{instance['address']}:{instance['port']}"

# Usage
discovery = ConsulDiscovery()
api_address = discovery.get_service_address("api")
response = requests.get(f"http://{api_address}/users/123")
```

### etcd

etcd is a distributed key-value store that serves as the backbone of Kubernetes. It can be used for service discovery, configuration management, and distributed coordination.

**Setup:**

```bash
# Start a 3-node etcd cluster
etcd --name node1 \
  --initial-advertise-peer-urls http://192.168.1.10:2380 \
  --listen-peer-urls http://192.168.1.10:2380 \
  --listen-client-urls http://192.168.1.10:2379,http://127.0.0.1:2379 \
  --advertise-client-urls http://192.168.1.10:2379 \
  --initial-cluster node1=http://192.168.1.10:2380,node2=http://192.168.1.11:2380,node3=http://192.168.1.12:2380 \
  --initial-cluster-state new
```

**Service Registration Pattern:**

```python
import etcd3
import json
import threading
import time

class EtcdServiceRegistry:
    def __init__(self, host="localhost", port=2379):
        self.client = etcd3.client(host=host, port=port)
        self.services = {}
        self.lease = None

    def register(self, service_name, address, port, ttl=30):
        """Register a service with a lease-based TTL."""
        self.lease = self.client.lease(ttl)

        service_key = f"/services/{service_name}/{address}:{port}"
        service_data = json.dumps({
            "name": service_name,
            "address": address,
            "port": port,
            "registered_at": time.time()
        })

        self.client.put(service_key, service_data, lease=self.lease)
        self.services[service_name] = service_key

        renewal_thread = threading.Thread(
            target=self._renew_lease, daemon=True
        )
        renewal_thread.start()

    def deregister(self, service_name):
        if service_name in self.services:
            key = self.services.pop(service_name)
            self.client.delete(key)

    def discover(self, service_name):
        """Find all instances of a service."""
        prefix = f"/services/{service_name}/"
        instances = []

        for value, metadata in self.client.get_prefix(prefix):
            instance = json.loads(value)
            instances.append(instance)

        return instances

    def watch(self, service_name, callback):
        """Watch for changes to a service."""
        prefix = f"/services/{service_name}/"
        events_iterator, cancel = self.client.watch_prefix(prefix)

        for event in events_iterator:
            callback(event)

    def _renew_lease(self):
        while self.lease:
            try:
                self.lease.refresh()
                time.sleep(self.lease.ttl // 3)
            except Exception:
                break

# Usage
registry = EtcdServiceRegistry()
registry.register("api", "10.0.1.20", 8080)

instances = registry.discover("api")
```

### ZooKeeper

Apache ZooKeeper is the original distributed coordination service. It is the foundation of many Hadoop ecosystem tools.

```java
// ZooKeeper service registration (Java)
public class ZooKeeperServiceRegistry {
    private ZooKeeper zk;
    private String basePath = "/services";

    public void register(String serviceName, String address, int port)
            throws Exception {
        String servicePath = basePath + "/" + serviceName;
        if (zk.exists(servicePath, false) == null) {
            zk.create(servicePath, new byte[0],
                      ZooDefs.Ids.OPEN_ACL_UNSAFE,
                      CreateMode.PERSISTENT);
        }

        // Ephemeral node — automatically deleted when session ends
        String instancePath = servicePath + "/" + address + ":" + port;
        zk.create(instancePath, new byte[0],
                  ZooDefs.Ids.OPEN_ACL_UNSAFE,
                  CreateMode.EPHEMERAL);
    }

    public List<String> discover(String serviceName) throws Exception {
        String servicePath = basePath + "/" + serviceName;
        return zk.getChildren(servicePath, true);
    }
}
```

**ZooKeeper vs Consul vs etcd:**

| Feature | ZooKeeper | Consul | etcd |
|---------|-----------|--------|------|
| Language | Java | Go | Go |
| Consensus | ZAB | Raft | Raft |
| Service discovery | Manual (ephemeral nodes) | Built-in | Manual (leases) |
| Health checking | Client-side | Built-in (HTTP/TCP/gRPC/Script) | Manual |
| DNS interface | No | Built-in (port 8600) | No |
| Multi-datacenter | Complex | Built-in | Manual |
| KV store | Yes | Yes | Yes |
| Watch mechanism | Yes | Blocking queries | Yes (gRPC streaming) |
| ACL | ACLs | ACLs + intentions | RBAC |
| Memory footprint | Heavy (JVM) | Light | Light |
| Configuration | Complex XML | Simple HCL | CLI flags/YAML |
| Best for | Hadoop ecosystem, legacy systems | Service discovery, service mesh | Kubernetes, configuration |

### Client-Side vs Server-Side Discovery

**Client-Side Discovery:**

The client queries the service registry directly and picks an instance.

```
+--------+     +------------------+     +--------+
| Client |---->| Service Registry |     | Svc A1 |
+--------+     +------------------+     +--------+
     |                                      ^
     |         (client picks instance)       |
     +--------------------------------------+
              +--------+
              | Svc A2 |
              +--------+
```

```python
# Client-side discovery with load balancing
class ClientSideDiscovery:
    def __init__(self, registry):
        self.registry = registry

    def call_service(self, service_name, path, **kwargs):
        instances = self.registry.discover(service_name)
        healthy = [i for i in instances if i.get("healthy", True)]

        if not healthy:
            raise Exception(f"No healthy instances of {service_name}")

        instance = random.choice(healthy)
        url = f"http://{instance['address']}:{instance['port']}{path}"
        return requests.request(url=url, **kwargs)
```

**Pros:** No extra hop. Client can implement smart load balancing (circuit breaking, retries, hedging).
**Cons:** Discovery logic coupled to every service. Each language needs its own discovery client.

**Server-Side Discovery:**

A load balancer or proxy queries the registry and routes traffic.

```
+--------+     +-------------+     +------------------+     +--------+
| Client |---->| Load        |---->| Service Registry |     | Svc A1 |
+--------+     | Balancer    |     +------------------+     +--------+
               +-------------+            |                   ^
                     |                    |                    |
                     +--------------------+--------------------+
                              (LB picks instance)
                                          +--------+
                                          | Svc A2 |
                                          +--------+
```

```nginx
# Nginx with Consul Template for server-side discovery
# consul-template generates nginx config from Consul data
# /etc/consul-template/nginx.conf.ctmpl

{{ range service "api" }}
upstream api_backend {
  {{ range service "api" }}
  server {{ .Address }}:{{ .Port }} weight=10;
  {{ end }}
}
{{ end }}

server {
  listen 80;
  location /api/ {
    proxy_pass http://api_backend;
  }
}
```

```bash
# consul-template watches Consul and regenerates config
consul-template \
  -template "/etc/consul-template/nginx.conf.ctmpl:/etc/nginx/conf.d/api.conf:nginx -s reload"
```

**Pros:** Simple clients. Centralized routing logic. Works with any language.
**Cons:** Extra hop adds latency. Load balancer is a single point of failure (unless clustered).

---

## The Production Way

### Consul with Service Mesh (Connect)

Consul Connect provides automatic TLS encryption and authorization between services.

```json
// Service definition with sidecar proxy
{
  "service": {
    "name": "api",
    "port": 8080,
    "connect": {
      "sidecar_service": {
        "proxy": {
          "upstreams": [
            {
              "destination_name": "payment",
              "local_bind_port": 9090
            },
            {
              "destination_name": "database",
              "local_bind_port": 5432
            }
          ]
        }
      }
    }
  }
}
```

```bash
# Intention-based authorization
consul intention create api payment
consul intention create -deny api admin

# List intentions
consul intention list
```

### Health Check Patterns

```json
{
  "check": [
    {
      "id": "http-check",
      "http": "http://localhost:8080/health",
      "interval": "10s",
      "timeout": "3s"
    },
    {
      "id": "tcp-check",
      "tcp": "localhost:8080",
      "interval": "5s",
      "timeout": "2s"
    },
    {
      "id": "script-check",
      "script": "/opt/scripts/check_disk.sh",
      "interval": "30s",
      "timeout": "5s"
    },
    {
      "id": "grpc-check",
      "grpc": "localhost:50051",
      "grpc_use_tls": true,
      "interval": "10s"
    }
  ]
}
```

### Multi-Datacenter Federation

```bash
# Connect two Consul datacenters via WAN gossip
consul join -wan 10.0.1.10 10.0.2.10

# Query services across datacenters
dig @127.0.0.1 -p 8600 api.service.consul SRV          # local dc
dig @127.0.0.1 -p 8600 api.service.dc2.consul SRV       # remote dc

# Prepared queries for failover across datacenters
curl -X PUT http://localhost:8500/v1/query \
  -d '{
    "Name": "api-failover",
    "Service": {
      "Service": "api",
      "Failover": {
        "Datacenters": ["dc2", "dc3"]
      }
    }
  }'
```

---

## Hands-On Lab

### Exercise 1: Consul Cluster Setup

```bash
# Start 3 Consul servers in Docker
for i in 1 2 3; do
  docker run -d --name consul-$i \
    -p $((8500 + i - 1)):8500 \
    consul:1.16 agent -server -bootstrap-expect=3 \
    -node=consul-$i -client=0.0.0.0 \
    -retry-join=consul-1
done

# Verify cluster
docker exec consul-1 consul members
```

### Exercise 2: Register and Discover Services

```bash
# Register a service via API
curl -X PUT http://localhost:8500/v1/agent/service/register \
  -d '{
    "ID": "web-1",
    "Name": "web",
    "Port": 8080,
    "Tags": ["v1"],
    "Check": {
      "HTTP": "http://httpbin.org/status/200",
      "Interval": "10s"
    }
  }'

# Register a second instance
curl -X PUT http://localhost:8500/v1/agent/service/register \
  -d '{
    "ID": "web-2",
    "Name": "web",
    "Port": 8081,
    "Tags": ["v1"],
    "Check": {
      "HTTP": "http://httpbin.org/status/200",
      "Interval": "10s"
    }
  }'

# Query services
curl http://localhost:8500/v1/health/service/web?passing=true | jq .

# DNS lookup
dig @127.0.0.1 -p 8600 web.service.consul SRV
```

### Exercise 3: etcd Service Registration

```bash
# Start etcd
docker run -d --name etcd -p 2379:2379 -p 2380:2380 \
  quay.io/coreos/etcd:v3.5.9 \
  etcd --listen-client-urls http://0.0.0.0:2379 \
  --advertise-client-urls http://localhost:2379

# Register a service
etcdctl put /services/api/instance1 '{"address":"10.0.1.10","port":8080}'
etcdctl put /services/api/instance2 '{"address":"10.0.1.11","port":8080}'

# List instances
etcdctl get /services/api/ --prefix

# Watch for changes
etcdctl watch /services/api/ --prefix
```

### Exercise 4: Compare Discovery Approaches

```python
# discovery_comparison.py
import time
import requests

# Approach 1: Direct (hardcoded)
def call_direct(path):
    return requests.get(f"http://10.0.1.60:8080{path}")

# Approach 2: DNS-based
def call_dns(path):
    import socket
    ips = socket.getaddrinfo("api.internal", 8080)
    addr = ips[0][4][0]
    return requests.get(f"http://{addr}:8080{path}")

# Approach 3: Consul HTTP
def call_consul(path):
    resp = requests.get("http://localhost:8500/v1/health/service/api?passing=true")
    instances = resp.json()
    instance = instances[0]["Service"]
    return requests.get(f"http://{instance['Address']}:{instance['Port']}{path}")

# Measure latency of each approach
for name, func in [("direct", call_direct), ("dns", call_dns), ("consul", call_consul)]:
    start = time.time()
    try:
        func("/health")
    except Exception:
        pass
    print(f"{name}: {(time.time() - start) * 1000:.2f}ms")
```

---

## Limitation

Service discovery tells you where services are and whether they are healthy. It does not tell you how to handle the inevitable failures of distributed systems — network partitions, split-brain scenarios, cascading failures, and consistency trade-offs. Once services can find each other across multiple hosts, you need patterns for dealing with the fundamental challenges of distributed computing.

---

## Next Topic

[74 - Distributed Systems Patterns](../74-distributed-systems-patterns/README.md) — Understand CAP theorem, consensus algorithms, circuit breakers, and the patterns that make distributed systems reliable.
