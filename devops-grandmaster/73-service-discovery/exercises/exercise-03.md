# Exercise 03: etcd-Based Service Registry

**Type:** Independent
**Time:** 40 minutes
**Difficulty:** Medium

## Objective

You will build a service registry on top of etcd using lease-based registration with automatic deregistration. This exercise practices etcd operations, TTL-based leases, prefix queries, and watch mechanisms.

## Scenario

Your team has chosen etcd as the backing store for a custom service registry. You need to implement a Python class that:

1. Registers services with a lease-based TTL (so crashed services are automatically removed).
2. Discovers healthy instances by querying a key prefix.
3. Watches for changes to a service and triggers callbacks.
4. Handles lease renewal in a background thread.

## Tasks

### Part A: Service Registration with Leases

Write a Python class `EtcdServiceRegistry` with the following methods:

- `register(service_name, address, port, ttl=30)` -- Registers a service under `/services/<name>/<address>:<port>` with a lease that expires after `ttl` seconds.
- `deregister(service_name)` -- Removes the service key.
- `discover(service_name)` -- Returns a list of all registered instances for the service.
- `watch(service_name, callback)` -- Watches for changes to the service prefix and calls the callback on each event.

Use the `etcd3` Python client library.

<details>
<summary>Hint</summary>
Use `self.client.lease(ttl)` to create a lease, then pass `lease=self.lease` to `self.client.put()`. For prefix queries, use `self.client.get_prefix(prefix)`. For watching, use `self.client.watch_prefix(prefix)`.
</details>

### Part B: Lease Renewal

The lease has a TTL and must be renewed before it expires. Write a background thread that renews the lease at `ttl // 3` intervals. What happens if the renewal thread crashes?

<details>
<summary>Hint</summary>
Use `self.lease.refresh()` to renew. The renewal interval should be 1/3 of the TTL to allow for retries. If renewal fails, the lease expires and the key is deleted -- this is the desired behavior for crash detection.
</details>

### Part C: Comparison with Consul

Compare your etcd-based registry with Consul's built-in service discovery:

1. What does Consul provide out of the box that you had to build manually?
2. What are the advantages of the etcd approach?
3. When would you choose etcd over Consul?

<details>
<summary>Hint</summary>
Consul provides: health checks, DNS interface, multi-datacenter support, gossip protocol. etcd provides: strong consistency (Raft), Kubernetes integration, simpler operational model.
</details>

## Success Criteria

- [ ] Your `EtcdServiceRegistry` class implements all 4 methods.
- [ ] Lease-based TTL ensures automatic deregistration on crash.
- [ ] Lease renewal runs in a background thread at 1/3 TTL interval.
- [ ] The `watch` method uses prefix watching and invokes callbacks.
- [ ] Your comparison identifies at least 3 differences between etcd and Consul.

## What You Should Understand After This Exercise

etcd is a distributed key-value store, not a service discovery system. Building service discovery on etcd requires manual implementation of leases, health checks, and discovery queries. Consul provides all of this out of the box. However, etcd's strong consistency (Raft consensus) and its role as the Kubernetes backing store make it the right choice when you need a coordination primitive rather than a full service mesh.
