# Exercise 04: Client-Side vs. Server-Side Discovery

**Type:** Challenge
**Time:** 45 minutes
**Difficulty:** Medium-Hard

## Objective

You will implement both client-side and server-side discovery patterns and compare their trade-offs in terms of latency, complexity, failure modes, and operational overhead.

## Scenario

Your company runs an e-commerce platform with the following services:

- **gateway** -- API gateway, receives all external traffic.
- **product** -- Product catalog, 4 replicas.
- **cart** -- Shopping cart, 3 replicas.
- **payment** -- Payment processing, 2 replicas.

You need to decide whether to implement client-side discovery (the gateway picks an instance) or server-side discovery (a proxy handles routing).

## Tasks

### Part A: Client-Side Discovery Implementation

Write a Python class `ClientSideDiscovery` that:

1. Queries a service registry (Consul or etcd) for healthy instances.
2. Implements round-robin load balancing across instances.
3. Handles the case where no healthy instances are available (raise an exception or return a fallback).
4. Includes a simple circuit breaker that stops calling a service after 3 consecutive failures.

```python
class ClientSideDiscovery:
    def __init__(self, registry):
        self.registry = registry
        # Add state for round-robin and circuit breaker

    def call_service(self, service_name, path, method="GET", **kwargs):
        # Your implementation here
        pass
```

<details>
<summary>Hint</summary>
Use a counter modulo the number of instances for round-robin. Track consecutive failures per service for the circuit breaker. The circuit breaker should have a cooldown period before retrying.
</details>

### Part B: Server-Side Discovery with Consul Template

Write an Nginx configuration template (using Consul Template syntax) that:

1. Automatically generates upstream blocks for each service.
2. Reloads Nginx when service instances change.
3. Includes health check configuration.

Also write the `consul-template` command to run this configuration.

<details>
<summary>Hint</summary>
Use `{{ range service "service_name" }}` to iterate over instances. The reload command is specified in the template path: `template.ctmpl:output_path:reload_command`.
</details>

### Part C: Trade-off Analysis

For each dimension below, explain which approach is better and why:

1. **Latency** -- Which adds more latency to each request?
2. **Failure isolation** -- Which handles a registry failure better?
3. **Language support** -- Which works across polyglot services?
4. **Load balancing sophistication** -- Which supports more advanced algorithms?
5. **Operational complexity** -- Which is harder to operate in production?

<details>
<summary>Hint</summary>
Client-side has no extra hop but couples discovery logic to every service. Server-side adds a hop but centralizes routing. Think about what happens when the registry is down in each case.
</details>

## Success Criteria

- [ ] Client-side implementation includes round-robin, error handling, and circuit breaker logic.
- [ ] Server-side template correctly uses Consul Template syntax for dynamic upstream generation.
- [ ] Trade-off analysis covers all 5 dimensions with specific, justified answers.
- [ ] You can articulate the failure mode of each approach when the registry is unavailable.

## What You Should Understanding After This Exercise

Client-side discovery gives callers full control over routing, enabling sophisticated load balancing and circuit breaking, but couples discovery logic to every service and requires client libraries for each language. Server-side discovery centralizes routing in a proxy, works with any language, and is operationally simpler, but adds latency per request and creates a potential bottleneck. In practice, many production systems use a hybrid: a service mesh (like Consul Connect or Istio) that provides server-side routing with client-side intelligence.
