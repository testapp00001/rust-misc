# Exercise 01: Load Balancing Algorithm Selection

**Type:** Conceptual
**Time:** 20 min
**Difficulty:** Easy

## Objective

Understand the characteristics, strengths, and weaknesses of common load balancing algorithms and identify the best algorithm for each scenario.

## Scenario

You are a platform engineer at a company with five different services. Each service has different traffic patterns and requirements. You need to select the right load balancing algorithm for each.

```
Services:
├── Service A: Stateless REST API, uniform request duration (~50ms each)
├── Service B: WebSocket server, long-lived connections (hours), varying connection count per client
├── Service C: E-commerce cart server, requires session persistence (sticky sessions)
├── Service D: Image processing service, requests vary from 100ms to 30s
└── Service E: Mixed cluster: 2 high-capacity servers + 1 low-capacity server
```

## Tasks

### Part A: Algorithm Characteristics

For each load balancing algorithm, describe its core behavior and identify one strength and one weakness.

| Algorithm | How It Works | Strength | Weakness |
|-----------|-------------|----------|----------|
| Round-Robin | | | |
| Weighted Round-Robin | | | |
| Least-Connections | | | |
| IP Hash | | | |
| Random | | | |

<details>
<parameter name="hint">Think about what each algorithm "knows" about the backends. Round-robin only needs a list. Least-connections needs to track active connections. IP hash needs the client IP. What information does each use to make its decision?</details>

### Part B: Match Algorithm to Service

For each service (A through E), select the best load balancing algorithm and explain why.

| Service | Best Algorithm | Why |
|---------|---------------|-----|
| A (Stateless REST API) | | |
| B (WebSocket server) | | |
| C (E-commerce cart) | | |
| D (Image processing) | | |
| E (Mixed capacity) | | |

<details>
<summary>Hint</summary>

Consider:
- Service A: Requests are uniform, so simple distribution works
- Service B: Connection count matters more than request count
- Service C: The same client must reach the same server
- Service D: One slow request can block a connection for 30 seconds
- Service E: Servers have different capacities

</details>

### Part C: Failure Scenarios

For each algorithm, describe what happens when one backend server becomes unhealthy (stops responding):

1. Round-Robin with 4 servers, server 2 goes down
2. Least-Connections with 3 servers, server 1 has 100 connections and goes down
3. IP Hash with 3 servers, server 3 goes down

<details>
<summary>Hint</summary>

Think about:
- Does the algorithm detect the failure automatically?
- Are existing connections affected?
- How quickly does traffic redistribute?
- Do clients get routed to the failed server before the failure is detected?

</details>

## Success Criteria

- [ ] You can describe how each load balancing algorithm distributes traffic
- [ ] You can identify the best algorithm for each service scenario
- [ ] You understand how each algorithm handles server failures
- [ ] You can explain the trade-offs between simplicity and intelligence

## What You Should Understand After This Exercise

No single load balancing algorithm is best for all scenarios. Round-robin is simple but ignores server state. Least-connections adapts to load but requires tracking. IP hash provides persistence but can create imbalances. The right choice depends on whether your requests are uniform or variable, whether you need session persistence, and whether your backends have equal capacity.
