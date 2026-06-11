# Cheatsheet: Service Discovery

## Methods

| Method | Description | Example |
|--------|-------------|---------|
| DNS | Name → IP | CoreDNS in k8s |
| Key-Value Store | Central registry | etcd, Consul |
| API-based | Query API | Kubernetes API |

## Consul
```bash
# Run Consul
docker run -d --name consul -p 8500:8500 consul:latest

# Register service
curl -X PUT http://localhost:8500/v1/agent/service/register \
  -d '{
    "Name": "web",
    "Port": 8080,
    "Check": {
      "HTTP": "http://localhost:8080/health",
      "Interval": "10s"
    }
  }'

# Discover services
dig @127.0.0.1 -p 8600 web.service.consul SRV
```

## Kubernetes DNS
```bash
# Service discovery via DNS
# Same namespace
curl http://my-service:8080

# Different namespace
curl http://my-service.other-namespace:8080

# Full FQDN
curl http://my-service.other-namespace.svc.cluster.local:8080
```

## etcd
```bash
# Set value
etcdctl put /services/web/1 '{"ip":"10.0.0.1","port":8080}'

# Get value
etcdctl get /services/web/1

# Watch for changes
etcdctl watch /services/web/
```
