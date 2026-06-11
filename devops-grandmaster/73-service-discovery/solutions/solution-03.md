# Solution 03: etcd-Based Service Registry

## Part A: EtcdServiceRegistry Implementation

```python
import etcd3
import json
import threading
import time


class EtcdServiceRegistry:
    def __init__(self, host="localhost", port=2379):
        self.client = etcd3.client(host=host, port=port)
        self.services = {}  # service_name -> key
        self.lease = None
        self._renewal_thread = None

    def register(self, service_name, address, port, ttl=30):
        """Register a service with a lease-based TTL."""
        # Create a lease with the specified TTL
        self.lease = self.client.lease(ttl)

        # Build the key and value
        service_key = f"/services/{service_name}/{address}:{port}"
        service_data = json.dumps({
            "name": service_name,
            "address": address,
            "port": port,
            "registered_at": time.time()
        })

        # Store the key with the lease
        self.client.put(service_key, service_data, lease=self.lease)
        self.services[service_name] = service_key

        # Start lease renewal in a background thread
        self._renewal_thread = threading.Thread(
            target=self._renew_lease, daemon=True
        )
        self._renewal_thread.start()

        print(f"Registered {service_name} at {address}:{port} with TTL {ttl}s")

    def deregister(self, service_name):
        """Remove a service from the registry."""
        if service_name in self.services:
            key = self.services.pop(service_name)
            self.client.delete(key)
            print(f"Deregistered {service_name}")
        else:
            print(f"Service {service_name} not found in registry")

    def discover(self, service_name):
        """Find all instances of a service."""
        prefix = f"/services/{service_name}/"
        instances = []

        for value, metadata in self.client.get_prefix(prefix):
            instance = json.loads(value)
            instances.append(instance)

        return instances

    def watch(self, service_name, callback):
        """Watch for changes to a service and invoke callback."""
        prefix = f"/services/{service_name}/"
        events_iterator, cancel = self.client.watch_prefix(prefix)

        for event in events_iterator:
            # event is a tuple of (event_type, key, value)
            callback(event)

        return cancel

    def _renew_lease(self):
        """Background thread to renew the lease before it expires."""
        while self.lease:
            try:
                # Renew at 1/3 of TTL to allow for retries
                time.sleep(self.lease.ttl // 3)
                self.lease.refresh()
            except Exception as e:
                print(f"Lease renewal failed: {e}")
                break
```

### Why This Works

- **Lease-based TTL:** When the service crashes, the renewal thread dies. The lease expires after `ttl` seconds, and etcd automatically deletes the key. This is the core mechanism for crash detection.
- **Prefix queries:** `get_prefix("/services/api/")` returns all keys that start with this prefix, giving you all instances of the `api` service.
- **Watch mechanism:** `watch_prefix` returns an iterator that yields events whenever a key under the prefix changes (put or delete). This enables reactive updates instead of polling.
- **Daemon thread:** The renewal thread is a daemon, so it does not prevent the process from exiting.

### Common Mistakes to Avoid

- Not creating a lease at all. Without a lease, the key persists forever and crashed services are never removed.
- Renewing too infrequently. If the renewal interval equals the TTL, a single missed renewal causes deregistration. Renewing at 1/3 TTL allows 2 retries before expiration.
- Not handling the case where `get_prefix` returns no results. An empty list is a valid result (no healthy instances).

---

## Part B: Lease Renewal Behavior

The renewal thread calls `self.lease.refresh()` at `ttl // 3` intervals (e.g., every 10 seconds for a 30-second TTL).

**What happens if the renewal thread crashes:**

1. The lease is not renewed.
2. After `ttl` seconds, the lease expires.
3. etcd automatically deletes all keys associated with the lease.
4. The service disappears from the registry.

This is the desired behavior. If the renewal thread crashes, it likely means the entire process is unhealthy. Automatic deregistration is the correct response.

**What happens if the etcd server is unreachable:**

1. `self.lease.refresh()` raises an exception.
2. The renewal thread catches the exception and breaks out of the loop.
3. The lease expires and the key is deleted.
4. When etcd becomes reachable again, the service must re-register.

### Common Mistakes to Avoid

- Catching exceptions too broadly in the renewal thread. You want to distinguish between transient errors (retry) and permanent errors (stop renewal).
- Not re-registering after a lease expires. The renewal thread exits, but the main application may still be running. Consider adding a callback or re-registration logic.

---

## Part C: Comparison with Consul

### 1. What Consul Provides Out of the Box

| Feature | Consul | etcd (your implementation) |
|---------|--------|---------------------------|
| Health checking | Built-in (HTTP, TCP, gRPC, script) | Manual (you must implement it) |
| DNS interface | Built-in on port 8600 | None (must build your own) |
| Multi-datacenter | Built-in WAN federation | Manual (must configure cross-cluster replication) |
| Gossip protocol | Built-in (Serf) for membership and failure detection | None (uses Raft for consistency, not membership) |
| Service metadata | Tags, meta, kind | Key-value only (you structure it yourself) |
| Blocking queries | Built-in long-polling | Watch API (gRPC streaming) |

### 2. Advantages of the etcd Approach

- **Strong consistency:** etcd uses Raft for every write. You get linearizable reads. Consul allows stale reads by default for performance.
- **Kubernetes native:** etcd is the backing store for Kubernetes. If you are already running K8s, you have etcd.
- **Simpler operational model:** etcd is a key-value store. No gossip protocol, no agent model, no separate DNS server.
- **Watch API:** etcd's gRPC streaming watch is more efficient than Consul's blocking queries for high-frequency changes.

### 3. When to Choose etcd Over Consul

- **You are building on Kubernetes.** etcd is already there. Adding Consul means running a second coordination system.
- **You need strong consistency guarantees.** etcd's linearizable reads ensure you always see the latest write.
- **You are building a custom coordination system.** etcd is a better building block (leases, watches, transactions) than Consul if you need fine-grained control.
- **You do not need multi-datacenter or DNS-based discovery.** Consul's strengths (WAN federation, DNS) are irrelevant if you only have one datacenter and use HTTP-based discovery.

### Common Mistakes to Avoid

- Saying etcd is "better" or "worse" than Consul. They solve different problems. etcd is a coordination primitive; Consul is a service mesh platform.
- Forgetting that etcd requires you to build health checking yourself. Consul does this out of the box.

---

## Key Takeaway

etcd is a distributed key-value store that can serve as the foundation for service discovery, but it requires manual implementation of leases, health checks, and discovery queries. Consul provides all of this out of the box plus DNS, multi-datacenter, and gossip-based failure detection. Choose etcd when you need strong consistency and are already in the Kubernetes ecosystem. Choose Consul when you need a complete service discovery platform with minimal custom code.
