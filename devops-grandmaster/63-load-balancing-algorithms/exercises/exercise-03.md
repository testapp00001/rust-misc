# Exercise 03: Least-Connections vs IP Hash

**Type:** Independent
**Time:** 35 min
**Difficulty:** Medium

## Objective

Compare least-connections and IP hash load balancing algorithms in detail, understanding when each is appropriate and how they handle different traffic patterns.

## Scenario

You are designing the load balancing strategy for two different applications:

```
Application 1: Chat Service
  - Long-lived WebSocket connections (avg 2 hours)
  - 10,000 concurrent users
  - Users connect from diverse IPs
  - Backend servers have limited connection capacity (5,000 each)

Application 2: E-Commerce API
  - Short HTTP requests (avg 100ms)
  - Session state stored in server memory (shopping cart)
  - 50,000 requests/second
  - 3 backend servers with equal capacity
```

## Tasks

### Part A: Least-Connections Deep Dive

The least-connections algorithm routes each new request to the server with the fewest active connections.

1. Draw a diagram showing how least-connections distributes 7 new WebSocket connections across 3 servers, given that Server-A already has 5 connections, Server-B has 3, and Server-C has 8.

2. Explain what happens when Server-B suddenly receives a burst of long-lived connections. How does the algorithm respond?

3. Identify a scenario where least-connections makes a suboptimal decision.

<details>
<summary>Hint</summary>

Least-connections counts active connections, not active requests. A server with 2 long-lived connections might be "less loaded" than a server with 5 fast connections, even though the 5-connection server is actually handling more work. Connection count does not equal load.

</details>

### Part B: IP Hash Deep Dive

The IP hash algorithm hashes the client IP address to select a backend server. The same client IP always goes to the same server.

1. Given 3 servers and these client IPs, calculate which server each client connects to (use a simple modulo hash):

```
Client IPs:
  192.168.1.100  → hash % 3 = ? → Server: ____
  10.0.0.50      → hash % 3 = ? → Server: ____
  172.16.0.200   → hash % 3 = ? → Server: ____
  192.168.1.100  → hash % 3 = ? → Server: ____  (same client reconnects)
  192.168.1.101  → hash % 3 = ? → Server: ____
```

2. What happens when you add a 4th server to the cluster? How many clients get reassigned?

3. What happens when all clients behind a corporate NAT appear as the same IP?

<details>
<summary>Hint</summary>

IP hash provides session persistence without storing session state. But it has problems: adding/removing servers reassigns many clients (consistent hashing helps), and NAT can cause all users from one office to hit the same server, creating hotspots.

</details>

### Part C: Algorithm Comparison

Create a detailed comparison table:

| Criterion | Least-Connections | IP Hash |
|-----------|-------------------|---------|
| Session persistence | | |
| Load distribution accuracy | | |
| Behavior when adding a server | | |
| Behavior when removing a server | | |
| Handling of varying request durations | | |
| Handling of NAT/masquerading | | |
| State required on load balancer | | |
| Best for | | |

<details>
<summary>Hint</summary>

Least-connections: no persistence, adapts to real-time load, connections redistribute on topology change, good for varying durations.
IP hash: provides persistence, can create imbalances, many clients reassign on topology change, bad with NAT.

</details>

### Part D: Design Decision

For the two applications in the scenario:
1. Which algorithm would you choose for the Chat Service? Why?
2. Which algorithm would you choose for the E-Commerce API? Why?
3. Could you combine both approaches? How?

<details>
<summary>Hint</summary>

For the Chat Service: least-connections prevents any single server from exceeding its 5,000 connection limit. For the E-Commerce API: IP hash provides session persistence for shopping carts. But consider: could you store cart state in Redis instead, making IP hash unnecessary?

</details>

## Success Criteria

- [ ] You can explain how least-connections tracks and distributes load
- [ ] You can explain how IP hash provides session persistence
- [ ] You can identify scenarios where each algorithm fails
- [ ] You can make informed algorithm choices based on application requirements

## What You Should Understand After This Exercise

Least-connections and IP hash solve different problems. Least-connections optimizes for even load distribution by tracking active connections, making it ideal for workloads with varying connection durations. IP hash provides session persistence by consistently mapping clients to servers, making it ideal for stateful applications. Neither is universally superior -- the choice depends on whether you need load balancing accuracy or session affinity, and how you handle server topology changes.
